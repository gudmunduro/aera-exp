
pub mod pattern_matching;
pub mod simulation;
mod seeds;

use std::vec;
use crate::interfaces::tcp_interface::TcpInterface;
use crate::types::runtime::{RuntimeCommand, System, SystemState, SystemTime};
use crate::runtime::pattern_matching::{compute_assumptions, compute_instantiated_states, state_matches_facts};
use crate::runtime::simulation::backward::backward_chain;
use crate::runtime::simulation::forward::forward_chain;
use crate::types::{EntityPatternValue, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::pattern::{PatternItem};
use crate::types::value::Value;

pub fn run_demo() {
    let mut system = System::new();
    seeds::setup_bindings_seed(&mut system);

    system.current_state.instansiated_csts = compute_instantiated_states(&system, &system.current_state);
    system.current_state.variables.extend(compute_assumptions(&system, &system.current_state));
    system.current_state.instansiated_csts = compute_instantiated_states(&system, &system.current_state);

    log::debug!("Instantiated composite states");
    for state in system.current_state.instansiated_csts.values().flatten() {
        log::debug!("{}", state.icst_for_cst());
    }

    let goal = vec![
        Fact {
            pattern: MkVal {
                entity_id: EntityPatternValue::EntityId("o".to_string()),
                var_name: "pos".to_string(),
                value: PatternItem::Value(Value::Vec(vec![Value::UncertainNumber(5.0, 0.1), Value::UncertainNumber(7.0, 0.1)])),
                assumption: false,
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
                    entity_id: EntityPatternValue::EntityId("2".to_string()),
                    var_name: "approximate_pos".to_string(),
                    value: PatternItem::Value(Value::Vec(vec![Value::Number(240.0), Value::Number(0.0), Value::Number(-107.0), Value::Number(180.0)])),
                    assumption: false,
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            },
        ],
        vec![
            Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Vec(vec![]),
                    assumption: false,
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            },
        ],
        vec![
            Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::EntityId("1".to_string()),
                    var_name: "approximate_pos".to_string(),
                    value: PatternItem::Value(Value::Vec(vec![Value::Number(240.0), Value::Number(0.0), Value::Number(-90.0), Value::Number(180.0)])),
                    assumption: false,
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            },
        ],
        vec![
            Fact {
                pattern: MkVal {
                    entity_id: EntityPatternValue::EntityId("h".to_string()),
                    var_name: "holding".to_string(),
                    value: PatternItem::Vec(vec![]),
                    assumption: false,
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            },
        ],
    ];

    let mut goal = goals.get(0).cloned().unwrap_or(Vec::new());

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        advance_time_step(&mut system);

        // Update state from TCP
        log::debug!("Waiting for variables");
        let tcp_variables = tcp_interface.update_variables();
        system.current_state.variables = tcp_variables;
        system.current_state.instansiated_csts = compute_instantiated_states(&system, &system.current_state);
        system.current_state.variables.extend(compute_assumptions(&system, &system.current_state));
        system.current_state.instansiated_csts = compute_instantiated_states(&system, &system.current_state);

        log::debug!("Got variables");
        print_all_variables(&system.current_state);

        log::debug!("Instantiated composite states");
        for state in system.current_state.instansiated_csts.values().flatten() {
            log::debug!("{}", state.icst_for_cst());
        }

        if state_matches_facts(&system.current_state, &goal) {
            log::info!("Goal achieved");
            goals.remove(0);
            goal = goals.get(0).cloned().unwrap_or(Vec::new());

            if goals.is_empty() {
                log::info!("All goals achieved");
            }
        }

        // Perform backward chaining
        let bwd_result = backward_chain(&goal, &system);
        log::debug!("Results of backward chaining");
        for mdl in &bwd_result {
            log::debug!("{mdl}");
        }

        // Perform forward chaining
        let mut path = forward_chain(&goal, &bwd_result, &system);
        log::debug!("Results of forward chaining");
        log::debug!("Goal reachable: {}", !path.is_empty());
        log::debug!("{path:?}");

        // Send command to controller
        if !path.is_empty() {
            tcp_interface.execute_command(&path[0]).expect("Failed to execute command with TCP");
            log::info!("Executed command {:?}", &path[0]);
        }
        else {
            log::info!("No action found with forward chaining");
            tcp_interface.execute_command(
                &RuntimeCommand {
                    name: "no_action".to_string(),
                    entity_id: "sys".to_string(),
                    params: Vec::new(),
                }).expect("Failed to execute command with TCP");
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

fn print_all_variables(state: &SystemState) {
    for (key, value) in &state.variables {
        let entity = &key.entity_id;
        let variable = &key.var_name;
        log::debug!("(mk.val {entity} {variable} {value})");
    }
}