use crate::runtime::pattern_matching::{all_causal_models, all_req_models};
use crate::types::models::{BoundModel, Mdl};
use crate::types::pattern::{bindings_in_pattern, PatternItem};
use crate::types::runtime::{AssignedMkVal, RuntimeCommand, RuntimeData, SystemState};
use crate::types::{Command, Fact, MatchesFact, MkVal};
use itertools::Itertools;
use std::collections::HashMap;

pub fn backward_chain(goal: &Vec<Fact<MkVal>>, data: &RuntimeData) -> Vec<BoundModel> {
    let mut instantiable_cas_mdl = Vec::new();

    let req_models = all_req_models(data);
    for m_req in &req_models {
        if let Some(bound_m_req) = m_req.try_instantiate_with_icst(&data.current_state) {
            let imdl = m_req.right.pattern.as_imdl();
            instantiable_cas_mdl.push(imdl.instantiate(&bound_m_req.bindings, data));
        }
    }

    let casual_models = all_causal_models(data);
    get_goal_requirements_for_goal(goal, &instantiable_cas_mdl, &casual_models, data)
}

fn get_goal_requirements_for_goal(
    goal: &Vec<Fact<MkVal>>,
    instantiable_cas_mdl: &Vec<BoundModel>,
    casual_models: &Vec<Mdl>,
    data: &RuntimeData,
) -> Vec<BoundModel> {
    if goal.iter().all(|f| {
        data.current_state
            .variables
            .get(&f.pattern.entity_key())
            .map(|v| *v == f.pattern.value)
            .unwrap_or(false)
    }) {
        return vec![];
    }

    let mut goal_requirements = Vec::new();

    let casual_goal_models = casual_models
        .iter()
        .filter(|m| {
            goal.iter().any(|goal_val| {
                let rhs_fact = Fact {
                    pattern: m.right.pattern.as_mk_val().clone(),
                    time_range: m.right.time_range.clone(),
                };
                rhs_fact.matches_fact(goal_val)
            })
        })
        .collect_vec();

    for c_goal_model in casual_goal_models {
        // TODO: Re-add this but add to goal_requirements first
        // TODO: Then remove early return at top (since these conditions do the same thing)
        /*if instantiable_c_mdl.iter().any(|imdl_val| { imdl_val.model.model_id == c_goal_model.model_id }) {
            continue;
        }*/

        let mut c_goal_model = BoundModel {
            model: c_goal_model.clone(),
            bindings: HashMap::new(),
        };

        // Find value from the goal that is supposed to be assigned to mk.val on rhs of casual model that matches the goal
        let mk_val = c_goal_model.model.right.pattern.as_mk_val();
        if let PatternItem::Binding(binding) = &mk_val.value {
            let goal_value = goal
                .iter()
                .find(|goal_val| {
                    goal_val.pattern.entity_id == mk_val.entity_id
                        && goal_val.pattern.var_name == mk_val.var_name
                })
                .unwrap()
                .pattern
                .value
                .clone();

            // Only add binding if goal value is a real value
            match goal_value {
                PatternItem::Value(value) => {
                    c_goal_model
                        .bindings
                        .insert(binding.to_owned(), value.into());
                }
                _ => {}
            }
        }

        // 5.c
        goal_requirements.push(c_goal_model.clone());

        // 5.d
        let goal_req_models = all_req_models(data)
            .into_iter()
            .filter(|m| m.right.pattern.as_imdl().model_id == c_goal_model.model.model_id)
            .map(|m| m.backward_chain_known_bindings_from_imdl(&c_goal_model))
            .collect_vec();

        // 5.e
        for g_req in &goal_req_models {
            let mut sub_goal = g_req.model.left.pattern.as_icst().expand_cst(data).facts;

            for fact in &mut sub_goal {
                match &fact.pattern.value {
                    PatternItem::Binding(b) if g_req.bindings.contains_key(b) => {
                        fact.pattern.value = PatternItem::Value(g_req.bindings[b].clone().into());
                    }
                    _ => {}
                }
            }

            goal_requirements.extend(get_goal_requirements_for_goal(
                &sub_goal,
                instantiable_cas_mdl,
                casual_models,
                data,
            ));
        }
    }

    goal_requirements
}

pub fn forward_chain(
    goal: &Vec<Fact<MkVal>>,
    goal_requirements: &Vec<BoundModel>,
    state: &SystemState,
    data: &RuntimeData,
) -> Vec<RuntimeCommand> {
    if goal.iter().all(|f| {
        state
            .variables
            .get(&f.pattern.entity_key())
            .map(|v| *v == f.pattern.value)
            .unwrap_or(false)
    }) {
        return vec![];
    }

    let mut commands = Vec::new();

    let insatiable_req_models = all_req_models(data)
        .into_iter()
        .filter_map(|m| m.try_instantiate_with_icst(&state))
        .collect_vec();

    for req_model in &insatiable_req_models {
        let Some(casual_model) = goal_requirements
            .iter()
            .find(|cm| cm.model.model_id == req_model.model.right.pattern.as_imdl().model_id)
        else {
            continue;
        };

        // 2.1
        let backward_chained_model = req_model
            .model
            .backward_chain_known_bindings_from_imdl(casual_model);

        // Can req_model instantiate casual_model with equal values
        // We only need to look at known bindings from req_model that appear in the pattern
        // (unknown bindings that appear only in rhs can be filled in with backward chaining)
        let pattern_bindings = bindings_in_pattern(&req_model.model.right.pattern.as_imdl().params);
        let bindings_to_compare = pattern_bindings
            .into_iter()
            .filter(|b| {
                req_model.bindings.contains_key(b)
                    && backward_chained_model.bindings.contains_key(b)
            })
            .collect_vec();
        let pattern_bindings_match_casual_model = req_model
            .bindings
            .iter()
            .filter(|(b, _)| bindings_to_compare.contains(b))
            .all(|(b, v)| {
                backward_chained_model
                    .bindings
                    .iter()
                    .any(|(b2, v2)| b == b2 && v == v2)
            });

        if pattern_bindings_match_casual_model {
            let mut fwd_chained_model = req_model
                .model
                .right
                .pattern
                .as_imdl()
                .instantiate(&req_model.bindings, data);

            // Fill in bindings that we got from backward chaining but not forward chaining
            fwd_chained_model.bindings.extend(
                casual_model
                    .bindings
                    .iter()
                    .filter(|(b, _)| !fwd_chained_model.bindings.contains_key(*b))
                    .map(|(b, v)| (b.clone(), v.clone()))
                    .collect_vec(),
            );

            let command = fwd_chained_model.model.left.pattern.as_command().to_runtime_command(&fwd_chained_model.bindings);
            commands.push(command);

            let state = fwd_chained_model.predict_state_change(&state, data);
            commands.extend(forward_chain(goal, goal_requirements, &state, data));
            break;
        }
    }

    commands
}
