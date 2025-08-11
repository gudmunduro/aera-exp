use std::io::Write;
use std::collections::HashMap;
use std::fs;
use std::process::exit;
use itertools::Itertools;
use crate::runtime::pattern_matching::{compare_commands, compare_imdls, compare_pattern_items, state_matches_facts, PatternMatchResult};
use crate::runtime::simulation::forward::{compute_instantiate_casual_models, compute_merged_forward_backward_models, ObservedState};
use crate::runtime::utils::{all_causal_models, all_req_models};
use crate::types::{Command, EntityPatternValue, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::models::{AbductionResult, IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::PatternItem;
use super::backward::{backward_chain, create_variations_of_sub_goal};
use crate::types::runtime::{RuntimeCommand, System, SystemState};
use crate::types::value::Value;

pub fn try_to_find_expected_path(goal: &Vec<Fact<MkVal>>, system: &System) {
    let expected_path: Vec<Command> = vec![
        Command::new_values("move", "h", &vec![Value::Vec(vec![Value::Number(-18.0), Value::Number(-112.0), Value::Number(0.0), Value::Number(0.0)])]),
        Command::new_values("grab", "h", &vec![]),
        Command::new_values("move", "h", &vec![Value::Vec(vec![Value::Number(-27.0), Value::Number(-37.0), Value::Number(0.0), Value::Number(0.0)])]),
        Command::new_values("move", "h", &vec![Value::Vec(vec![Value::Number(-27.0), Value::Number(-37.0), Value::Number(0.0), Value::Number(0.0)])]),
        Command::new_values("release", "h", &vec![]),
    ];
    let expected_mk_vals = vec![
        Fact::new(MkVal {
            entity_id: EntityPatternValue::EntityId("h".to_string()),
            var_name: "position".to_string(),
            value: PatternItem::Value(Value::Vec(vec![Value::UncertainNumber(267.98901398444303, 10.0),Value::UncertainNumber(-32.32548587749615, 10.0), Value::UncertainNumber(0.0, 10.0),Value::UncertainNumber(180.0, 10.0)])),
            assumption: false,
        }, TimePatternRange::wildcard()),
        Fact::new(MkVal {
            entity_id: EntityPatternValue::EntityId("h".to_string()),
            var_name: "holding".to_string(),
            value: PatternItem::Value(Value::Vec(vec![Value::EntityId("co3".to_string())])),
            assumption: false,
        }, TimePatternRange::wildcard()),
        Fact::new(MkVal {
            entity_id: EntityPatternValue::EntityId("h".to_string()),
            var_name: "position".to_string(),
            value: PatternItem::Value(Value::Vec(vec![Value::UncertainNumber(240.0, 10.0),Value::UncertainNumber(-70.0, 10.0), Value::UncertainNumber(0.0, 10.0),Value::UncertainNumber(180.0, 10.0)])),
            assumption: false,
        }, TimePatternRange::wildcard()),
        Fact::new(MkVal {
            entity_id: EntityPatternValue::EntityId("co3".to_string()),
            var_name: "approximate_pos".to_string(),
            value: PatternItem::Value(Value::Vec(vec![Value::UncertainNumber(240.0, 10.0),Value::UncertainNumber(-70.0, 10.0), Value::UncertainNumber(0.0, 10.0),Value::UncertainNumber(180.0, 10.0)])),
            assumption: false,
        }, TimePatternRange::wildcard()),
        Fact::new(MkVal {
            entity_id: EntityPatternValue::EntityId("co3".to_string()),
            var_name: "approximate_pos".to_string(),
            value: PatternItem::Value(Value::Vec(vec![Value::UncertainNumber(240.0, 10.0),Value::UncertainNumber(-70.0, 10.0), Value::UncertainNumber(-100.0, 10.0),Value::UncertainNumber(180.0, 10.0)])),
            assumption: false,
        }, TimePatternRange::wildcard()),
    ];
    let expected_rhs_properties: Vec<String> = vec!["position".to_string(), "holding".to_string(), "position".to_string(), "approximate_pos".to_string()];
    save_models(system);
    // First, validate that we can find all expected commands through backwards chaining
    for g in goal {
        can_find_all_needed_models_in_backward_chaining(0, g, &expected_path, system);
    }
    // Create backwards chaining results
    let bwd_results = backward_chain(goal, system);
    // Validate the backwards chaining results contain expected commands
    let bwd_associated_models = validate_backwards_chaining_result(&bwd_results, &expected_path, &expected_mk_vals, system);
    // Make sure we can go though all expected commands with forward chaining, using the backwards chaining results
    can_forward_chain_through_models(0, &bwd_associated_models, &bwd_results, goal, &system.current_state, &system);
}

fn validate_backwards_chaining_result(bwd_result: &Vec<IMdl>, path: &Vec<Command>, expected_mk_vals: &Vec<Fact<MkVal>>, system: &System) -> Vec<(Command, Vec<IMdl>)> {
    let mut path_models = Vec::new();

    for (path_cmd, mk_val) in path.iter().zip(expected_mk_vals.iter()) {
        let mut associated_models = Vec::new();

        let mut bwd_cmds = Vec::new();
        for res in bwd_result {
            let mdl = res.instantiate(&HashMap::new(), system);
            let mdl_cmd = match mdl.filled_in_lhs() {
                MdlLeftValue::Command(cmd) => cmd,
                _ => {
                    log::warn!("Model in backwards chaining does not have command lhs");
                    continue;
                }
            };
            let mdl_mk_val = match mdl.filled_in_rhs() {
                MdlRightValue::MkVal(mk_val) => mk_val,
                _ => {
                    log::warn!("Model in backwards chaining does not have mk.val rhs");
                    continue;
                }
            };
            bwd_cmds.push(mdl_cmd.clone());

            // compare_commands(&mdl_cmd, path_cmd, true, false)
            if mdl_mk_val.var_name == mk_val.pattern.var_name
                && compare_pattern_items(&mdl_mk_val.entity_id.to_pattern_item(), &mk_val.pattern.entity_id.to_pattern_item(), true)
                && compare_pattern_items(&mdl_mk_val.value, &mk_val.pattern.value, true) {
                associated_models.push(res.clone());
            }
        }

        if associated_models.is_empty() {
            log::error!("No model found for {path_cmd} during backward chaining");
            log::debug!("All backward chaining rhs: ");
            for bwd in bwd_result {
                log::debug!("{}", &bwd.instantiate(&HashMap::new(), system).filled_in_rhs());
            }
            exit(1);
        }

        path_models.push((path_cmd.clone(), associated_models));
    }

    path_models
}

fn can_forward_chain_through_models(depth: usize, bwd_associated_models: &Vec<(Command, Vec<IMdl>)>, goal_requirements: &Vec<IMdl>, goal: &Vec<Fact<MkVal>>, state: &SystemState, system: &System) {
    if depth == bwd_associated_models.len() {
        log::info!("Executed all commands from path through found models");
        print_all_variables(state);
        if state_matches_facts(&state, &goal) {
            log::info!("State matches goal");
        }
        return;
    }

    let expected_command = &bwd_associated_models[depth].0.to_runtime_command(&HashMap::new()).unwrap();
    // Get all casual models that can be instantiated with forward chaining
    let fwd_chained_casual_models = compute_instantiate_casual_models(state, system);
    let (insatiable_casual_models, final_casual_models)
        = compute_merged_forward_backward_models(&fwd_chained_casual_models, goal_requirements, system);

    let associated_models = &bwd_associated_models[depth].1;
    let associated_available_models = final_casual_models
        .iter()
        .filter(|m| associated_models.iter().any(|am| m.model.model_id == am.model_id))
        .collect_vec();

    let mut found_path_at_depth = false;
    let mut commands = Vec::new();
    for am in &associated_available_models {
        let is_grab_model = match &am.model {
            Mdl {
                left: Fact { pattern: MdlLeftValue::Command(cmd), .. },
                right: Fact { pattern: MdlRightValue::MkVal(mk_val), .. },
                ..
            } if cmd.name == "grab" && mk_val.var_name == "holding" => true,
            _ => false
        };
        let (is_other_grab_model, var_name) = match &am.model {
            Mdl {
                left: Fact { pattern: MdlLeftValue::Command(cmd), .. },
                right: Fact { pattern: MdlRightValue::MkVal(mk_val), .. },
                ..
            } if cmd.name == "grab" => (true, mk_val.var_name.clone()),
            _ => (false, String::new())
        };

        let Some(command) = am
            .get_casual_model_command(&insatiable_casual_models, &system)
            .map(|c| c.to_runtime_command(&am.bindings).ok())
            .flatten() else {
            continue
        };
        commands.push(command.clone());
        let Some(next_state) = am.predict_state_change(
            &state,
            &fwd_chained_casual_models.iter().filter(|(_, anti)| *anti).map(|(imdl, _)| imdl).collect(),
            &insatiable_casual_models,
            system,
        ) else {
            continue;
        };
        if is_grab_model {
            log::debug!("Forward chained through grab model\nCurrent state:");
            print_all_variables(&next_state);
        }
        if is_other_grab_model {
            log::debug!("Forward chained through other grab (grab -> {var_name}) model\nCurrent state:");
            print_all_variables(&next_state);
        }

        // Running did not change state
        if state == &next_state {
            log::warn!("Model {} did not change state", am.imdl_for_model());
            continue;
        }

        if &command == expected_command {
            found_path_at_depth = true;
            can_forward_chain_through_models(depth + 1, bwd_associated_models, goal_requirements, goal, &next_state, system);
        }
    }


    if !found_path_at_depth {
        log::info!("Expected command not found at level {depth}");
        log::info!("Expected command {expected_command}");
        log::debug!("All command options:");
        for cmd in commands {
            log::debug!("{cmd}");
        }
        log::debug!("All command targets:");
        for mdl in &associated_available_models {
            log::debug!("{}", mdl.filled_in_rhs());
        }
        log::debug!("Associated models");
        for mdl in &associated_available_models {
            log::debug!("{}", mdl.imdl_for_model());
        }
        log::debug!("Current state");
        print_all_variables(state);
    }
}

fn can_find_all_needed_models_in_backward_chaining(
    depth: usize,
    goal: &Fact<MkVal>,
    path: &Vec<Command>,
    system: &System,
) {
    if depth == path.len() {
        //log::info!("Backward chained through all expected commands!");
        return;
    }

    // Find models that match goal on RHS and have matching command on LHS
    let casual_models = all_causal_models(system);
    let expected_command = &path[path.len() - 1 - depth];
    let mut matched_models = Vec::new();

    // Abduce trough all casual models with the goal, and find those where lhs (filled in) matches the goal
    // If none is found, print all models that matches rhs (through abduce) and print which command we were trying to find
    // Otherwise if at leat one is found, abduce the requirement model as well (the result of abduce on casual models) to find the subgoal. Do that for each of the matching models and recursively call can_find_all_needed_models_in_backward_chaining with the results

    let goal_rhs = Fact::new(MdlRightValue::MkVal(goal.pattern.clone()), TimePatternRange::wildcard());

    for model in &casual_models {
        if let Some(AbductionResult::IMdl(imdl)) = model.as_bound_model().abduce(&goal_rhs, system) {
            if let MdlLeftValue::Command(cmd) = imdl.instantiate(&HashMap::new(), system).filled_in_lhs() {
                if compare_commands(&cmd, expected_command, true, false) {
                    matched_models.push(imdl);
                }
            }
        }
    }

    // No matching models found - print debug info
    if matched_models.is_empty() {
        log::info!("No models found matching expected command {expected_command} at depth {depth}");
        log::debug!("Models matching goal on RHS:");
        for model in &casual_models {
            if let PatternMatchResult::True(_) = model.right.pattern.matches(&HashMap::new(), &goal_rhs.pattern) {
                log::debug!("{model}");
            }
        }
        return;
    }

    // Found matches - recursively check subgoals
    for req_model in all_req_models(system) {
        for cas_model in &matched_models {
            let imdl_rhs = Fact::new(MdlRightValue::IMdl(cas_model.clone()), TimePatternRange::wildcard());
            if let Some(AbductionResult::SubGoal(sub_goal, sub_goal_cst_id, _, _)) = req_model.as_bound_model().abduce(&imdl_rhs, system) {
                let is_grab_model = match cas_model.get_model(system) {
                    Mdl {
                        left: Fact { pattern: MdlLeftValue::Command(cmd), .. },
                        right: Fact { pattern: MdlRightValue::MkVal(mk_val), .. },
                        ..
                    } if cmd.name == "grab" && mk_val.var_name == "holding" => true,
                    _ => false
                };
                let is_release_model = match cas_model.get_model(system) {
                    Mdl {
                        left: Fact { pattern: MdlLeftValue::Command(cmd), .. },
                        right: Fact { pattern: MdlRightValue::MkVal(mk_val), .. },
                        ..
                    } if cmd.name == "release" && mk_val.var_name == "approximate_pos" => true,
                    _ => false
                };

                let sub_goal_entities = sub_goal_cst_id
                    .map(|cst_id| &system.csts.get(&cst_id).unwrap().entities);
                let mut all_sub_goals = create_variations_of_sub_goal(&sub_goal, sub_goal_entities, system);
                all_sub_goals.insert(0, sub_goal);
                // Only show subgoal of cmd grab -> holding model
                for sub_goal in all_sub_goals {
                    if is_grab_model {
                        log::debug!("Grab model subgoal:\n{}", sub_goal.iter().map(|f| f.to_string()).join("\n"));
                    }
                    if is_release_model {
                        log::debug!("Release model subgoal:\n{}", sub_goal.iter().map(|f| f.to_string()).join("\n"));
                    }
                    for fact in sub_goal {
                        can_find_all_needed_models_in_backward_chaining(
                            depth + 1,
                            &fact,
                            path,
                            system,
                        );
                    }
                }
            }
        }
    }
}

fn print_all_variables(state: &SystemState) {
    for (key, value) in &state.variables {
        let entity = &key.entity_id;
        let variable = &key.var_name;
        log::debug!("(mk.val {entity} {variable} {value})");
    }
}

fn save_models(system: &System) {
    let req_model = all_req_models(system);

    let mut output = fs::File::create("models2.replicode").unwrap();
    for req_mdl in &req_model {
        if let MdlLeftValue::ICst(icst) = &req_mdl.left.pattern {
            let cst = &system.csts[&icst.cst_id];
            writeln!(&mut output, "{cst}").unwrap();
        }
        if let MdlRightValue::IMdl(c) = &req_mdl.right.pattern {
            let c_mdl = &system.models[&c.model_id];
            writeln!(&mut output, "{c_mdl}").unwrap();
        }
        writeln!(&mut output, "{req_mdl}").unwrap();
    }
}