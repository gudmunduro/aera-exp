use crate::runtime::pattern_matching::{combine_pattern_bindings, compute_instantiated_states, extract_bindings_from_pattern, fill_in_pattern_with_bindings, PatternMatchResult};
use crate::types::cst::ICst;
use crate::types::functions::Function;
use crate::types::pattern::Pattern;
use crate::types::runtime::{System, SystemState};
use crate::types::{Command, EntityPatternValue, EntityVariableKey, Fact, MkVal, PatternItem};
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use tap::Tap;
use crate::types::value::Value;

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
            MdlLeftValue::ICst(cst) => cst.params.clone(),
            MdlLeftValue::Command(cmd) => {
                if let EntityPatternValue::Binding(b) = &cmd.entity_id {
                    // Add entity id binding as well so it appears first in params
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
                    // Add entity id binding as well so it appears first in params
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
        // TODO: Temporary hack to make grab command work
        let right_params = match &self.right.pattern {
            MdlRightValue::IMdl(imdl) => imdl.params.clone(),
            MdlRightValue::MkVal(mk_val) => {
                if let EntityPatternValue::Binding(b) = &mk_val.entity_id {
                    vec![PatternItem::Binding(b.clone()), mk_val.value.clone()]
                } else {
                    vec![mk_val.value.clone()]
                }
            },
        }.into_iter().filter_map(|pattern| match &pattern {
            PatternItem::Binding(name) => Some(name.clone()),
            _ => None,
        });

        left_pattern
            .into_iter()
            .filter_map(|pattern| match &pattern {
                PatternItem::Binding(name) => Some(name.clone()),
                _ => None,
            })
            .chain(params_in_computed.into_iter())
            .chain(right_params)
            // Bindings assigned to function results cannot be passes as parameters
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
            match instantiated_cst.match_and_get_bindings_for_icst(&icst) {
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

    /// Get a bound version of this model from rhs imdl
    pub fn backward_chain_known_bindings_from_imdl(&self, imdl: &IMdl) -> BoundModel {
        let self_imdl = self.right.pattern.as_imdl();
        let bindings = extract_bindings_from_pattern(&self_imdl.params, &imdl.params);

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
}

impl MdlRightValue {
    pub fn as_imdl(&self) -> &IMdl {
        match self {
            MdlRightValue::IMdl(imdl) => imdl,
            _ => panic!("Rhs needs to be imdl in model"),
        }
    }

    pub fn as_filled_in_imdl(&self, bindings: &HashMap<String, Value>) -> IMdl {
        let mut imdl = self.as_imdl().clone();
        imdl.params = fill_in_pattern_with_bindings(imdl.params, bindings);
        imdl
    }

    pub fn as_mk_val(&self) -> &MkVal {
        match self {
            MdlRightValue::MkVal(mk_val) => mk_val,
            _ => panic!("Rhs needs to be mk.val in model"),
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
        bindings: &HashMap<String, Value>,
        data: &System,
    ) -> HashMap<String, Value> {
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
        bindings: &HashMap<String, Value>,
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

    pub fn merge_with(mut self, imdl: IMdl) -> IMdl {
        self.params = combine_pattern_bindings(self.params, imdl.params);
        self
    }
}

impl Display for IMdl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(imdl {} {})", self.model_id, self.params.iter().map(|p| p.to_string()).join(" "))?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BoundModel {
    pub bindings: HashMap<String, Value>,
    pub model: Mdl,
}

impl BoundModel {
    /// Create imdl that would be used to instantiate this model, including known binding values.
    /// Can be used to compare it to imdl on rhs of req. models
    pub fn imdl_for_model(&self) -> IMdl {
        let binding_params = self.model.binding_param();
        let param_pattern = binding_params.iter()
            .map(|b| match self.bindings.get(b) {
                Some(v) => PatternItem::Value(v.clone()),
                // There is no specific binding in the imdl, since those depend on the model the imdl is in
                None => PatternItem::Any
            })
            .collect();

        IMdl {
            model_id: self.model.model_id.clone(),
            params: param_pattern,
        }
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
    pub fn fill_missing_bindings(&mut self, bindings: &HashMap<String, Value>) {
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
