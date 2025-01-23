use crate::types::models::BoundModel;
use crate::types::pattern::PatternItem;
use crate::types::runtime::{AssignedMkVal, RuntimeData};
use crate::types::Fact;
use std::collections::HashMap;
use itertools::Itertools;
use crate::runtime::pattern_matching::{all_causal_models, all_req_models};
use crate::types::cst::BoundCst;

pub struct GoalRequirement {
    // Cst expanded from lhs pattern in model and including bindings from backward chaining (values from the goal)
    pub req: BoundCst,
}

fn backward_chain(data: &RuntimeData, goal: &Vec<Fact<AssignedMkVal>>) -> Vec<GoalRequirement> {
    let mut goal_requirements = Vec::new();
    let mut instantiable_c_mdl = Vec::new();

    let req_models = all_req_models(data);
    for m_req in &req_models {
        if let Some(bound_m_req) = m_req.try_instantiate_with_icst(&data.current_state) {
            let imdl = m_req.right.pattern.as_imdl();
            instantiable_c_mdl.push(imdl.instantiate(&bound_m_req.bindings, data));
        }
    }

    let casual_models = all_causal_models(data);
    loop {
        let casual_goal_models = casual_models
            .iter()
            .filter(|m| goal.iter().any(|goal_val| {
                let rhs_fact = Fact { pattern: m.right.pattern.as_mk_val().clone(), time_range: m.right.time_range.clone() };
                rhs_fact.matches_fact(goal_val)
            }))
            .collect_vec();

        for c_goal_model in casual_goal_models {
            if instantiable_c_mdl.iter().any(|imdl_val| { imdl_val.model.model_id == c_goal_model.model_id }) {
                continue;
            }

            let mut goal_model = BoundModel {
                model: c_goal_model.clone(),
                bindings: HashMap::new()
            };

            let mk_val = goal_model.model.right.pattern.as_mk_val();
            if let PatternItem::Binding(binding) = &mk_val.value {
                let goal_value = goal.iter()
                    .find(|goal_val| goal_val.pattern.entity_id == mk_val.entity_id
                        && goal_val.pattern.var_name == mk_val.var_name)
                    .unwrap()
                    .pattern
                    .value
                    .clone();

                goal_model.bindings.insert(binding.to_owned(), goal_value);
            }

            todo!("Step 5.c+")
        }
    }

    goal_requirements
}