mod seed;
pub mod pattern_matching;
mod simulation;

use crate::types::runtime::{RuntimeData, RuntimeValue};
use crate::runtime::pattern_matching::{compute_instantiated_states};
use crate::runtime::simulation::{backward_chain, forward_chain};
use crate::types::{EntityVariableKey, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::pattern::{PatternItem, PatternValue};

pub fn run_aera() {
    let mut runtime_data = RuntimeData::new();
    seed::setup_seed(&mut runtime_data);
    runtime_data.current_state.variables.insert(EntityVariableKey::new("h", "position"), RuntimeValue::Number(0.0));
    runtime_data.current_state.instansiated_csts = compute_instantiated_states(&runtime_data, &runtime_data.current_state);

    println!("Instantiated composite states");
    for state in runtime_data.current_state.instansiated_csts.values() {
        println!("State: {}", state.cst_id);
    }

    let goal = vec![
        Fact {
            pattern: MkVal {
                entity_id: "h".to_string(),
                var_name: "position".to_string(),
                value: PatternItem::Value(PatternValue::Number(1.0)),
            },
            time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
        },
    ];

    let bwd_result = backward_chain(&goal, &runtime_data);
    println!("Results of backward chaining");
    println!("{bwd_result:#?}");

    let fwd_result = forward_chain(&goal, &bwd_result, &runtime_data.current_state, &runtime_data);
    println!("Results of forward chaining");
    println!("{fwd_result:#?}");
}

