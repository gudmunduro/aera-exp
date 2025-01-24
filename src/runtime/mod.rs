mod seed;
pub mod pattern_matching;
mod simulation;

use crate::types::runtime::{RuntimeData, RuntimeValue};
use crate::runtime::pattern_matching::{compute_instantiated_states, models_for_cst};
use crate::runtime::simulation::backward_chain;
use crate::types::{EntityVariableKey, Fact, MkVal, TimePatternRange, TimePatternValue};
use crate::types::pattern::{PatternItem, PatternValue};

pub fn run_aera() {
    let mut runtime_data = RuntimeData::new();
    seed::setup_seed(&mut runtime_data);
    runtime_data.current_state.variables.insert(EntityVariableKey::new("h", "position"), RuntimeValue::Number(0.0));

    loop {
        let instantiated_states = compute_instantiated_states(&runtime_data, &runtime_data.current_state);
        runtime_data.current_state.instansiated_csts = instantiated_states.into_iter().map(|s| (s.cst_id.clone(), s)).collect();

        for state in runtime_data.current_state.instansiated_csts.values() {
            println!("State: {}", state.cst_id);

            for model in &models_for_cst(&state, &runtime_data) {
                println!("Model ({}) has been instantiated with forward chaining", model.model.model_id);
            }
        }

        let chain_result = backward_chain(&vec![
            Fact {
                pattern: MkVal {
                    entity_id: "h".to_string(),
                    var_name: "position".to_string(),
                    value: PatternItem::Value(PatternValue::Number(1.0)),
                },
                time_range: TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
            }
        ], &runtime_data);
        println!("Results of backward chaining");
        println!("{chain_result:#?}");

        break;
    }
}

