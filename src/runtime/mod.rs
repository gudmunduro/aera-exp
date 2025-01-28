mod seed;
pub mod pattern_matching;
mod simulation;

use crate::types::runtime::{RuntimeData, RuntimeValue, SystemTime};
use crate::runtime::pattern_matching::{compute_instantiated_states};
use crate::runtime::simulation::backward::backward_chain;
use crate::runtime::simulation::forward::forward_chain;
use crate::types::{EntityVariableKey, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::pattern::{PatternItem, PatternValue};

pub fn run_aera() {
    let mut runtime_data = RuntimeData::new();
    seed::setup_seed(&mut runtime_data);
    runtime_data.current_state.variables.insert(EntityVariableKey::new("h", "position"), RuntimeValue::Number(1.0));
    runtime_data.current_state.variables.insert(EntityVariableKey::new("o", "position"), RuntimeValue::Number(5.0));
    runtime_data.current_state.instansiated_csts = compute_instantiated_states(&runtime_data, &runtime_data.current_state);

    println!("Instantiated composite states");
    for state in runtime_data.current_state.instansiated_csts.values() {
        println!("State: {}", state.cst_id);
    }

    let goal = vec![
        Fact {
            pattern: MkVal {
                entity_id: "o".to_string(),
                var_name: "position".to_string(),
                value: PatternItem::Value(PatternValue::Number(7.0)),
            },
            time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
        },
    ];

    let bwd_result = backward_chain(&goal, &runtime_data);
    println!("Results of backward chaining");
    println!("{bwd_result:#?}");

    let (fwd_result, goal_reachable) = forward_chain(&goal, &bwd_result, &runtime_data.current_state, &runtime_data, &mut Vec::new());
    println!("Results of forward chaining");
    println!("Goal reachable: {goal_reachable}");
    println!("{fwd_result:#?}");

    advance_time_step(&mut runtime_data);
}

fn advance_time_step(data: &mut RuntimeData) {
    let SystemTime::Exact(time) = data.current_state.time else {
        panic!("System time should always be exact during runtime");
    };
    // Increment by 100ms
    data.current_state.time = SystemTime::Exact(time + 100);
}