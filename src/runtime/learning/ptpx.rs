use crate::runtime::learning::utils::{change_intersects_entity_var, create_bindings_for_value, create_pattern_for_value, generate_anti_req_model_name, generate_req_model_name, EntityVarChange, PatternValueMap, ValueKey};
use crate::runtime::utils::all_req_models;
use crate::types::cst::{Cst, ICst};
use crate::types::models::{IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::runtime::{RuntimeCommand, System, SystemState};
use crate::types::value::Value;
use crate::types::{EntityVariableKey, Fact, TimePatternRange};
use itertools::Itertools;
use std::collections::HashMap;
use crate::runtime::learning::cst::form_new_cst_for_state;
use crate::types::pattern::PatternItem;

pub fn extract_patterns(
    entity_var: &EntityVariableKey,
    before: &Value,
    after: &Value,
    expected_change: &Value,
    model_at_fault: &IMdl,
    executed_command: &RuntimeCommand,
    system: &mut System,
    state_before: &SystemState,
) {
    log::debug!("Expected {entity_var:?} to become {expected_change}, but it did not change");

    let change = EntityVarChange {
        entity: entity_var.clone(),
        before: Some(before.clone()),
        // TODO: This is hack to reuse the function, makes it so that facts including the predicted value will be found (i.e. tried to grab and expected *[co1]*)
        after: expected_change.clone(),
    };
    let mut pattern_map = create_initial_pattern_value_map(entity_var, before, executed_command);
    let new_cst_id = form_new_cst_for_state(&change, system, state_before, &mut pattern_map);
    let new_cst = system.csts[&new_cst_id].clone();
    form_new_anti_req_model(&new_cst, model_at_fault, &mut pattern_map, system);

}

fn form_new_anti_req_model(cst: &Cst, failed_command_model: &IMdl, pattern_map: &mut PatternValueMap, system: &mut System) -> String {
    let cst_binding_params = cst.binding_params();
    let lhs = MdlLeftValue::ICst(ICst {
        cst_id: cst.cst_id.clone(),
        params: cst_binding_params
            .iter()
            .map(|b| PatternItem::Binding(b.clone()))
            .collect(),
    });
    let mut binding_count = pattern_map.len();
    let rhs = MdlRightValue::IMdl(IMdl {
        model_id: failed_command_model.model_id.clone(),
        params: failed_command_model
            .params
            .iter()
            .map(|value| {
                match value {
                    PatternItem::Value(v) => {
                        let limited_pattern_map = pattern_map.iter().filter(|(_,  binding)| binding.starts_with("P")).collect();
                        create_pattern_for_imdl_value(v, &limited_pattern_map, &mut binding_count)
                    }
                    PatternItem::Any => {
                        binding_count += 1;
                        PatternItem::Binding(format!("v{binding_count}"))
                    }
                    _ => {
                        panic!("Trace of executed model includes unbound variables ({value}), this should never happen");
                    }
                }
            })
            .collect(),
        fwd_guard_bindings: Default::default(),
    });

    let model_id = generate_anti_req_model_name(system);
    let model = Mdl {
        model_id: model_id.clone(),
        left: Fact::new(lhs, TimePatternRange::wildcard()),
        right: Fact::anti(rhs, TimePatternRange::wildcard()),
        confidence: 0.5,
        forward_computed: vec![],
        backward_computed: vec![],
    };
    println!("Created new anti-requirement model");
    println!("{cst}");
    println!("{model}");
    system.models.insert(model_id.clone(), model);

    model_id.clone()
}

fn create_initial_pattern_value_map(
    entity_var: &EntityVariableKey,
    before: &Value,
    executed_command: &RuntimeCommand,
) -> PatternValueMap {
    let mut map = HashMap::new();
    map.insert(
        ValueKey(Value::EntityId(entity_var.entity_id.clone())),
        "PE".to_string(),
    );
    if executed_command.entity_id != entity_var.entity_id {
        map.insert(
            ValueKey(Value::EntityId(executed_command.entity_id.clone())),
            "CMD_E".to_string(),
        );
    }
    create_bindings_for_value(&before, &mut map, "P", &mut 0);
    let mut cmd_binding_index = 0;
    for v in &executed_command.params {
        create_bindings_for_value(v, &mut map, "CMD", &mut cmd_binding_index);
    }

    map
}

fn create_pattern_for_imdl_value(
    value: &Value,
    limited_pattern_binding_map: &HashMap<&ValueKey, &String>,
    binding_count: &mut usize,
) -> PatternItem {
    match value {
        Value::Number(_) | Value::UncertainNumber(_, _) | Value::String(_) | Value::EntityId(_) => {
            if !limited_pattern_binding_map.contains_key(&ValueKey(value.clone())) {
                *binding_count += 1;
                PatternItem::Binding(format!("v{binding_count}"))
            } else {
                PatternItem::Binding(limited_pattern_binding_map[&ValueKey(value.clone())].clone())
            }
        }
        Value::Vec(vec) => PatternItem::Vec(
            vec.iter()
                .map(|v| create_pattern_for_imdl_value(v, limited_pattern_binding_map, binding_count))
                .collect(),
        ),
        Value::ConstantNumber(_) => PatternItem::Value(value.clone()),
    }
}