use std::collections::HashMap;
use crate::types::EntityPatternValue;
use crate::types::models::{Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::{Pattern, PatternItem};
use crate::types::value::Value;

pub(super) fn compare_casual_models_with_bindings(model1: &Mdl, model2: &Mdl) -> bool {
    // TODO: Compare model1 to model2, if something is a binding: check if binding_map includes it and compare (using the mapping) if it does
    // TODO: if binding_map does not include it, add to binding map
    // TODO: If something is a value, compare the value
    // TODO: Binding map maps bindings from model1 to model2
    let mut binding_map = HashMap::<String, String>::new();

    // Compare left side of models
    let lhs_equal = match (&model1.left.pattern, &model2.left.pattern) {
        (MdlLeftValue::IMdl(imdl1), MdlLeftValue::IMdl(imdl2)) => {
            // Check model IDs are the same
            if imdl1.model_id != imdl2.model_id {
                return false;
            }

            // Compare parameters using binding map
            if !compare_patterns_with_binding_map(&imdl1.params, &imdl2.params, &mut binding_map) {
                return false;
            }

            // Compare forward guard bindings
            compare_fwd_guard_bindings(&imdl1.fwd_guard_bindings, &imdl2.fwd_guard_bindings, &mut binding_map)
        }
        (MdlLeftValue::Command(cmd1), MdlLeftValue::Command(cmd2)) => {
            // Check command names are the same
            if cmd1.name != cmd2.name {
                return false;
            }

            // Compare entity IDs
            if !compare_entity_patterns_with_binding_map(&cmd1.entity_id, &cmd2.entity_id, &mut binding_map) {
                return false;
            }

            // Compare parameters
            compare_patterns_with_binding_map(&cmd1.params, &cmd2.params, &mut binding_map)
        }
        _ => false, // Different pattern types
    };

    if !lhs_equal {
        return false;
    }

    // Compare right side of models
    let rhs_equal = match (&model1.right.pattern, &model2.right.pattern) {
        (MdlRightValue::MkVal(mk_val1), MdlRightValue::MkVal(mk_val2)) => {
            // Check var names are the same
            if mk_val1.var_name != mk_val2.var_name {
                return false;
            }

            // Compare entity IDs
            if !compare_entity_patterns_with_binding_map(&mk_val1.entity_id, &mk_val2.entity_id, &mut binding_map) {
                return false;
            }

            // Compare values
            compare_pattern_items_with_binding_map(&mk_val1.value, &mk_val2.value, &mut binding_map)
        }
        _ => false, // Different pattern types or unsupported
    };

    rhs_equal
}

fn compare_patterns_with_binding_map(
    pattern1: &Pattern,
    pattern2: &Pattern,
    binding_map: &mut HashMap<String, String>,
) -> bool {
    if pattern1.len() != pattern2.len() {
        return false;
    }

    pattern1.iter().zip(pattern2.iter()).all(|(p1, p2)| {
        compare_pattern_items_with_binding_map(p1, p2, binding_map)
    })
}

fn compare_pattern_items_with_binding_map(
    item1: &PatternItem,
    item2: &PatternItem,
    binding_map: &mut HashMap<String, String>,
) -> bool {
    match (item1, item2) {
        (PatternItem::Value(v1), PatternItem::Value(v2)) => v1 == v2,
        (PatternItem::Binding(b1), PatternItem::Binding(b2)) => {
            // Check if binding is already mapped
            if let Some(mapped_binding) = binding_map.get(b1) {
                // Use existing mapping
                mapped_binding == b2
            } else {
                // Add new mapping
                binding_map.insert(b1.clone(), b2.clone());
                true
            }
        }
        (PatternItem::Vec(v1), PatternItem::Vec(v2)) => {
            if v1.len() != v2.len() {
                return false;
            }
            compare_patterns_with_binding_map(v1, v2, binding_map)
        }
        (PatternItem::Any, PatternItem::Any) => true,
        _ => false, // Different pattern types
    }
}

fn compare_entity_patterns_with_binding_map(
    entity1: &EntityPatternValue,
    entity2: &EntityPatternValue,
    binding_map: &mut HashMap<String, String>,
) -> bool {
    match (entity1, entity2) {
        (EntityPatternValue::EntityId(id1), EntityPatternValue::EntityId(id2)) => id1 == id2,
        (EntityPatternValue::Binding(b1), EntityPatternValue::Binding(b2)) => {
            // Check if binding is already mapped
            if let Some(mapped_binding) = binding_map.get(b1) {
                // Use existing mapping
                mapped_binding == b2
            } else {
                // Add new mapping
                binding_map.insert(b1.clone(), b2.clone());
                true
            }
        }
        _ => false, // Different entity pattern types
    }
}

fn compare_fwd_guard_bindings(
    bindings1: &HashMap<String, Value>,
    bindings2: &HashMap<String, Value>,
    binding_map: &mut HashMap<String, String>,
) -> bool {
    // For forward guard bindings, we need to map the keys using the binding map
    // and compare the values directly
    if bindings1.len() != bindings2.len() {
        return false;
    }

    for (key1, value1) in bindings1 {
        // Check if this binding is mapped
        if let Some(mapped_key) = binding_map.get(key1) {
            // Use the mapped key
            if let Some(value2) = bindings2.get(mapped_key) {
                if value1 != value2 {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            // Find a matching key in bindings2 and add to binding map
            let mut found = false;
            for (key2, value2) in bindings2 {
                if value1 == value2 && !binding_map.values().any(|v| v == key2) {
                    binding_map.insert(key1.clone(), key2.clone());
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
    }

    true
}
