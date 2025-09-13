use std::fs::File;
use std::io::Write;
use std::process::exit;
use crate::runtime::learning;
use crate::runtime::pattern_matching::state_matches_facts;
use crate::runtime::simulation::backward::backward_chain;
use crate::runtime::simulation::forward::{forward_chain, predict_all_changes_of_command};
use crate::runtime::simulation::sim_debugger::{save_models, try_to_find_expected_path};
use crate::runtime::utils::{compute_assumptions, compute_instantiated_states};
use crate::types::{Command, EntityPatternValue, Fact, MkVal};
use crate::types::runtime::{RuntimeCommand, System, SystemState, SystemTime};
use crate::types::value::Value;

const ENABLE_DEBUG: bool = false;

pub fn run_aera(seed: impl FnOnce(&mut System), receive_input: impl Fn(&mut System), eject_command: impl Fn(&RuntimeCommand, &mut System)) {
    let mut system = System::new();
    seed(&mut system);

    let mut last_state = system.current_state.clone();
    let mut last_executed_command = None;
    let mut last_was_babble_command = true;
    let mut predicted_changes = Vec::new();
    loop {
        let goal = system.goals.get(system.current_goal_index).cloned().unwrap_or(Vec::new());
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Update state from interface
        log::debug!("Waiting for variables");
        receive_input(&mut system);
        // Learn new csts and models, this needs to happen before instantiating csts so we can instantiate the new csts
        if let Some(cmd) = &last_executed_command {
            learning::extract_patterns(cmd, &mut system, &last_state, &predicted_changes);
        }
        system.current_state.instansiated_csts = compute_instantiated_states(&system, &system.current_state);
        system.current_state.variables.extend(compute_assumptions(&system, &system.current_state));
        system.current_state.instansiated_csts = compute_instantiated_states(&system, &system.current_state);
        last_state = system.current_state.clone();

        log::debug!("Got variables");
        print_all_variables(&system.current_state);

        log::debug!("Instantiated composite states");
        for state in system.current_state.instansiated_csts.values().flatten() {
            log::debug!("{}", state.icst_for_cst());
        }

        if !last_was_babble_command && state_matches_facts(&system.current_state, &goal) {
            log::info!("Goal achieved");
            system.current_goal_index += 1;

            if system.current_goal_index == system.goals.len() {
                log::info!("All goals achieved");
            }
        }

        let mut path = if system.babble_command.is_empty() {
            let mut res_path = Vec::new();
            for g in goal.iter() {
                // For debugging
                if ENABLE_DEBUG {
                    try_to_find_expected_path(&g, &system);
                    exit(0);
                }
                save_models(&system);

                // Perform backward chaining
                let bwd_result = backward_chain(&g, &system);
                log::debug!("Results of backward chaining");
                for (mdl, _) in &bwd_result {
                    log::debug!("{mdl}");
                }

                // Perform forward chaining
                let path = forward_chain(&goal, &bwd_result, &system);
                log::debug!("Results of forward chaining");
                log::debug!("Goal reachable: {}", !path.is_empty());
                log::debug!("{}", path.iter().map(|cmd| cmd.to_string()).collect::<Vec<String>>().join(", "));

                last_was_babble_command = false;
                if !path.is_empty() {
                    res_path = path;
                    break;
                }
            }
            
            res_path
        } else {
            let command = system.babble_command[0].clone();
            system.babble_command.remove(0);
            last_was_babble_command = true;

            // Save knowledge after performing babble commands
            /*let json = serde_json::to_string_pretty(&system.models).unwrap();
            let mut file = File::create("models.json").unwrap();
            file.write_all(json.as_bytes()).unwrap();
            drop(file);
            let json = serde_json::to_string_pretty(&system.csts).unwrap();
            let mut file = File::create("csts.json").unwrap();
            file.write_all(json.as_bytes()).unwrap();
            log::debug!("Written models and composite states");*/

            vec![command]
        };

        // Send command with interface
        if !path.is_empty() {
            eject_command(&path[0], &mut system);
            log::info!("Executed command {:?}", &path[0]);
            predicted_changes = predict_all_changes_of_command(&path[0], false, &system);
            last_executed_command = Some(path.remove(0));
        }
        else {
            log::info!("No action found with forward chaining");
            eject_command(&RuntimeCommand {
                name: "no_action".to_string(),
                entity_id: "sys".to_string(),
                params: Vec::new(),
            }, &mut system);
            predicted_changes.clear();
            last_executed_command = None;
        }

        advance_time_step(&mut system);
    }
}

fn advance_time_step(data: &mut System) {
    let SystemTime::Exact(time) = data.current_state.time else {
        panic!("System time should always be exact during runtime");
    };
    // Increment by 100ms
    data.current_state.time = SystemTime::Exact(time + 100);
}

fn print_all_variables(state: &SystemState) {
    for (key, value) in &state.variables {
        let entity = &key.entity_id;
        let variable = &key.var_name;
        log::debug!("(mk.val {entity} {variable} {value})");
    }
}