mod ctpx;
mod utils;
mod model_comparison;

use crate::types::EntityVariableKey;
use crate::types::runtime::{RuntimeCommand, System, SystemState};
use crate::types::value::Value;

pub fn extract_patterns(executed_command: &RuntimeCommand, system: &mut System, state_before: &SystemState, predicted_changes: &Vec<(EntityVariableKey, Value)>) {
    log::debug!("Checking for patterns");
    for (key, value) in &system.current_state.variables.clone() {
        let Some(old_value) = state_before.variables.get(key) else {
            continue
        };
        // Fact changed after executing command, and we have no model that predicted it
        if value != old_value && !predicted_changes.iter().any(|(k, v)| key == k && value == v) {
            log::debug!("Found change on {key:?}");
            ctpx::extract_patterns(key, old_value, value, executed_command, system, state_before);
        }
    }
}