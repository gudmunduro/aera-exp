
pub mod pattern_matching;
mod simulation;
mod seeds;

use std::collections::HashSet;
use std::rc::Rc;
use itertools::Itertools;
use crate::interfaces::tcp_interface::TcpInterface;
use crate::types::runtime::{RuntimeCommand, System, SystemTime};
use crate::runtime::pattern_matching::{compute_instantiated_states};
use crate::runtime::simulation::backward::backward_chain;
use crate::runtime::simulation::forward::{forward_chain, ForwardChainNode};
use crate::types::{EntityPatternValue, EntityVariableKey, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::pattern::{PatternItem};
use crate::types::value::Value;

pub fn run_demo() {
    let mut system = System::new();
    seeds::setup_bindings_seed(&mut system);
    system.current_state.instansiated_csts = compute_instantiated_states(&system, &system.current_state);

    log::debug!("Instantiated composite states");
    for state in system.current_state.instansiated_csts.values().flatten() {
        log::debug!("State: {}", state.cst.cst_id);
    }

    let goal = vec![
        Fact {
            pattern: MkVal {
                entity_id: EntityPatternValue::EntityId("o".to_string()),
                var_name: "pos".to_string(),
                value: PatternItem::Value(Value::List(vec![Value::UncertainNumber(5.0, 0.1), Value::UncertainNumber(7.0, 0.1)])),
            },
            time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
        },
    ];

    let bwd_result = backward_chain(&goal, &system);
    log::debug!("Results of backward chaining");
    for mdl in &bwd_result {
        log::debug!("{mdl}");
    }

    let fwd_result = forward_chain(&goal, &bwd_result, &system);
    log::debug!("Results of forward chaining");
    log::debug!("Goal reachable: {}", !fwd_result.is_empty());
    log::debug!("{fwd_result:?}");

    advance_time_step(&mut system);
}

#[allow(unused)]
pub fn run_with_tcp() {
    let mut tcp_interface = TcpInterface::connect().expect("Failed to connect to controller with TCP");
    let mut system = System::new();
    seeds::robot_advanced_move::setup_robot_advanced_seed(&mut system);

    let mut goals = vec![
        vec![
            Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::EntityId("co1".to_string()),
                    var_name: "approximate_pos".to_string(),
                    value: PatternItem::Value(Value::List(vec![Value::UncertainNumber(20.0, 0.1), Value::UncertainNumber(140.0, 0.1), Value::UncertainNumber(0.0, 0.1), Value::UncertainNumber(45.0, 0.1)])),
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            },
        ],
    ];

    let mut goal = goals[0].clone();
    let mut committed_path: Option<Vec<RuntimeCommand>> = None;

    loop {
        advance_time_step(&mut system);

        // Update state from TCP
        log::debug!("Waiting for variables");
        let tcp_variables = tcp_interface.update_variables();
        system.current_state.variables.extend(tcp_variables);
        system.current_state.instansiated_csts = compute_instantiated_states(&system, &system.current_state);

        log::debug!("Instantiated composite states");
        for state in system.current_state.instansiated_csts.values().flatten() {
            log::debug!("State: {}", state.cst.cst_id);
        }

        let path = if let Some(committed) = committed_path.as_ref() {
            Some(committed.clone())
        } else {
            // Perform backward chaining
            let bwd_result = backward_chain(&goal, &system);
            log::debug!("Results of backward chaining");
            for mdl in &bwd_result {
                log::debug!("{mdl}");
            }

            log::debug!("co1 pos");
            log::debug!("{:?}", system.current_state.variables[&EntityVariableKey::new("co1", "approximate_pos")]);

            log::debug!("co1 obj_type");
            log::debug!("{:?}", system.current_state.variables[&EntityVariableKey::new("co1", "obj_type")]);

            log::debug!("h pos");
            log::debug!("{:?}", system.current_state.variables[&EntityVariableKey::new("h", "position")]);

            // Perform forward chaining
            let path = forward_chain(&goal, &bwd_result, &system);
            log::debug!("Results of forward chaining");
            log::debug!("Goal reachable: {}", !path.is_empty());
            log::debug!("{path:?}");

            if !path.is_empty() {
                Some(path)
            }
            else {
                None
            }
        };

        // Send command to controller
        if let Some(mut path) = path {
            tcp_interface.execute_command(&path[0]).expect("Failed to execute command with TCP");
            log::info!("Executed command {:?}", &path[0]);

            if !path.is_empty() {
                path.remove(0);
                committed_path = Some(path);
            }
            else {
                committed_path = None;

                log::debug!("Goal achieved, switching to next goal");
                goals.remove(0);
                goal = goals[0].clone();
            }
        }
        else {
            log::info!("No action found with forward chaining")
        }
    }
}

fn advance_time_step(data: &mut System) {
    let SystemTime::Exact(time) = data.current_state.time else {
        panic!("System time should always be exact during runtime");
    };
    // Increment by 100ms
    data.current_state.time = SystemTime::Exact(time + 100);
}