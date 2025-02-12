
pub mod pattern_matching;
mod simulation;
mod seeds;

use std::collections::HashSet;
use std::rc::Rc;
use itertools::Itertools;
use crate::interfaces::tcp_interface::TcpInterface;
use crate::types::runtime::{System, SystemTime};
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
        log::debug!("State: {}", state.cst_id);
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
    log::debug!("{bwd_result:#?}");

    let (fwd_result, goal_reachable) = forward_chain(&goal, &bwd_result, &system.current_state, &system, &mut HashSet::new());
    log::debug!("Results of forward chaining");
    log::debug!("Goal reachable: {goal_reachable}");
    log::debug!("{fwd_result:#?}");

    advance_time_step(&mut system);
}

#[allow(unused)]
pub fn run_with_tcp() {
    let mut tcp_interface = TcpInterface::connect().expect("Failed to connect to controller with TCP");
    let mut system = System::new();
    seeds::hand_grab_sphere::setup_hand_grab_sphere_seed(&mut system);

    let mut goals = vec![
        vec![
            Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Value(Value::List(vec![Value::Number(-1.0), Value::Number(0.5), Value::Number(0.0)])),
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            },
        ],
        vec![
            Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::EntityId("b_0".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Value(Value::List(vec![Value::Number(-1.0), Value::Number(0.5), Value::Number(0.0)])),
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            },
        ],
        vec![
            Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Value(Value::List(vec![])),
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            },
        ],
        vec![
            Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::EntityId("b_1".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Value(Value::List(vec![Value::Number(-1.0), Value::Number(0.7), Value::Number(0.0)])),
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            },
        ],
        vec![
            Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::EntityId("b_2".to_string()),
                    var_name: "position".to_string(),
                    value: PatternItem::Value(Value::List(vec![Value::Number(-1.0), Value::Number(0.3), Value::Number(0.0)])),
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            },
        ],
    ];

    let mut goal = goals[0].clone();
    let mut committed_path: Option<Rc<ForwardChainNode>> = None;

    loop {
        advance_time_step(&mut system);

        // Update state from TCP
        log::debug!("Waiting for variables");
        let tcp_variables = tcp_interface.update_variables();
        system.current_state.variables.extend(tcp_variables);
        system.current_state.instansiated_csts = compute_instantiated_states(&system, &system.current_state);

        log::debug!("Instantiated composite states");
        for state in system.current_state.instansiated_csts.values().flatten() {
            log::debug!("State: {}", state.cst_id);
        }

        let node = if let Some(committed) = committed_path.as_ref() {
            Some(committed.clone())
        } else {
            // Perform backward chaining
            let bwd_result = backward_chain(&goal, &system);
            log::debug!("Results of backward chaining");
            log::debug!("{bwd_result:#?}");

            log::debug!("b_0 pos");
            log::debug!("{:?}", system.current_state.variables[&EntityVariableKey::new("b_0", "position")]);

            log::debug!("h pos");
            log::debug!("{:?}", system.current_state.variables[&EntityVariableKey::new("h", "position")]);

            // Perform forward chaining
            let (fwd_result, goal_reachable) = forward_chain(&goal, &bwd_result, &system.current_state, &system, &mut HashSet::new());
            log::debug!("Results of forward chaining");
            log::debug!("Goal reachable: {goal_reachable}");
            log::debug!("{fwd_result:#?}");
            fwd_result.into_iter().filter(|n| n.is_in_goal_path).next()
        };

        // Send command to controller
        if let Some(node) = node {
            tcp_interface.execute_command(&node.command).expect("Failed to execute command with TCP");
            log::info!("Executed command {:?}", &node.command);

            if !node.children.is_empty() {
                committed_path = Some(node.children.iter().filter(|n| n.is_in_goal_path).next().unwrap().clone());
            }
            else if node.children.is_empty() && node.is_in_goal_path && goals.len() > 1 {
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