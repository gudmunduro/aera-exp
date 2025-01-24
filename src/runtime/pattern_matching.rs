use crate::types::cst::{ICst, InstantiatedCst};
use crate::types::models::{BoundModel, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::runtime::{
    RuntimeData, RuntimeValue, RuntimeVariable, SystemState,
};
use crate::types::{Fact, TimePatternRange};
use std::collections::HashMap;
use crate::types::pattern::{Pattern, PatternItem};

pub enum PatternMatchResult {
    True(HashMap<String, RuntimeValue>),
    False,
}

pub fn compute_instantiated_states(
    data: &RuntimeData,
    state: &SystemState,
) -> Vec<InstantiatedCst> {
    data.csts
        .iter()
        .filter_map(|(id, cst)| InstantiatedCst::try_instantiate_from_current_state(cst, state))
        .collect()
}

pub fn models_for_cst(instantiated_cst: &InstantiatedCst, data: &RuntimeData) -> Vec<BoundModel> {
    data.models
        .iter()
        .filter_map(|(id, m)| {
            let MdlLeftValue::ICst(icst) = &m.left.pattern else {
                return None;
            };

            match model_lhs_match_cst(&m.left.time_range, icst, instantiated_cst, data) {
                PatternMatchResult::True(bindings) => Some(BoundModel {
                    bindings,
                    model: m.clone(),
                }),
                PatternMatchResult::False => None,
            }
        })
        .collect()
}

pub fn model_lhs_match_cst(
    time_range: &TimePatternRange,
    icst: &ICst,
    instantiated_cst: &InstantiatedCst,
    data: &RuntimeData,
) -> PatternMatchResult {
    instantiated_cst.matches_pattern(&icst.pattern, &HashMap::new())
}

pub fn all_causal_models(data: &RuntimeData) -> Vec<Mdl> {
    data.models
        .iter()
        .filter(|(_, m)| match m {
            Mdl {
                left: Fact { pattern:  MdlLeftValue::Command(_), .. },
                right: Fact { pattern: MdlRightValue::MkVal(_), .. },
                ..
            } => true,
            _ => false,
        })
        .map(|(_, m)| m)
        .cloned()
        .collect()
}

pub fn all_req_models(data: &RuntimeData) -> Vec<Mdl> {
    data.models
        .iter()
        .filter(|(_, m)| match m {
            Mdl {
                left: Fact { pattern: MdlLeftValue::ICst(_), .. },
                right: Fact { pattern: MdlRightValue::IMdl(_), .. },
                ..
            } => true,
            _ => false,
        })
        .map(|(_, m)| m)
        .cloned()
        .collect()
}

pub fn bind_values_to_pattern(pattern: &Pattern, bindings: &HashMap<String, RuntimeValue>) -> Vec<RuntimeValue> {
    pattern.iter()
        .filter_map(|p| match p {
            PatternItem::Any => panic!("Wildcard in parma pattern is currently not supported"),
            PatternItem::Binding(b) => bindings.get(b).map(|v| v.clone()),
            PatternItem::Value(v) => Some(v.clone().into())
        })
        .collect()
}
