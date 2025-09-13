use std::collections::HashMap;
use itertools::Itertools;
use crate::types::cst::BoundCst;
use crate::types::{EntityVariableKey, MkVal};
use crate::types::models::{Mdl, MdlRightValue};
use crate::types::runtime::{System, SystemState};
use crate::types::value::Value;

pub const MODEL_CONFIDENCE_THRESHOLD: f64 = 0.59;

pub fn compute_instantiated_states(
    system: &System,
    state: &SystemState,
) -> HashMap<String, Vec<BoundCst>> {
    system
        .csts
        .iter()
        .map(|(id, cst)| {
            if cst.confidence() > MODEL_CONFIDENCE_THRESHOLD {
                let csts = BoundCst::try_instantiate_from_state(cst, state, system);

                (id.clone(), csts)
            }
            else {
                (id.clone(), vec![])
            }
        })
        .collect()
}

pub fn compute_assumptions(system: &System, state: &SystemState) -> HashMap<EntityVariableKey, Value> {
    let models = all_assumption_models(&system)
        .into_iter()
        .flat_map(|m| m.try_instantiate_with_icst(state))
        .collect_vec();
    models.into_iter()
        .filter_map(|m| match m.model.right.pattern {
            MdlRightValue::MkVal(rhs @ MkVal { assumption: true, .. }) => {
                let entity_id = rhs.entity_id.get_id_with_bindings(&m.bindings)
                    .expect("Cannot fill in entity id binding of assumption");
                let value = rhs.value.get_value_with_bindings(&m.bindings)
                    .expect("Cannot fill in all bindings of assumption");
                Some((EntityVariableKey { entity_id, var_name: rhs.var_name }, value))
            },
            _ => None
        })
        .collect()
}

pub fn compute_state_predictions(system: &System, state: &SystemState) -> HashMap<EntityVariableKey, Value> {
    let models = all_state_prediction_models(&system)
        .into_iter()
        .flat_map(|m| m.try_instantiate_with_icst(state))
        .collect_vec();
    models.into_iter()
        .filter_map(|m| match m.model.right.pattern {
            MdlRightValue::MkVal(rhs @ MkVal { assumption: false, .. }) => {
                let entity_id = rhs.entity_id.get_id_with_bindings(&m.bindings)
                    .expect("Cannot fill in entity id binding of state prediction");
                let value = rhs.value.get_value_with_bindings(&m.bindings)
                    .expect("Cannot fill in all bindings of state prediction");
                Some((EntityVariableKey { entity_id, var_name: rhs.var_name }, value))
            },
            _ => None
        })
        .collect()
}

pub fn all_causal_models(data: &System) -> Vec<Mdl> {
    data.models
        .iter()
        .filter(|(_, m)| m.is_casual_model())
        .map(|(_, m)| m)
        .cloned()
        .collect()
}

pub fn all_req_models(data: &System) -> Vec<Mdl> {
    data.models
        .iter()
        .filter(|(_, m)| m.is_req_model())
        .map(|(_, m)| m)
        .cloned()
        .collect()
}

pub fn all_assumption_models(data: &System) -> Vec<Mdl> {
    data.models
        .iter()
        .filter(|(_, m)| m.is_assumption_model())
        .map(|(_, m)| m)
        .cloned()
        .collect()
}

pub fn all_state_prediction_models(data: &System) -> Vec<Mdl> {
    data.models
        .iter()
        .filter(|(_, m)| m.is_state_prediction())
        .map(|(_, m)| m)
        .cloned()
        .collect()
}