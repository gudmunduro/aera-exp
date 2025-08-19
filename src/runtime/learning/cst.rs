use std::collections::HashMap;
use itertools::Itertools;
use crate::runtime::learning::utils::{change_intersects_entity_var, create_pattern_for_value, generate_cst_name, EntityVarChange, PatternValueMap, ValueKey};
use crate::types::cst::Cst;
use crate::types::{EntityDeclaration, EntityPatternValue, EntityVariableKey, Fact, MkVal, TimePatternRange};
use crate::types::pattern::PatternItem;
use crate::types::runtime::{System, SystemState};
use crate::types::value::Value;

// Creates a new CST for facts that appear to be related from the change
pub fn form_new_cst_for_state(
    change: &EntityVarChange,
    system: &mut System,
    state_before: &SystemState,
    pattern_value_map: &mut PatternValueMap,
) -> String {
    let mut matching_entity_vars = state_before
        .variables
        .iter()
        .filter(|(key, _)| *key != &change.entity)
        .filter(|(key, value)| change_intersects_entity_var(change, (key, value)))
        .map(|(key, value)| (key, value, false))
        .sorted_by_key(|(e, _, _)| {
            // TODO: May have to do the same for CMD_E
            // Have facts with premise entity at the top
            if e.entity_id == change.entity.entity_id {
                0
            }
            else {
                1
            }
        })
        .collect_vec();
    // Always have the premise first in the CST
    if let Some(before) = &change.before {
        matching_entity_vars.insert(0, (&change.entity, before, true));
    }
    let name = generate_cst_name(system);
    let cst = form_new_cst_from_entity_vars(
        name.to_string(),
        &matching_entity_vars,
        pattern_value_map,
        system,
    );
    system.csts.insert(name.to_string(), cst);
    name
}

fn form_new_cst_from_entity_vars(
    cst_id: String,
    entity_vars: &Vec<(&EntityVariableKey, &Value, bool)>,
    pattern_value_map: &mut PatternValueMap,
    system: &System,
) -> Cst {
    let mut entities_for_class: HashMap<String, String> = HashMap::new();
    let mut facts = Vec::new();
    for (key, value, is_premise) in entity_vars {
        let entity_class = system.find_class_of_entity(&key.entity_id).unwrap();
        if let Some(class_entity) = entities_for_class.get(&entity_class) {
            if class_entity != &key.entity_id {
                // Don't allow more than one entity of the same class
                continue
            }
        }
        else {
            entities_for_class.insert(entity_class, key.entity_id.to_string());
        }

        let other_entity_classes = get_entity_vars_for_value(value)
            .into_iter()
            .filter_map(|e| system.find_class_of_entity(&key.entity_id).map(|c| (e, c)))
            .collect_vec();
        for (entity_id, class) in other_entity_classes {
            if !entities_for_class.contains_key(&class) {
                entities_for_class.insert(class, entity_id);
            }
        }

        let value: PatternItem = create_pattern_for_value(value, pattern_value_map, false);

        let entity_var = Value::EntityId(key.entity_id.clone());
        let entity_binding: String = match create_pattern_for_value(&entity_var, pattern_value_map, false)
        {
            PatternItem::Binding(b) => b,
            _ => panic!("Invalid pattern created for entity value"),
        };
        facts.push(Fact::new(
            MkVal {
                entity_id: EntityPatternValue::Binding(entity_binding.clone()),
                var_name: key.var_name.clone(),
                value,
                assumption: false,
            },
            TimePatternRange::wildcard(),
        ));
    }

    let entities = pattern_value_map
        .iter()
        .filter_map(|(ValueKey(v), b)| match v {
            Value::EntityId(e) => Some(EntityDeclaration::new(b, &system.find_class_of_entity(e).unwrap())),
            _ => None
        })
        .collect_vec();

    Cst {
        cst_id,
        facts,
        entities,
    }
}

fn get_entity_vars_for_value(value: &Value) -> Vec<String> {
    match value {
        Value::Number(_) => Vec::new(),
        Value::ConstantNumber(_) => Vec::new(),
        Value::UncertainNumber(_, _) => Vec::new(),
        Value::String(_) => Vec::new(),
        Value::Vec(vec_values) => {
            vec_values.iter()
                .flat_map(|v| get_entity_vars_for_value(v))
                .collect()
        },
        Value::EntityId(entity_id) => vec![entity_id.clone()],
    }

}