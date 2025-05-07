use itertools::Itertools;
use crate::runtime::learning::utils::{change_intersects_entity_var, create_pattern_for_value, generate_cst_name, EntityVarChange, PatternValueMap};
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
        .collect_vec();
    // Always have the premise first in the CST
    matching_entity_vars.insert(0, (&change.entity, &change.before, true));
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
    let mut facts = Vec::new();
    for (key, value, is_premise) in entity_vars {
        let value: PatternItem = create_pattern_for_value(value, pattern_value_map);

        let entity_var = Value::EntityId(key.entity_id.clone());
        let entity_binding: String = match create_pattern_for_value(&entity_var, pattern_value_map)
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
        .filter_map(|(v, b)| match v {
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