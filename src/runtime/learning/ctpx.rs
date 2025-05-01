use crate::runtime::learning::utils::{
    change_intersects_entity_var, change_intersects_fact, compute_vec_norm,
    generate_casual_model_name, generate_cst_name, generate_req_model_name, EntityVarChange,
};
use crate::types::cst::{Cst, ICst};
use crate::types::functions::Function;
use crate::types::models::{IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::PatternItem;
use crate::types::runtime::{RuntimeCommand, System, SystemState};
use crate::types::value::Value;
use crate::types::{
    Command, EntityDeclaration, EntityPatternValue, EntityVariableKey, Fact, MkVal,
    TimePatternRange,
};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::vec;

pub type PatternValueMap = HashMap<Value, String>;

pub fn extract_patterns(
    changed_var: &EntityVariableKey,
    before: &Value,
    after: &Value,
    executed_command: &RuntimeCommand,
    system: &mut System,
    state_before: &SystemState,
) {
    // Before here is like target in AERA, and after is like consequent

    let change = EntityVarChange {
        entity: changed_var.clone(),
        before: before.clone(),
        after: after.clone(),
    };
    let mut pattern_value_map = create_initial_pattern_value_map(&change, executed_command);
    let cst = form_new_cst(&change, system, state_before, &mut pattern_value_map);
    let cmd_model =
        form_new_command_model(executed_command, &change, &mut pattern_value_map, system);
    let req_model = form_new_req_model(
        &system.csts[&cst].clone(),
        &system.models[&cmd_model].clone(),
        system,
    );
    println!("{:#?}", system.csts[&cst]);
    println!("{:#?}", system.models[&cmd_model]);
    println!("{:#?}", system.models[&req_model]);
    println!("Learned new models");
}

fn find_existing_cst(change: &EntityVarChange, system: &System) -> Option<String> {
    system
        .csts
        .iter()
        .filter(|(_, cst)| cst.facts.iter().any(|f| change_intersects_fact(change, f)))
        .map(|(cst_id, _)| cst_id.clone())
        .next()
}

// Creates a new CST for facts that appear to be related from the change
fn form_new_cst(
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
        change,
        pattern_value_map,
        system,
    );
    system.csts.insert(name.to_string(), cst);
    name
}

fn form_new_cst_from_entity_vars(
    cst_id: String,
    entity_vars: &Vec<(&EntityVariableKey, &Value, bool)>,
    change: &EntityVarChange,
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

fn form_new_req_model(cst: &Cst, command_model: &Mdl, system: &mut System) -> String {
    let cst_binding_params = cst.binding_params();
    let lhs = MdlLeftValue::ICst(ICst {
        cst_id: cst.cst_id.clone(),
        params: cst_binding_params
            .iter()
            .map(|b| PatternItem::Binding(b.clone()))
            .collect(),
    });
    let rhs = MdlRightValue::IMdl(IMdl {
        model_id: command_model.model_id.clone(),
        params: command_model
            .binding_param()
            .iter()
            .map(|b| {
                if b.starts_with("P") || b.starts_with("C") {
                    PatternItem::Binding(b.clone())
                } else {
                    PatternItem::Any
                }
            })
            .collect(),
        fwd_guard_bindings: Default::default(),
    });

    let model_id = generate_req_model_name(system);
    let model = Mdl {
        model_id: model_id.clone(),
        left: Fact::new(lhs, TimePatternRange::wildcard()),
        right: Fact::new(rhs, TimePatternRange::wildcard()),
        confidence: 0.5,
        forward_computed: vec![],
        backward_computed: vec![],
    };
    system.models.insert(model_id.clone(), model);

    model_id.clone()
}

fn form_new_command_model(
    cmd: &RuntimeCommand,
    change: &EntityVarChange,
    pattern_value_map: &mut PatternValueMap,
    system: &mut System,
) -> String {
    let cmd_entity_binding = if cmd.entity_id == change.entity.entity_id {
        "PE".to_string()
    } else {
        "CMD_E".to_string()
    };
    let lhs = MdlLeftValue::Command(Command {
        name: cmd.name.to_string(),
        entity_id: EntityPatternValue::Binding(cmd_entity_binding),
        params: create_pattern_for_values(&cmd.params, pattern_value_map),
    });
    let (fwd_guards, bwd_guards) = create_delta_guards(pattern_value_map);
    let rhs = MdlRightValue::MkVal(MkVal {
        entity_id: EntityPatternValue::Binding("PE".to_string()),
        var_name: change.entity.var_name.clone(),
        value: create_pattern_for_value(&change.after, pattern_value_map),
        assumption: false,
    });

    let model_id = generate_casual_model_name(system);
    let model = Mdl {
        model_id: model_id.clone(),
        left: Fact::new(lhs, TimePatternRange::wildcard()),
        right: Fact::new(rhs, TimePatternRange::wildcard()),
        forward_computed: fwd_guards,
        backward_computed: bwd_guards,
        confidence: 0.5,
    };
    system.models.insert(model_id.clone(), model);

    model_id
}

fn create_delta_guards(pattern_value_map: &PatternValueMap) -> (Vec<(String, Function)>, Vec<(String, Function)>) {
    // If there are no C binding, then every value in the consequent matched another either in the premise or the command
    // so there is no need to create a guard
    let non_matching_c_values = pattern_value_map
        .iter()
        .filter(|(_, b)| b.starts_with("C") && !b.starts_with("CMD"))
        .collect_vec();
    if non_matching_c_values.is_empty() {
        return (Vec::new(), Vec::new());
    }
    // TODO: Get real bindings for premise and cmd, P{X} and CMD{X} are only used if the values are unique
    // TODO: Could also combine them and look through any combination?
    let premise_values: HashMap<_, _> = pattern_value_map
        .iter()
        .filter(|(v, b)| {
            b.starts_with("P") && matches!(v, Value::Number(_) | Value::UncertainNumber(_, _))
        })
        .collect();
    let cmd_values: HashMap<_, _> = pattern_value_map
        .iter()
        .filter(|(v, b)| {
            b.starts_with("CMD") && matches!(v, Value::Number(_) | Value::UncertainNumber(_, _))
        })
        .collect();

    let (fwd_guards, bwd_guards) = non_matching_c_values
        .iter()
        .filter_map(|(v, b)| create_delta_guard(v, b, &premise_values, &cmd_values))
        .unzip();
    (fwd_guards, bwd_guards)
}

fn create_delta_guard(
    value: &Value,
    binding: &str,
    premise_values: &HashMap<&Value, &String>,
    cmd_values: &HashMap<&Value, &String>,
) -> Option<((String, Function), (String, Function))> {
    let addition = premise_values
        .iter()
        .filter_map(|(pv, pb)| {
            cmd_values
                .iter()
                .filter_map(|(cv, cb)| {
                    if (*pv).clone() + (*cv).clone() == *value {
                        Some((pb, cb))
                    } else {
                        None
                    }
                })
                .next()
        })
        .next();
    // Case 1: P{X} + CMD{X}
    if let Some((premise_binding, cmd_binding)) = addition {
        let fwd_function = Function::Add(
            Box::new(Function::Value(PatternItem::Binding(
                premise_binding.to_string(),
            ))),
            Box::new(Function::Value(PatternItem::Binding(
                cmd_binding.to_string(),
            ))),
        );
        let bwd_function = Function::Sub(
            Box::new(Function::Value(PatternItem::Binding(
                binding.to_string(),
            ))),
            Box::new(Function::Value(PatternItem::Binding(
                premise_binding.to_string(),
            ))),
        );

        Some(((binding.to_string(), fwd_function), (cmd_binding.to_string(), bwd_function.clone())))
    } else {
        None
    }
}

fn compare_values_for_guard(value1: &Value, value2: &Value) -> bool {
    if value1.can_do_numeric_op(value2) {
        let cmp_res = value1.clone() - value2.clone();
        // TODO: Dynamic threshold for comparison
        match cmp_res {
            Value::Number(n) | Value::UncertainNumber(n, _) => (n).abs() < 0.5,
            Value::Vec(v) => compute_vec_norm(&v) < 1.0,
            _ => panic!("Got non-numeric value from math operation"),
        }
    } else {
        value1 == value2
    }
}

fn create_initial_pattern_value_map(
    change: &EntityVarChange,
    executed_command: &RuntimeCommand,
) -> HashMap<Value, String> {
    let mut map = HashMap::new();
    map.insert(
        Value::EntityId(change.entity.entity_id.clone()),
        "PE".to_string(),
    );
    if executed_command.entity_id != change.entity.entity_id {
        map.insert(
            Value::EntityId(executed_command.entity_id.clone()),
            "CMD_E".to_string(),
        );
    }
    create_bindings_for_value(&change.before, &mut map, "P", &mut 0);
    let mut cmd_binding_index = 0;
    for v in &executed_command.params {
        create_bindings_for_value(v, &mut map, "CMD", &mut cmd_binding_index);
    }
    // Create this last so we know if any value in consequent did not match any other by checking if there is a C value
    create_bindings_for_value(&change.after, &mut map, "C", &mut 0);

    map
}

fn create_pattern_for_values(
    values: &Vec<Value>,
    pattern_value_map: &mut HashMap<Value, String>,
) -> Vec<PatternItem> {
    values
        .iter()
        .map(|v| create_pattern_for_value(v, pattern_value_map))
        .collect()
}

// TODO: Boolean vectors will get special treatment here, will be learned as boolean values with rest ignored
fn create_pattern_for_value(
    value: &Value,
    pattern_value_map: &mut HashMap<Value, String>,
) -> PatternItem {
    match value {
        Value::Number(_) | Value::UncertainNumber(_, _) | Value::String(_) | Value::EntityId(_) => {
            if !pattern_value_map.contains_key(value) {
                let binding = format!("v{}", pattern_value_map.len());
                pattern_value_map.insert(value.clone(), binding.clone());
                PatternItem::Binding(binding)
            } else {
                PatternItem::Binding(pattern_value_map[value].clone())
            }
        }
        Value::Vec(vec) => PatternItem::Vec(
            vec.iter()
                .map(|v| create_pattern_for_value(v, pattern_value_map))
                .collect(),
        ),
        Value::ConstantNumber(_) => PatternItem::Value(value.clone()),
    }
}

fn create_bindings_for_value(
    value: &Value,
    pattern_value_map: &mut HashMap<Value, String>,
    binding_prefix: &str,
    start_index: &mut i32,
) {
    match value {
        Value::Number(_) | Value::UncertainNumber(_, _) | Value::String(_) | Value::EntityId(_) => {
            if !pattern_value_map.contains_key(value) {
                pattern_value_map.insert(value.clone(), format!("{binding_prefix}{start_index}"));
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
