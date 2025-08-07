use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use crate::types::{EntityPatternValue, EntityVariableKey, Fact, MkVal};
use crate::types::pattern::PatternItem;
use crate::types::runtime::System;
use crate::types::value::Value;

pub type PatternValueMap = HashMap<ValueKey, String>;

#[derive(Debug, PartialEq, Eq)]
pub struct ValueKey(pub Value);

impl Hash for ValueKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        1.hash(state);
    }
}

pub struct EntityVarChange {
    pub entity: EntityVariableKey,
    pub before: Option<Value>,
    pub after: Value
}

pub fn change_intersects_fact(change: &EntityVarChange, fact: &Fact<MkVal>) -> bool {
    let change_set = extract_values_from_change(change);
    let fact_set = extract_values_from_fact(fact);
    change_set.intersection(&fact_set).count() > 0
}

pub fn change_intersects_entity_var(change: &EntityVarChange, entity_var: (&EntityVariableKey, &Value)) -> bool {
    let change_set = extract_values_from_change(change);
    let entity_var_set = extract_values_from_entity_var(entity_var);
    change_set.intersection(&entity_var_set).count() > 0
}

// Extract all the relevant values from the change, which can then be used to check if specific facts and CSTS are relevant
fn extract_values_from_change(change: &EntityVarChange) -> HashSet<Value> {
    let entity_id = change.entity.entity_id.clone();
    let before_values = if let Some(before) = &change.before {
        extract_values_from_value(before)
    } else {
        HashSet::new()
    };
    let after_values = extract_values_from_value(&change.after);
    let mut res: HashSet<Value> = before_values.union(&after_values).cloned().collect();
    res.insert(Value::EntityId(entity_id));
    res
}

fn extract_values_from_entity_var(entity_var: (&EntityVariableKey, &Value)) -> HashSet<Value> {
    let entity_id = entity_var.0.entity_id.clone();
    let mut values_set = extract_values_from_value(&entity_var.1);
    values_set.insert(Value::EntityId(entity_id));
    values_set
}

fn extract_values_from_fact(fact: &Fact<MkVal>) -> HashSet<Value> {
    let EntityPatternValue::EntityId(entity_id) = &fact.pattern.entity_id else {
        log::error!("Fact in CTPX does not have value for entity, that should never happen");
        return HashSet::new();
    };
    let PatternItem::Value(value) = &fact.pattern.value else {
        log::error!("Fact in CTPX does not have value for value pattern, that should never happen");
        return HashSet::new();
    };
    let mut value_set = extract_values_from_value(value);
    value_set.insert(Value::EntityId(entity_id.clone()));
    value_set
}

fn extract_values_from_value(value: &Value) -> HashSet<Value> {
    match value {
        Value::Number(_) | Value::ConstantNumber(_) | Value::String(_) | Value::EntityId(_) => HashSet::from([value.clone()]),
        Value::UncertainNumber(m, s) => HashSet::from([Value::Number(*m), Value::Number(*s)]),
        Value::Vec(vec) => vec.iter()
            .flat_map(|v| extract_values_from_value(v))
            .collect()
    }
}

pub fn generate_casual_model_name(system: &System) -> String {
    format!("mdl_{}", system.models.len())
}

pub fn generate_req_model_name(system: &System) -> String {
    format!("mdl_req_{}", system.models.len())
}

pub fn generate_anti_req_model_name(system: &System) -> String {
    format!("mdl_anti_req_{}", system.models.len())
}

pub fn generate_cst_name(system: &System) -> String {
    format!("cst_{}", system.csts.len())
}

pub fn compute_vec_norm(values: &Vec<Value>) -> f64 {
    let sum: f64 = values.iter().map(|v| match v {
        Value::UncertainNumber(n, _) | Value::Number(n) => n.powi(2),
        Value::Vec(v) => compute_vec_norm(&v).powi(2),
        _ => panic!("Trying to compute norm of vec with string value")
    }).sum();

    sum.sqrt()
}

pub fn create_pattern_for_values(
    values: &Vec<Value>,
    pattern_value_map: &mut PatternValueMap,
) -> Vec<PatternItem> {
    values
        .iter()
        .map(|v| create_pattern_for_value(v, pattern_value_map, false))
        .collect()
}

pub fn create_pattern_for_value(
    value: &Value,
    pattern_value_map: &mut PatternValueMap,
    insert_constant_for_unknown: bool,
) -> PatternItem {
    match value {
        Value::Number(_) | Value::UncertainNumber(_, _) | Value::String(_) | Value::EntityId(_) => {
            if !pattern_value_map.contains_key(&ValueKey(value.clone())) {
                if insert_constant_for_unknown {
                    PatternItem::Value(value.clone())
                }
                else {
                    let binding = format!("v{}", pattern_value_map.len());
                    pattern_value_map.insert(ValueKey(value.clone()), binding.clone());
                    PatternItem::Binding(binding)
                }
            } else {
                PatternItem::Binding(pattern_value_map[&ValueKey(value.clone())].clone())
            }
        }
        Value::Vec(vec) => PatternItem::Vec(
            vec.iter()
                .map(|v| create_pattern_for_value(v, pattern_value_map, insert_constant_for_unknown))
                .collect(),
        ),
        Value::ConstantNumber(_) => PatternItem::Value(value.clone()),
    }
}

pub fn create_bindings_for_value(
    value: &Value,
    pattern_value_map: &mut PatternValueMap,
    binding_prefix: &str,
    start_index: &mut i32,
) {
    match value {
        Value::Number(_) | Value::UncertainNumber(_, _) | Value::String(_) | Value::EntityId(_) => {
            if !pattern_value_map.contains_key(&ValueKey(value.clone())) {
                pattern_value_map.insert(ValueKey(value.clone()), format!("{binding_prefix}{start_index}"));
                *start_index += 1;
            }
        }
        Value::Vec(vec) => {
            for v in vec {
                create_bindings_for_value(v, pattern_value_map, binding_prefix, start_index);
            }
        }
        // Don't create bindings for constant values
        Value::ConstantNumber(_) => {}
    }
}