mod seed;
pub mod pattern_matching;
mod simulation;

use crate::types::runtime::{RuntimeData, RuntimeValue, RuntimeVariable};
use log::log;
use crate::runtime::pattern_matching::{compute_instantiated_states, models_for_cst};

pub fn run_aera() {
    let mut runtime_data = RuntimeData::new();
    seed::setup_seed(&mut runtime_data);
    runtime_data.current_state.variables.insert("position".to_string(), RuntimeValue::Number(0.0));

    loop {
        let instantiated_states = compute_instantiated_states(&runtime_data, &runtime_data.current_state);

        for state in instantiated_states {
            println!("State: {}", state.cst_id);

            for model in &models_for_cst(&state, &runtime_data) {
                println!("Model ({}) has been instantiated with forward chaining", model.model.model_id);
            }
        }

        break;
    }
}

