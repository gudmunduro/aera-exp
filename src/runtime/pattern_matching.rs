use crate::types::cst::InstantiatedCst;
use crate::types::models::{IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::{Pattern, PatternItem};
use crate::types::runtime::{System, SystemState};
use crate::types::value::Value;
use crate::types::{Fact, MkVal};
use itertools::Itertools;
use std::collections::HashMap;

pub enum PatternMatchResult {
    True(HashMap<String, Value>),
    False,
}

pub fn compute_instantiated_states(
    system: &System,
    state: &SystemState,
) -> HashMap<String, Vec<InstantiatedCst>> {
    system
        .csts
        .iter()
        .map(|(id, cst)| {
            let csts = InstantiatedCst::try_instantiate_from_current_state(cst, state, system);

            (id.clone(), csts)
        })
        .collect()
}

pub fn all_causal_models(data: &System) -> Vec<Mdl> {
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

pub fn all_req_models(data: &System) -> Vec<Mdl> {
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

pub fn bind_values_to_pattern(pattern: &Pattern, bindings: &HashMap<String, Value>) -> Vec<Value> {
    pattern
        .iter()
        .filter_map(|p| match p {
            PatternItem::Any => panic!("Wildcard in parma pattern is currently not supported"),
            PatternItem::Binding(b) => bindings.get(b).map(|v| v.clone()),
            PatternItem::Value(v) => Some(v.clone()),
        })
        .collect()
}

pub fn state_matches_facts(state: &SystemState, facts: &Vec<Fact<MkVal>>) -> bool {
    facts.iter().all(|f| {
        let Some(entity_key) = f.pattern.entity_key(&HashMap::new()) else {
            return false;
        };
        state
            .variables
            .get(&entity_key)
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

pub fn compare_patterns(
    pattern1: &Pattern,
    pattern2: &Pattern,
    allow_unbound: bool,
    allow_different_length: bool,
) -> bool {
    if !allow_different_length && pattern1.len() != pattern2.len() {
        return false;
    }

    pattern1.iter().zip(pattern2).all(|(p1, p2)| match p1 {
        PatternItem::Any | PatternItem::Binding(_) => allow_unbound,
        PatternItem::Value(v1) => match p2 {
            PatternItem::Any | PatternItem::Binding(_) => allow_unbound,
            PatternItem::Value(v2) => v1 == v2,
        },
    })
}

pub fn compare_imdls(
    imdl1: &IMdl,
    imdl2: &IMdl,
    allow_unbound: bool,
    allow_different_length: bool,
) -> bool {
    imdl1.model_id == imdl2.model_id
        && compare_patterns(
            &imdl1.params,
            &imdl2.params,
            allow_unbound,
            allow_different_length,
        )
}

/// Combine bound values from both to fill in as many values as possible
/// If both are bound, values from pattern1 are preferred
pub fn combine_pattern_bindings(mut pattern1: Pattern, mut pattern2: Pattern) -> Pattern {
    // If they are different length, fill in the rest with wildcard pattern
    if pattern1.len() > pattern2.len() {
        pattern2.extend(
            (0..(pattern1.len() - pattern2.len()))
                .map(|_| PatternItem::Any)
                .collect_vec(),
        );
    } else if pattern1.len() < pattern2.len() {
        pattern1.extend(
            (0..(pattern2.len() - pattern1.len()))
                .map(|_| PatternItem::Any)
                .collect_vec(),
        );
    }

    pattern1
        .into_iter()
        .zip(pattern2)
        .map(|(p1, p2)| match (p1, p2) {
            // Only left is a value
            (
                vp @ PatternItem::Value(_),
                PatternItem::Binding(_) | PatternItem::Any | PatternItem::Value(_),
            ) => vp,
            // Only right is a value
            (PatternItem::Binding(_) | PatternItem::Any, vp @ PatternItem::Value(_)) => vp,
            // Neither is a value, but at least one side is a binding (prefer left)
            (bp @ PatternItem::Binding(_), PatternItem::Any | PatternItem::Binding(_))
            | (PatternItem::Any, bp @ PatternItem::Binding(_)) => bp,
            // Both are wildcard
            (PatternItem::Any, PatternItem::Any) => PatternItem::Any,
        })
        .collect()
}

pub fn fill_in_pattern_with_bindings(
    pattern: Pattern,
    bindings: &HashMap<String, Value>,
) -> Pattern {
    pattern
        .into_iter()
        .map(|p| match &p {
            PatternItem::Binding(b) => {
                if let Some(v) = bindings.get(b).cloned() {
                    PatternItem::Value(v)
                } else {
                    p
                }
            }
            _ => p,
        })
        .collect()
}

pub fn extract_bindings_from_pattern(
    pattern_with_bindings: &Pattern,
    pattern_with_values: &Pattern,
) -> HashMap<String, Value> {
    pattern_with_bindings
        .iter()
        .zip(pattern_with_values)
        .filter_map(|(b, v)| match (b, v) {
            (PatternItem::Binding(b), PatternItem::Value(v)) => Some((b.to_owned(), v.to_owned())),
            _ => None,
        })
        .collect()
}
