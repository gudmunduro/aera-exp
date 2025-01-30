mod seed;
pub mod pattern_matching;
mod simulation;

use itertools::Itertools;
use crate::interfaces::tcp_interface::TcpInterface;
use crate::types::runtime::{System, RuntimeValue, SystemTime};
use crate::runtime::pattern_matching::{compute_instantiated_states};
use crate::runtime::simulation::backward::backward_chain;
use crate::runtime::simulation::forward::forward_chain;
use crate::types::{EntityPatternValue, EntityVariableKey, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::pattern::{PatternItem, PatternValue};

pub fn run_demo() {
    let mut system = System::new();
    seed::setup_simple_seed(&mut system);
    system.current_state.instansiated_csts = compute_instantiated_states(&system, &system.current_state);

    log::debug!("Instantiated composite states");
    for state in system.current_state.instansiated_csts.values() {
        log::debug!("State: {}", state.cst_id);
    }

    let goal = vec![
        Fact {
            pattern: MkVal {
                entity_id: EntityPatternValue::EntityId("o".to_string()),
                var_name: "position".to_string(),
                value: PatternItem::Value(PatternValue::Number(7.0)),
            },
            time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
        },
    ];

    let bwd_result = backward_chain(&goal, &system);
    log::debug!("Results of backward chaining");
    log::debug!("{bwd_result:#?}");

    let (fwd_result, goal_reachable) = forward_chain(&goal, &bwd_result, &system.current_state, &system, &mut Vec::new());
    log::debug!("Results of forward chaining");
    log::debug!("Goal reachable: {goal_reachable}");
    log::debug!("{fwd_result:#?}");

    advance_time_step(&mut system);
}

pub fn run_with_tcp() {
    let mut tcp_interface = TcpInterface::connect().expect("Failed to connect to controller with TCP");
    let mut system = System::new();
    seed::setup_simple_seed(&mut system);

    let goal = vec![
        Fact {
            pattern: MkVal {
                entity_id: EntityPatternValue::EntityId("o".to_string()),
                var_name: "position".to_string(),
                value: PatternItem::Value(PatternValue::Number(7.0)),
            },
            time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
        },
    ];

    loop {
        advance_time_step(&mut system);

        // Update state from TCP
        log::debug!("Waiting for variables");
        let tcp_variables = tcp_interface.update_variables();
        system.current_state.variables.extend(tcp_variables);
        system.current_state.instansiated_csts = compute_instantiated_states(&system, &system.current_state);

        // Perform backward chaining
        let bwd_result = backward_chain(&goal, &system);
        log::debug!("Results of backward chaining");
        log::debug!("{bwd_result:#?}");

        // Perform forward chaining
        let (fwd_result, goal_reachable) = forward_chain(&goal, &bwd_result, &system.current_state, &system, &mut Vec::new());
        log::debug!("Results of forward chaining");
        log::debug!("Goal reachable: {goal_reachable}");
        log::debug!("{fwd_result:#?}");

        // Send command to controller
        let node = fwd_result.iter().sorted_by_key(|n| n.is_in_goal_path).rev().next();
        if let Some(node) = node {
            tcp_interface.execute_command(&node.command).expect("Failed to execute command with TCP");
            log::info!("Executed command {:?}", &node.command);
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