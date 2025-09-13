mod ctpx;
mod utils;
mod model_comparison;
mod ptpx;
mod cst;
mod full_causal_model_comparison;

use crate::types::EntityVariableKey;
use crate::types::models::{IMdl, MdlLeftValue, MdlRightValue};
use crate::types::runtime::{RuntimeCommand, System, SystemState};
use crate::types::value::Value;

pub fn extract_patterns(executed_command: &RuntimeCommand, system: &mut System, state_before: &SystemState, predicted_changes: &Vec<(EntityVariableKey, Value, IMdl)>) {
    log::debug!("Checking for patterns");
    for (key, value) in &system.current_state.variables.clone() {
        let old_value = state_before.variables.get(key);
        // Fact changed after executing command, and we have no model that predicted it
        if Some(value) != old_value && !predicted_changes.iter().any(|(k, v, _)| key == k && value == v) {
            log::debug!("Found change on {key:?}");
            ctpx::extract_patterns(key, old_value, value, executed_command, system, state_before);
        }
    }

    for (key, predicted_value, model) in predicted_changes {
        let Some(current_value) = system.current_state.variables.get(key).cloned() else {
            continue
        };
        log::debug!("Expected change {predicted_value} on {key:?} using model {}", model.model_id);
        let cst_id_of_model = system.models.values().find_map(|m| match (&m.left.pattern, &m.right.pattern) {
            (MdlLeftValue::ICst(icst), MdlRightValue::IMdl(imdl)) if imdl.model_id == model.model_id => Some(icst.cst_id.clone()),
            _ => None
        });

        // The state did not change when we expected it to
        if &current_value != predicted_value {
            log::debug!("Expected change did not happen, model {} demoted (expected {} got {})", model.model_id, &predicted_value, &current_value);
            let model_ref = system.models.get_mut(&model.model_id).unwrap();
            model_ref.demote();
            
            if let Some(cst_id) = &cst_id_of_model {
                log::debug!("Cst {} also demoted", cst_id);
                let cst_ref = system.csts.get_mut(cst_id).unwrap();
                cst_ref.demote();
            }
            // ptpx::extract_patterns(key, old_value, &current_value, predicted_value, model, executed_command, system, state_before);
        }
        else {
            log::debug!("Expected change did happen, model {} promoted", model.model_id);
            let model_ref = system.models.get_mut(&model.model_id).unwrap();
            model_ref.promote();

            if let Some(cst_id) = &cst_id_of_model {
                log::debug!("Cst {} also promoted", cst_id);
                let cst_ref = system.csts.get_mut(cst_id).unwrap();
                cst_ref.promote();
            }
        }
    }
}