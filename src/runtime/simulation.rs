use crate::types::models::{BoundModel, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::PatternItem;
use crate::types::runtime::{AssignedMkVal, RuntimeData, RuntimeValue};
use crate::types::{EntityVariableKey, Fact, MkVal};
use std::collections::HashMap;
use itertools::Itertools;
use crate::runtime::pattern_matching::{all_causal_models, all_req_models};
use crate::types::cst::BoundCst;

pub struct SimulationGoalModel {
    pub variables: HashMap<EntityVariableKey, RuntimeValue>,
}

pub struct BackwardChainResult {
    sub_goals: Vec<SimulationGoalModel>,
}

// TODO: Search backward from the goal until we find the current state, or until there is nothing more to do. (no model rhs matches current state)
pub fn perform_backward_chaining(goal_model: &Mdl, data: &RuntimeData) -> BackwardChainResult {
    let MdlLeftValue::MkVal(goal_value) = &goal_model.left.pattern else {
        panic!("Only goal models with lhs as mk.val are supported");
    };
    let PatternItem::Value(value) = &goal_value.value else {
        panic!("mk.val in goal can only contain literal value");
    };

    let state_var = (
        EntityVariableKey::new(&goal_value.entity_id, &goal_value.var_name),
        RuntimeValue::from(value.clone()),
    );
    let state = [state_var].into_iter().collect();
    let sub_goals = backward_sub_goals_for_state(&state, data);
    BackwardChainResult { sub_goals }
}

fn backward_sub_goals_for_state(
    state: &HashMap<EntityVariableKey, RuntimeValue>,
    data: &RuntimeData,
) -> Vec<SimulationGoalModel> {
    let models_matching_rhs = data.models.values().filter(|m| match &m.right.pattern {
        MdlRightValue::MkVal(mk_val) => compare_model_to_state_for_backward_chain(state, &mk_val),
        _ => false,
    })
        .map(|m| todo!());

    todo!()
}

fn compare_model_to_state_for_backward_chain(
    state: &HashMap<EntityVariableKey, RuntimeValue>,
    model_rhs: &MkVal,
) -> bool {
    match &model_rhs.value {
        PatternItem::Any => false, // Ignore models with wildcard rhs, possibly should be reconsidered
        PatternItem::Binding(_) => {
            todo!("Implement comparison with binding")
        }
        PatternItem::Value(v) => state
            .get(&EntityVariableKey::new(
                &model_rhs.entity_id,
                &model_rhs.var_name,
            ))
            .map(|state_val| state_val == v)
            .unwrap_or(false),
    }
}

// TODO: This is more complicated, likely needs to take req model and req cst into account
fn backward_chain_model(model: &Mdl, state: &HashMap<EntityVariableKey, RuntimeValue>) -> HashMap<EntityVariableKey, RuntimeValue> {
    todo!()
}


//
// Attempt NR. 2
//

pub struct GoalRequirement {
    // Cst expanded from lhs pattern in model and including bindings from backward chaining (values from the goal)
    pub req: BoundCst,
}

fn backward_chain_2(data: &RuntimeData, goal: &Vec<Fact<AssignedMkVal>>) -> Vec<GoalRequirement> {
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


        }
    }

    goal_requirements
}