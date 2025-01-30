use crate::runtime::pattern_matching::{compute_instantiated_states, PatternMatchResult};
use crate::types::cst::ICst;
use crate::types::functions::Function;
use crate::types::pattern::Pattern;
use crate::types::runtime::{RuntimeValue, System, SystemState};
use crate::types::{Command, EntityPatternValue, EntityVariableKey, Fact, MkVal, PatternItem};
use itertools::Itertools;
use std::collections::HashMap;
use simple_log::new;
use tap::Tap;

pub type GoalName = String;

#[derive(Clone, Debug)]
pub struct Mdl {
    pub model_id: String,
    pub left: Fact<MdlLeftValue>,
    pub right: Fact<MdlRightValue>,
    pub confidence: f64,
    pub forward_computed: HashMap<String, Function>,
    pub backward_computed: HashMap<String, Function>,
}

impl Mdl {
    pub fn binding_param(&self) -> Vec<String> {
        let left_pattern = match &self.left.pattern {
            MdlLeftValue::ICst(cst) => cst.pattern.clone(),
            MdlLeftValue::Command(cmd) => {
                if let EntityPatternValue::Binding(b) = &cmd.entity_id {
                    // Make PatternItem binding for the entity id binding as well so it appears first in params
                    vec![PatternItem::Binding(b.clone())]
                        .into_iter()
                        .chain(cmd.params.clone())
                        .collect_vec()
                } else {
                    cmd.params.clone()
                }
            }
            MdlLeftValue::MkVal(mk_val) => {
                if let EntityPatternValue::Binding(b) = &mk_val.entity_id {
                    // Make PatternItem binding for the entity id binding as well so it appears first in params
                    vec![PatternItem::Binding(b.clone()), mk_val.value.clone()]
                } else {
                    vec![mk_val.value.clone()]
                }
            }
        };
        let params_in_computed = self
            .forward_computed
            .iter()
            .flat_map(|(_, f)| f.binding_params());

        left_pattern
            .into_iter()
            .filter_map(|pattern| match &pattern {
                PatternItem::Binding(name) => Some(name.clone()),
                _ => None,
            })
            .chain(params_in_computed.into_iter())
            .filter(|b| !self.forward_computed.contains_key(b))
            .unique()
            .collect()
    }

    /// Attempt to instantiate this model using the lhs icst instruction
    pub fn try_instantiate_with_icst(&self, state: &SystemState) -> Option<BoundModel> {
        let icst = match &self.left.pattern {
            MdlLeftValue::ICst(icst) => icst,
            _ => return None,
        };
        for instantiated_cst in state.instansiated_csts.get(&icst.cst_id)? {
            match instantiated_cst.matches_param_pattern(&icst.pattern, &HashMap::new()) {
                PatternMatchResult::True(bindings) => {
                    return Some(
                        BoundModel {
                            bindings,
                            model: self.clone(),
                        }
                        .tap_mut(|m| m.compute_forward_bindings()),
                    )
                }
                PatternMatchResult::False => continue,
            }
        }

        None
    }

    /// Get a bound version of this model from rhs imdl and an already bound model of the same type as in imdl
    pub fn backward_chain_known_bindings_from_imdl(&self, bound_model: &BoundModel) -> BoundModel {
        let imdl = self.right.pattern.as_imdl();
        let bindings = bound_model
            .model
            .binding_param()
            .iter()
            .zip(&imdl.params)
            .filter_map(|(param, pattern)| match pattern {
                PatternItem::Binding(b) => bound_model
                    .bindings
                    .get(param)
                    .map(|value| (b.clone(), value.clone())),
                _ => None,
            })
            .collect();

        BoundModel {
            model: self.clone(),
            bindings,
        }
        .tap_mut(|m| m.compute_backward_bindings())
    }
}

#[derive(Clone, Debug)]
pub enum MdlLeftValue {
    ICst(ICst),
    Command(Command),
    MkVal(MkVal),
}

impl MdlLeftValue {
    pub fn as_icst(&self) -> &ICst {
        match self {
            MdlLeftValue::ICst(icst) => icst,
            _ => panic!("Lhs needs to be icst in model"),
        }
    }

    pub fn as_command(&self) -> &Command {
        match self {
            MdlLeftValue::Command(cmd) => cmd,
            _ => panic!("Lhs needs to be a command in model"),
        }
    }

    pub fn as_mk_val(&self) -> &MkVal {
        match self {
            MdlLeftValue::MkVal(mk_val) => mk_val,
            _ => panic!("Lhs needs to be mk.val in model"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum MdlRightValue {
    IMdl(IMdl),
    MkVal(MkVal),
    Goal(GoalName),
}

impl MdlRightValue {
    pub fn as_imdl(&self) -> &IMdl {
        match self {
            MdlRightValue::IMdl(imdl) => imdl,
            _ => panic!("Rhs needs to be imdl in model"),
        }
    }

    pub fn as_mk_val(&self) -> &MkVal {
        match self {
            MdlRightValue::MkVal(mk_val) => mk_val,
            _ => panic!("Rhs needs to be mk.val in model"),
        }
    }

    pub fn as_goal(&self) -> &str {
        match self {
            MdlRightValue::Goal(goal) => goal,
            _ => panic!("Rhs needs to be a goal in model"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IMdl {
    pub model_id: String,
    pub params: Pattern,
}

impl IMdl {
    pub fn map_bindings_to_model(
        &self,
        bindings: &HashMap<String, RuntimeValue>,
        data: &System,
    ) -> HashMap<String, RuntimeValue> {
        // TODO: Potentially needs to be fixed so it works with both forward and backward chaining
        let model = data.models.get(&self.model_id).unwrap();
        model
            .binding_param()
            .iter()
            .zip(&self.params)
            .filter_map(|(binding_name, value)| {
                value
                    .get_value_with_bindings(bindings)
                    .map(|v| (binding_name.clone(), v))
            })
            .collect()
    }

    pub fn instantiate(
        &self,
        bindings: &HashMap<String, RuntimeValue>,
        data: &System,
    ) -> BoundModel {
        let model = data
            .models
            .get(&self.model_id)
            .expect(&format!("Model in imdl does not exist {}", self.model_id))
            .clone();
        let bindings = self.map_bindings_to_model(bindings, data);

        BoundModel { bindings, model }.tap_mut(|m| m.compute_forward_bindings())
    }
}

#[derive(Clone, Debug)]
pub struct BoundModel {
    pub bindings: HashMap<String, RuntimeValue>,
    pub model: Mdl,
}

impl BoundModel {
    pub fn rhs_imdl_matches_bound_model(&self, model: &BoundModel, data: &System) -> bool {
        let imdl = self.model.right.pattern.as_imdl();
        let are_bindings_equal = imdl.map_bindings_to_model(&self.bindings, data) == model.bindings;
        let are_models_equal = self.model.model_id == model.model.model_id;
        are_bindings_equal && are_models_equal
    }

    /// Predict what happens to SystemState after model is executed
    /// Only meant to be used for casual models, has no effect on other types of models
    /// At the moment this does not take into account changes that other models predict will take place when the same action is taken
    pub fn predict_state_change(
        &self,
        state: &SystemState,
        other_casual_models: &Vec<&BoundModel>,
        system: &System,
    ) -> SystemState {
        let MdlLeftValue::Command(cmd) = &self.model.left.pattern else {
            return state.clone();
        };
        let MdlRightValue::MkVal(mk_val) = &self.model.right.pattern else {
            return state.clone();
        };
        let Ok(runtime_cmd) = cmd.to_runtime_command(&self.bindings) else {
            log::error!(
                "Cannot get runtime command when tyring to predict state change in {}",
                self.model.model_id
            );
            return state.clone();
        };
        // Find all other models that predict a state change based on the exact same command, and use those predictions as well
        let other_predicted_changes = other_casual_models
            .iter()
            .filter_map(|m| {
                match m
                    .model
                    .left
                    .pattern
                    .as_command()
                    .to_runtime_command(&m.bindings)
                {
                    Ok(cmd) if runtime_cmd == cmd => Some(m),
                    _ => None,
                }
            })
            .filter_map(|m| {
                let mk_val = m.model.right.pattern.as_mk_val();
                let entity_id = mk_val.entity_id.get_id_with_bindings(&m.bindings)?;
                let value = mk_val
                    .value
                    .get_value_with_bindings(&m.bindings)?;
                Some((EntityVariableKey { entity_id, var_name: mk_val.var_name.to_owned() }, value))
            })
            .collect_vec();

        let predicted_value = mk_val
            .value
            .get_value_with_bindings(&self.bindings)
            .expect("Binding missing when trying to predict state change");

        let mut new_state = state.clone();
        new_state.variables.extend(other_predicted_changes);
        new_state.variables.insert(
            EntityVariableKey::new(
                &mk_val
                    .entity_id
                    .get_id_with_bindings(&self.bindings)
                    .expect("Entity binding missing when performing state change"),
                &mk_val.var_name,
            ),
            predicted_value,
        );
        new_state.instansiated_csts = compute_instantiated_states(system, &new_state);

        new_state
    }

    /// Add bindings from `bindings` for variables which were not bound before
    pub fn fill_missing_bindings(&mut self, bindings: &HashMap<String, RuntimeValue>) {
        self.bindings.extend(
            bindings
                .iter()
                .filter(|(b, _)| !self.bindings.contains_key(*b))
                .map(|(b, v)| (b.clone(), v.clone()))
                .collect_vec(),
        )
    }

    pub fn compute_forward_bindings(&mut self) {
        for (binding, function) in &self.model.forward_computed {
            if let Some(res) = function.evaluate(&self.bindings) {
                self.bindings.insert(binding.clone(), res);
            }
        }
    }

    pub fn compute_backward_bindings(&mut self) {
        for (binding, function) in &self.model.backward_computed {
            if let Some(res) = function.evaluate(&self.bindings) {
                self.bindings.insert(binding.clone(), res);
            }
        }
    }
}
