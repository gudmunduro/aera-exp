use std::collections::HashMap;
use itertools::Itertools;
use crate::runtime::pattern_matching::{bind_values_to_pattern, compute_instantiated_states, PatternMatchResult};
use crate::types::{Command, EntityVariableKey, Fact, MkVal, PatternItem};
use crate::types::cst::ICst;
use crate::types::pattern::Pattern;
use crate::types::runtime::{RuntimeData, RuntimeValue, SystemState};

pub type GoalName = String;

#[derive(Clone, Debug)]
pub struct Mdl {
    pub model_id: String,
    pub left: Fact<MdlLeftValue>,
    pub right: Fact<MdlRightValue>,
    pub confidence: f64,
}

impl Mdl {
    pub fn binding_params(&self) -> Vec<String> {
        let left_pattern = match &self.left.pattern {
            MdlLeftValue::ICst(cst) => cst.pattern.clone(),
            MdlLeftValue::Command(cmd) => cmd.params.clone(),
            MdlLeftValue::MkVal(mk_val) => vec![mk_val.value.clone()],
        };
        let right_pattern = match &self.right.pattern {
            MdlRightValue::IMdl(imdl) => imdl.params.clone(),
            MdlRightValue::MkVal(mk_val) => vec![mk_val.value.clone()],
            MdlRightValue::Goal(_) => Vec::new(),
        };
        vec![left_pattern, right_pattern]
            .concat()
            .into_iter()
            .filter_map(|pattern| match &pattern {
                PatternItem::Binding(name) => Some(name.clone()),
                _ => None
            })
            .unique()
            .collect()
    }

    /// Attempt to instantiate this model using the lhs icst instruction
    pub fn try_instantiate_with_icst(&self, state: &SystemState) -> Option<BoundModel> {
        let icst = match &self.left.pattern {
            MdlLeftValue::ICst(icst) => icst,
            _ => return None
        };
        let instantiated_cst = state.instansiated_csts.get(&icst.cst_id).unwrap();

        match instantiated_cst.matches_pattern(&icst.pattern, &HashMap::new()) {
            PatternMatchResult::True(bindings) => Some(BoundModel {
                bindings,
                model: self.clone()
            }),
            PatternMatchResult::False => None,
        }
    }

    /// Get a bound version of this model from rhs imdl and an already bound model of the same type as in imdl
    pub fn backward_chain_known_bindings_from_imdl(&self, bound_model: &BoundModel) -> BoundModel {
        let imdl = self.right.pattern.as_imdl();
        let bindings = bound_model.model.binding_params()
            .iter()
            .zip(&imdl.params)
            .filter_map(|(param, pattern)| match pattern {
                PatternItem::Binding(b) => bound_model.bindings.get(param)
                    .map(|value| (b.clone(), value.clone())),
                _ => None
            })
            .collect();

        BoundModel {
            model: self.clone(),
            bindings
        }
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
            _ => panic!("Lhs needs to be icst in model")
        }
    }

    pub fn as_command(&self) -> &Command {
        match self {
            MdlLeftValue::Command(cmd) => cmd,
            _ => panic!("Lhs needs to be a command in model")
        }
    }

    pub fn as_mk_val(&self) -> &MkVal {
        match self {
            MdlLeftValue::MkVal(mk_val) => mk_val,
            _ => panic!("Lhs needs to be mk.val in model")
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
            _ => panic!("Rhs needs to be imdl in model")
        }
    }

    pub fn as_mk_val(&self) -> &MkVal {
        match self {
            MdlRightValue::MkVal(mk_val) => mk_val,
            _ => panic!("Rhs needs to be mk.val in model")
        }
    }

    pub fn as_goal(&self) -> &str {
        match self {
            MdlRightValue::Goal(goal) => goal,
            _ => panic!("Rhs needs to be a goal in model")
        }
    }
}

#[derive(Clone, Debug)]
pub struct IMdl {
    pub model_id: String,
    pub params: Pattern
}

impl IMdl {
    pub fn expand(&self, data: &RuntimeData) -> Mdl {
        let model = data.models.get(&self.model_id).expect(&format!("Model in imdl does not exist {}", self.model_id)).clone();

        // TODO: Replace bindings in in lhs and rhs patterns with params in imdl

        unimplemented!()
    }

    pub fn map_bindings_to_model(&self, bindings: &HashMap<String, RuntimeValue>, data: &RuntimeData) -> HashMap<String, RuntimeValue> {
        let model = data.models.get(&self.model_id).unwrap();
        model.binding_params()
            .iter()
            .zip(&self.params)
            .filter_map(|(binding_name, value)| value
                .get_value_with_bindings(bindings)
                .map(|v| (binding_name.clone(), v)))
            .collect()
    }

    pub fn instantiate(&self, bindings: &HashMap<String, RuntimeValue>, data: &RuntimeData) -> BoundModel {
        // TODO: Stop using bind_values_to_pattern here and allow instantiating with part of bindings regardless of position
        let params = bind_values_to_pattern(&self.params, bindings);
        let model = data.models.get(&self.model_id).expect(&format!("Model in imdl does not exist {}", self.model_id)).clone();
        let binding_params = model.binding_params().into_iter().zip(params).collect::<HashMap<_, _>>();

        BoundModel {
            bindings: binding_params,
            model
        }
    }
}

#[derive(Clone, Debug)]
pub struct BoundModel {
    pub bindings: HashMap<String, RuntimeValue>,
    pub model: Mdl,
}

impl BoundModel {
    pub fn rhs_imdl_matches_bound_model(&self, model: &BoundModel, data: &RuntimeData) -> bool {
        let imdl = self.model.right.pattern.as_imdl();
        let are_bindings_equal = imdl.map_bindings_to_model(&self.bindings, data) == model.bindings;
        let are_models_equal = self.model.model_id == model.model.model_id;
        are_bindings_equal && are_models_equal
    }

    /// Predict what happens to SystemState after model is executed
    /// Only meant to be used for casual models, has no effect on other types of models
    pub fn predict_state_change(&self, state: &SystemState, data: &RuntimeData) -> SystemState {
        let MdlRightValue::MkVal(mk_val) = &self.model.right.pattern else {
            return state.clone();
        };

        let predicted_value = mk_val.value.get_value_with_bindings(&self.bindings)
            .expect("Binding missing when trying to predict state change");

        let mut new_state = state.clone();
        new_state.variables.insert(EntityVariableKey::new(&mk_val.entity_id, &mk_val.var_name), predicted_value);
        new_state.instansiated_csts = compute_instantiated_states(data, &new_state);

        new_state
    }
}