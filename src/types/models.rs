use std::collections::HashMap;
use itertools::Itertools;
use crate::runtime::pattern_matching::{bind_values_to_pattern, PatternMatchResult};
use crate::types::{Command, Fact, MkVal, PatternItem};
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

    pub fn instantiate(&self, bindings: &HashMap<String, RuntimeValue>, data: &RuntimeData) -> BoundModel {
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