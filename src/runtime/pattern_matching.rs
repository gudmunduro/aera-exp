use crate::types::models::{IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::{Pattern, PatternItem};
use crate::types::runtime::{System, SystemState};
use crate::types::value::Value;
use crate::types::{EntityVariableKey, Fact, MkVal};
use itertools::Itertools;
use std::collections::HashMap;
use crate::types::cst::{BoundCst, ICst};

pub enum PatternMatchResult {
    True(HashMap<String, Value>),
    False,
}

pub fn compute_instantiated_states(
    system: &System,
    state: &SystemState,
) -> HashMap<String, Vec<BoundCst>> {
    system
        .csts
        .iter()
        .map(|(id, cst)| {
            let csts = BoundCst::try_instantiate_from_state(cst, state, system);

            (id.clone(), csts)
        })
        .collect()
}

pub fn compute_assumptions(system: &System, state: &SystemState) -> HashMap<EntityVariableKey, Value> {
    let models = all_assumption_models(&system)
        .into_iter()
        .flat_map(|m| m.try_instantiate_with_icst(state))
        .collect_vec();
    models.into_iter()
        .filter_map(|m| match m.model.right.pattern {
            MdlRightValue::MkVal(rhs @ MkVal { assumption: true, .. }) => {
                let entity_id = rhs.entity_id.get_id_with_bindings(&m.bindings)
                    .expect("Cannot fill in entity id binding of assumption");
                let value = rhs.value.get_value_with_bindings(&m.bindings)
                    .expect("Cannot fill in all bindings of assumption");
                Some((EntityVariableKey { entity_id, var_name: rhs.var_name }, value))
            },
            _ => None
        })
        .collect()
}

pub fn all_causal_models(data: &System) -> Vec<Mdl> {
    data.models
        .iter()
        .filter(|(_, m)| m.is_casual_model())
        .map(|(_, m)| m)
        .cloned()
        .collect()
}

pub fn all_req_models(data: &System) -> Vec<Mdl> {
    data.models
        .iter()
        .filter(|(_, m)| m.is_req_model())
        .map(|(_, m)| m)
        .cloned()
        .collect()
}

pub fn all_assumption_models(data: &System) -> Vec<Mdl> {
    data.models
        .iter()
        .filter(|(_, m)| m.is_assumption_model())
        .map(|(_, m)| m)
        .cloned()
        .collect()
}

pub fn all_state_prediction_models(data: &System) -> Vec<Mdl> {
    data.models
        .iter()
        .filter(|(_, m)| m.is_state_prediction())
        .map(|(_, m)| m)
        .cloned()
        .collect()
}

pub fn bind_values_to_pattern(pattern: &Pattern, bindings: &HashMap<String, Value>) -> Vec<Value> {
    pattern
        .iter()
        .filter_map(|p| match p {
            PatternItem::Any => panic!("Wildcard in parma pattern is not supported"),
            PatternItem::Binding(b) => bindings.get(b).map(|v| v.clone()),
            PatternItem::Value(v) => Some(v.clone()),
            PatternItem::Vec(v) => Some(Value::Vec(bind_values_to_pattern(v, bindings))),
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

    pattern1.iter().zip(pattern2).all(|(p1, p2)| compare_pattern_items(p1, p2, allow_unbound))
}

pub fn compare_pattern_items(
    pattern_item1: &PatternItem,
    pattern_item2: &PatternItem,
    allow_unbound: bool,
) -> bool {
    match pattern_item1 {
        PatternItem::Any | PatternItem::Binding(_) => allow_unbound,
        PatternItem::Value(v1) => match pattern_item2 {
            PatternItem::Any | PatternItem::Binding(_) => allow_unbound,
            PatternItem::Value(v2) => v1 == v2,
            PatternItem::Vec(v2) => match v1 {
                Value::Vec(v1) => v1 == v2,
                _ => false,
            },
        },
        PatternItem::Vec(v1) => match pattern_item2 {
            PatternItem::Any | PatternItem::Binding(_) => allow_unbound,
            PatternItem::Vec(v2) => compare_patterns(v1, v2, allow_unbound, false),
            PatternItem::Value(Value::Vec(v2)) => v2 == v1,
            PatternItem::Value(_) => false,
        }
    }
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

pub fn compare_icsts(
    icst1: &ICst,
    icst2: &ICst,
    allow_unbound: bool,
    allow_different_length: bool,
) -> bool {
    icst1.cst_id == icst2.cst_id
        && compare_patterns(
        &icst1.params,
        &icst2.params,
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
            // Both are vec patterns
            (PatternItem::Vec(p1), PatternItem::Vec(p2)) => PatternItem::Vec(combine_pattern_bindings(p1, p2)),
            // Both are vec but left is value
            (PatternItem::Value(Value::Vec(v1)), PatternItem::Vec(p2)) => PatternItem::Vec(combine_pattern_bindings(value_vec_to_pattern_vec(v1), p2)),
            // Both are vec but right is value
            (PatternItem::Vec(p1), PatternItem::Value(Value::Vec(v2))) => PatternItem::Vec(combine_pattern_bindings(p1, value_vec_to_pattern_vec(v2))),
            // Only left is a value
            (
                vp @ (PatternItem::Value(_) | PatternItem::Vec(_)),
                PatternItem::Binding(_) | PatternItem::Any | PatternItem::Value(_) | PatternItem::Vec(_),
            ) => vp,
            // Only right is a value
            (PatternItem::Binding(_) | PatternItem::Any, vp @ (PatternItem::Value(_) | PatternItem::Vec(_))) => vp,
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
        .map(|mut p| {
            p.insert_binding_values(bindings);
            p
        })
        .collect()
}

pub fn extract_bindings_from_patterns(
    pattern_with_bindings: &Pattern,
    pattern_with_values: &Pattern,
) -> HashMap<String, Value> {
    pattern_with_bindings
        .iter()
        .zip(pattern_with_values)
        .flat_map(|(b, v)| match (b, v) {
            (PatternItem::Binding(b), PatternItem::Value(v)) => vec![(b.to_owned(), v.to_owned())],
            (PatternItem::Binding(b), PatternItem::Vec(v)) => {
                if let Some(vec) = pattern_vec_to_value_vec(v.clone()) {
                    vec![(b.to_owned(), Value::Vec(vec))]
                }
                else {
                    log::error!("Could not extract bindings from partially bound vec");
                    log::error!("Binding vec [{}], value vec [{}]", pattern_with_bindings.iter().map(|p| p.to_string()).join(", "), pattern_with_values.iter().map(|p| p.to_string()).join(", "));
                    Vec::new()
                }
            },
            (PatternItem::Vec(bv), PatternItem::Vec(vv)) => extract_duplicate_bindings_from_pattern(bv, vv),
            (PatternItem::Vec(bv), PatternItem::Value(Value::Vec(vv))) => extract_duplicate_bindings_from_pattern_and_values(bv, vv),
            _ => Vec::new(),
        })
        .collect()
}

/// Extract all bindings from pattern but allow duplicate (one binding with multiple values)
pub fn extract_duplicate_bindings_from_pattern(
    pattern_with_bindings: &Pattern,
    pattern_with_values: &Pattern,
) -> Vec<(String, Value)> {
    pattern_with_bindings
        .iter()
        .zip(pattern_with_values)
        .flat_map(|(b, v)| match (b, v) {
            (PatternItem::Binding(b), PatternItem::Value(v)) => vec![(b.to_owned(), v.to_owned())],
            (PatternItem::Vec(bv), PatternItem::Vec(vv)) => extract_duplicate_bindings_from_pattern(bv, vv),
            (PatternItem::Vec(bv), PatternItem::Value(Value::Vec(vv))) => extract_duplicate_bindings_from_pattern_and_values(bv, vv),
            _ => Vec::new(),
        })
        .collect()
}

pub fn extract_duplicate_bindings_from_pattern_and_values(
    pattern_with_bindings: &Pattern,
    values: &Vec<Value>,
) -> Vec<(String, Value)> {
    pattern_with_bindings
        .iter()
        .zip(values)
        .flat_map(|(b, v)| match (b, v) {
            (PatternItem::Binding(b), v) => vec![(b.to_owned(), v.to_owned())],
            (PatternItem::Vec(bv), Value::Vec(vv)) => extract_duplicate_bindings_from_pattern_and_values(bv, vv),
            _ => Vec::new(),
        })
        .collect()
}

/// Checks if a patten item matches a value
/// Temporarily takes ownership of the binding map, and gives a updated one with new bindings if the pattern matches
pub fn pattern_item_matches_value_with_bindings(pattern_item: &PatternItem, value: &Value, mut binding_map: HashMap<String, Value>) -> PatternMatchResult {
    match pattern_item {
        PatternItem::Any => {}
        PatternItem::Binding(b) => {
            if let Some(bound_val) = binding_map.get(b) {
                // If value was already bound before (with another variable), compare to that var
                if bound_val != value {
                    return PatternMatchResult::False;
                }
            }
            else {
                // Add value to the binding map if we have not seen this
                binding_map.insert(b.clone(), value.clone());
            }
        }
        PatternItem::Value(v1) => {
            // Don't instantiate model if value doesn't match current state
            if v1 != value {
                return PatternMatchResult::False;
            }
        }
        PatternItem::Vec(v1) => {
            match value {
                Value::Vec(v2) if v1.len() == v2.len() => {
                    for (v1, v2) in v1.iter().zip(v2) {
                        match pattern_item_matches_value_with_bindings(v1, &v2, binding_map) {
                            PatternMatchResult::True(updated_bindings) => {
                                binding_map = updated_bindings;
                            }
                            PatternMatchResult::False => {
                                return PatternMatchResult::False;
                            }
                        }
                    }
                }
                _ => return PatternMatchResult::False,
            }
        }
    }

    PatternMatchResult::True(binding_map)
}

pub fn value_vec_to_pattern_vec(pattern: Vec<Value>) -> Vec<PatternItem> {
    pattern.into_iter().map(PatternItem::Value).collect()
}

pub fn pattern_vec_to_value_vec(pattern: Vec<PatternItem>) -> Option<Vec<Value>> {
    pattern.into_iter().map(|p| match p {
        PatternItem::Binding(_) | PatternItem::Any => None,
        PatternItem::Value(v) => Some(v),
        PatternItem::Vec(v) => pattern_vec_to_value_vec(v).map(Value::Vec)
    }).collect()
}