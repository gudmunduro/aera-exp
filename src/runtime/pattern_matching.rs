use crate::types::cst::InstantiatedCst;
use crate::types::models::{Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::{Pattern, PatternItem};
use crate::types::runtime::{RuntimeData, RuntimeValue, SystemState};
use crate::types::{Fact, MkVal};
use std::collections::HashMap;

pub enum PatternMatchResult {
    True(HashMap<String, RuntimeValue>),
    False,
}

pub fn compute_instantiated_states(
    data: &RuntimeData,
    state: &SystemState,
) -> HashMap<String, InstantiatedCst> {
    data.csts
        .iter()
        .filter_map(|(id, cst)| {
            InstantiatedCst::try_instantiate_from_current_state(cst, state)
                .map(|cst| (id.clone(), cst))
        })
        .collect()
}

pub fn all_causal_models(data: &RuntimeData) -> Vec<Mdl> {
    data.models
        .iter()
        .filter(|(_, m)| match m {
            Mdl {
                left:
                    Fact {
                        pattern: MdlLeftValue::Command(_),
                        ..
                    },
                right:
                    Fact {
                        pattern: MdlRightValue::MkVal(_),
                        ..
                    },
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
                left:
                    Fact {
                        pattern: MdlLeftValue::ICst(_),
                        ..
                    },
                right:
                    Fact {
                        pattern: MdlRightValue::IMdl(_),
                        ..
                    },
                ..
            } => true,
            _ => false,
        })
        .map(|(_, m)| m)
        .cloned()
        .collect()
}

pub fn bind_values_to_pattern(
    pattern: &Pattern,
    bindings: &HashMap<String, RuntimeValue>,
) -> Vec<RuntimeValue> {
    pattern
        .iter()
        .filter_map(|p| match p {
            PatternItem::Any => panic!("Wildcard in parma pattern is currently not supported"),
            PatternItem::Binding(b) => bindings.get(b).map(|v| v.clone()),
            PatternItem::Value(v) => Some(v.clone().into()),
        })
        .collect()
}

pub fn state_matches_facts(state: &SystemState, facts: &Vec<Fact<MkVal>>) -> bool {
    facts.iter().all(|f| {
        state
            .variables
            .get(&f.pattern.entity_key())
            .map(|v| *v == f.pattern.value)
            .unwrap_or(false)
    })
}

/// Goals are considered equal even if timing is not the same
pub fn are_goals_equal(goal1: &Vec<Fact<MkVal>>, goal2: &Vec<Fact<MkVal>>) -> bool {
    goal1.len() == goal2.len()
        && goal1
            .iter()
            .all(|f1| goal2.iter().any(|f2| f1.pattern == f2.pattern))
}
