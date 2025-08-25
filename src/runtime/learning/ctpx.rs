use crate::runtime::learning::utils::{change_intersects_entity_var, change_intersects_fact, compute_vec_norm, create_bindings_for_value, create_pattern_for_value, create_pattern_for_values, generate_casual_model_name, generate_cst_name, generate_req_model_name, EntityVarChange, PatternValueMap, ValueKey};
use crate::types::cst::{Cst, ICst};
use crate::types::functions::Function;
use crate::types::models::{BoundModel, IMdl, Mdl, MdlLeftValue, MdlRightValue};
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
use crate::runtime::learning::cst::form_new_cst_for_state;
use crate::runtime::learning::model_comparison::compare_model_effects;
use crate::runtime::utils::all_req_models;

pub fn extract_patterns(
    changed_var: &EntityVariableKey,
    before: Option<&Value>,
    after: &Value,
    executed_command: &RuntimeCommand,
    system: &mut System,
    state_before: &SystemState,
) {
    // Before here is like target in AERA, and after is like consequent

    let change = EntityVarChange {
        entity: changed_var.clone(),
        before: before.cloned(),
        after: after.clone(),
    };
    let mut pattern_value_map = create_initial_pattern_value_map(&change, executed_command);
    let cst = form_new_cst_for_state(&change, system, state_before, &mut pattern_value_map);
    let cmd_model =
        form_new_command_model(executed_command, &change, &mut pattern_value_map, system);
    let req_model = form_new_req_model(
        &system.csts[&cst].clone(),
        &system.models[&cmd_model].clone(),
        system,
    );
    println!("{}", system.csts[&cst]);
    println!("{}", system.models[&cmd_model]);
    println!("{}", system.models[&req_model]);
    println!("Learned new models");

    let cst = system.csts[&cst].clone();
    let cmd_model = system.models[&cmd_model].clone();
    let req_model = system.models[&req_model].clone();
    check_and_merge_with_existing_model(&cst, &req_model, &cmd_model, system);
}

fn find_existing_cst(change: &EntityVarChange, system: &System) -> Option<String> {
    system
        .csts
        .iter()
        .filter(|(_, cst)| cst.facts.iter().any(|f| change_intersects_fact(change, f)))
        .map(|(cst_id, _)| cst_id.clone())
        .next()
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
                PatternItem::Binding(b.clone())
            })
            .collect(),
        fwd_guard_bindings: Default::default(),
    });

    let model_id = generate_req_model_name(system);
    let model = Mdl {
        model_id: model_id.clone(),
        left: Fact::new(lhs, TimePatternRange::wildcard()),
        right: Fact::new(rhs, TimePatternRange::wildcard()),
        failure_count: 0,
        success_count: 1,
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
    let (fwd_guards, bwd_guards) = create_delta_guards(pattern_value_map, change);

    // Previously consequent bindings were removed here

    if cmd.name == "grab" && change.entity.var_name == "holding" {
        log::debug!("Grab command / holding");
    }

    let rhs = MdlRightValue::MkVal(MkVal {
        entity_id: EntityPatternValue::Binding("PE".to_string()),
        var_name: change.entity.var_name.clone(),
        value: create_pattern_for_value(&change.after, pattern_value_map, false),
        assumption: false,
    });

    let model_id = generate_casual_model_name(system);
    let model = Mdl {
        model_id: model_id.clone(),
        left: Fact::new(lhs, TimePatternRange::wildcard()),
        right: Fact::new(rhs, TimePatternRange::wildcard()),
        forward_computed: fwd_guards,
        backward_computed: bwd_guards,
        failure_count: 0,
        success_count: 1,
    };
    system.models.insert(model_id.clone(), model);

    model_id
}

fn create_delta_guards(pattern_value_map: &PatternValueMap, change: &EntityVarChange) -> (Vec<(String, Function)>, Vec<(String, Function)>) {
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
            b.starts_with("P") && matches!(&v.0, Value::Number(_) | Value::UncertainNumber(_, _))
        })
        .collect();
    let cmd_values: HashMap<_, _> = pattern_value_map
        .iter()
        .filter(|(v, b)| {
            b.starts_with("CMD") && matches!(&v.0, Value::Number(_) | Value::UncertainNumber(_, _))
        })
        .collect();

    let (fwd_guards, bwd_guards) = non_matching_c_values
        .iter()
        .filter_map(|(v, b)| {
            let premise_binding = if let Some(before) = &change.before {
                find_equivalent_binding_in_other(v, &change.after, &before, pattern_value_map)
            } else {
                None
            };

            create_delta_guard(&v.0, b, &premise_binding, &premise_values, &cmd_values)
        })
        .unzip();
    (fwd_guards, bwd_guards)
}

fn create_delta_guard(
    value: &Value,
    binding: &str,
    premise_equivalent: &Option<(String, Value)>,
    premise_values: &HashMap<&ValueKey, &String>,
    cmd_values: &HashMap<&ValueKey, &String>,
) -> Option<((String, Function), (String, Function))> {
    let addition = premise_values
        .iter()
        .filter_map(|(ValueKey(pv), pb)| {
            cmd_values
                .iter()
                .filter_map(|(ValueKey(cv), cb)| {
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
    } else if let Some((pb, pv)) = premise_equivalent {
        if value.can_do_numeric_op(pv) {
            let diff = value.clone() - pv.clone();
            let fwd_function = Function::Add(
                Box::new(Function::Value(PatternItem::Binding(
                    pb.to_string(),
                ))),
                Box::new(Function::Value(PatternItem::Value(
                    diff.clone()
                ))),
            );
            let bwd_function = Function::Sub(
                Box::new(Function::Value(PatternItem::Binding(
                    binding.to_string(),
                ))),
                Box::new(Function::Value(PatternItem::Value(
                    diff
                ))),
            );

            Some(((binding.to_string(), fwd_function), (pb.to_string(), bwd_function.clone())))
        }
        else {
            None
        }
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
) -> PatternValueMap {
    let mut map = HashMap::new();
    map.insert(
        ValueKey(Value::EntityId(change.entity.entity_id.clone())),
        "PE".to_string(),
    );
    if executed_command.entity_id != change.entity.entity_id {
        map.insert(
            ValueKey(Value::EntityId(executed_command.entity_id.clone())),
            "CMD_E".to_string(),
        );
    }
    // Only add premise bindings if there is a premise (if the fact is not new)
    if let Some(before) = &change.before {
        create_bindings_for_value(&before, &mut map, "P", &mut 0);
    }
    let mut cmd_binding_index = 0;
    for v in &executed_command.params {
        create_bindings_for_value(v, &mut map, "CMD", &mut cmd_binding_index);
    }
    // Create this last so we know if any value in consequent did not match any other by checking if there is a C binding in the binding map
    create_bindings_for_value(&change.after, &mut map, "C", &mut 0);

    map
}

// Check if the newly formed model triplet is the same as an existing one, except for only conditions in the CST
// and if it is, then merge it into the prior model (by removing unnecessary conditions)
fn check_and_merge_with_existing_model(cst: &Cst, req_model: &Mdl, casual_model: &Mdl, system: &mut System) {
    // Start by comparing casual model, are patterns the same in lhs, rhs and guards
    // Check if imdl pattern in req_model is the same
    // Find variables used in imdl pattern, check if those specific variables are the same in CSTs
    // Create a new CST, with only the facts that appear in both
    // Recreate icst in req model to fit the new cst

    let req_models = all_req_models(system);
    let Some((new_cst, new_req_model, new_casual_model)) = req_models.into_iter()
        .filter_map(|req_model2| {
            match &req_model2.right.pattern {
                MdlRightValue::IMdl(imdl) if req_model2.model_id != req_model.model_id => {
                    let casual_model = system.models.get(&imdl.model_id)?.clone();
                    Some((req_model2, casual_model))
                }
                _ => None
            }
        })
        .filter(|(req_model2, casual_model2)| quick_compare_models(req_model, casual_model, req_model2, casual_model2))
        .map(|(req_model, casual_model)| {
            log::debug!("Found quick match to merge {}", req_model.model_id);
            let cst_id = &req_model.left.pattern.as_icst().cst_id;
            let cst = system.csts.get(cst_id).expect("Cst from req model missing");
            (cst.clone(), req_model, casual_model)
        })
        .find_map(|(cst2, req_model2, casual_model2)| {
            let new_cst = compare_model_effects(&cst2, &req_model2, &casual_model2, cst, req_model, casual_model, system)?;
            Some((new_cst, req_model2, casual_model2))
        }) else {
        return;
    };

    let new_cst_id = new_cst.cst_id.clone();
    system.csts.insert(new_cst_id.clone(), new_cst);
    
    // Since the model would have succeeded if it had already been merged, promote it
    system.models.get_mut(&new_casual_model.model_id).unwrap().promote();
    
    let new_cst = system.csts.get(&new_cst_id).unwrap();
    let new_req_model_ref = system.models.get_mut(&new_req_model.model_id).unwrap();
    new_req_model_ref.left = new_req_model_ref.left.with_pattern(MdlLeftValue::ICst(ICst {
      cst_id: new_cst.cst_id.clone(),
        params: new_cst.binding_params()
            .iter()
            .map(|b| PatternItem::Binding(b.clone()))
            .collect(),
    }));

    println!("Merged into existing model");
    println!("{new_cst}");
    println!("{new_req_model_ref}");

    system.csts.remove(&cst.cst_id);
    system.models.remove(&req_model.model_id);
    system.models.remove(&casual_model.model_id);
}

fn quick_compare_models(req_model1: &Mdl, casual_model1: &Mdl, req_model2: &Mdl, casual_model2: &Mdl) -> bool {
    let lhs_cmd_matches = matches!((&casual_model1.left.pattern, &casual_model2.left.pattern), (MdlLeftValue::Command(cmd1), MdlLeftValue::Command(cmd2)) if cmd1.name == cmd2.name);
    let rhs_var_matches = matches!((&casual_model1.right.pattern, &casual_model2.right.pattern), (MdlRightValue::MkVal(mk_val1), MdlRightValue::MkVal(mk_val2)) if mk_val1.var_name == mk_val2.var_name);
    let imdl_param_count_matches = matches!((&req_model1.right.pattern, &req_model2.right.pattern), (MdlRightValue::IMdl(imdl1), MdlRightValue::IMdl(imdl2)) if imdl1.params.len() == imdl2.params.len());

    lhs_cmd_matches && rhs_var_matches && imdl_param_count_matches
}

fn find_equivalent_binding_in_other(value_key: &ValueKey, value_of_key: &Value, other_value: &Value, pattern_value_map: &PatternValueMap) -> Option<(String, Value)> {
    if value_of_key == &value_key.0 {
        return Some((pattern_value_map.get(&ValueKey(other_value.clone())).cloned()?, other_value.clone()));
    }

    if let (Value::Vec(v1), Value::Vec(v2)) = (value_of_key, other_value) {
        for (item1, item2) in v1.iter().zip(v2.iter()) {
            if let Some(result) = find_equivalent_binding_in_other(value_key, item1, item2, pattern_value_map) {
                return Some(result);
            }
        }
    }

    None
}
