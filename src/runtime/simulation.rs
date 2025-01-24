use crate::types::models::{BoundModel, Mdl};
use crate::types::pattern::PatternItem;
use crate::types::runtime::{AssignedMkVal, RuntimeData};
use crate::types::{Fact, MatchesFact, MkVal};
use std::collections::HashMap;
use itertools::Itertools;
use crate::runtime::pattern_matching::{all_causal_models, all_req_models};

pub fn backward_chain(goal: &Vec<Fact<MkVal>>, data: &RuntimeData) -> Vec<BoundModel> {
    let mut instantiable_c_mdl = Vec::new();

    let req_models = all_req_models(data);
    for m_req in &req_models {
        if let Some(bound_m_req) = m_req.try_instantiate_with_icst(&data.current_state) {
            let imdl = m_req.right.pattern.as_imdl();
            instantiable_c_mdl.push(imdl.instantiate(&bound_m_req.bindings, data));
        }
    }

    let casual_models = all_causal_models(data);
    get_goal_requirements_for_goal(goal, &instantiable_c_mdl, &casual_models, data)
}

fn get_goal_requirements_for_goal(goal: &Vec<Fact<MkVal>>, instantiable_c_mdl: &Vec<BoundModel>, casual_models: &Vec<Mdl>, data: &RuntimeData) -> Vec<BoundModel> {
    if goal.iter().all(|f| data.current_state.variables.get(&f.pattern.entity_key()).map(|v| *v == f.pattern.value).unwrap_or(false)) {
        return vec![];
    }

    let mut goal_requirements = Vec::new();

    let casual_goal_models = casual_models
        .iter()
        .filter(|m| goal.iter().any(|goal_val| {
            let rhs_fact = Fact { pattern: m.right.pattern.as_mk_val().clone(), time_range: m.right.time_range.clone() };
            rhs_fact.matches_fact(goal_val)
        }))
        .collect_vec();

    for c_goal_model in casual_goal_models {
        /*if instantiable_c_mdl.iter().any(|imdl_val| { imdl_val.model.model_id == c_goal_model.model_id }) {
            continue;
        }*/

        let mut c_goal_model = BoundModel {
            model: c_goal_model.clone(),
            bindings: HashMap::new()
        };

        // Find value from the goal that is supposed to be assigned to mk.val on rhs of casual model that matches the goal
        let mk_val = c_goal_model.model.right.pattern.as_mk_val();
        if let PatternItem::Binding(binding) = &mk_val.value {
            let goal_value = goal.iter()
                .find(|goal_val| goal_val.pattern.entity_id == mk_val.entity_id
                    && goal_val.pattern.var_name == mk_val.var_name)
                .unwrap()
                .pattern
                .value
                .clone();

            // Only add binding if goal value is a real value
            match goal_value {
                PatternItem::Value(value) => {
                    c_goal_model.bindings.insert(binding.to_owned(), value.into());
                }
                _ => {}
            }
        }

        // 5.d
        let goal_req_models = all_req_models(data)
            .into_iter()
            .filter(|m| m.right.pattern.as_imdl().model_id == c_goal_model.model.model_id)
            .map(|m| m.backward_chain_known_bindings_from_imdl(&c_goal_model))
            .collect_vec();
        // 5.c
        goal_requirements.push(c_goal_model);

        // 5.e
        for g_req in &goal_req_models {
            let mut sub_goal = g_req.model.left.pattern.as_icst().expand_cst(data).facts;

            // TODO: Skip facts with no value assigned? (essentially turn goal into list of assigned mk.val)
            for fact in &mut sub_goal {
                match &fact.pattern.value {
                    PatternItem::Binding(b) if g_req.bindings.contains_key(b) => {
                        fact.pattern.value = PatternItem::Value(g_req.bindings[b].clone().into());
                    }
                    _ => {}
                }
            }

            goal_requirements.extend(get_goal_requirements_for_goal(&sub_goal, instantiable_c_mdl, casual_models, data));
        }
    }

    goal_requirements
}