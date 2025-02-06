use crate::runtime::pattern_matching::{
    all_causal_models, all_req_models, are_goals_equal,
};
use crate::types::cst::Cst;
use crate::types::models::{BoundModel, Mdl};
use crate::types::pattern::PatternItem;
use crate::types::runtime::System;
use crate::types::{
    EntityPatternValue, Fact, MatchesFact, MkVal,
};
use itertools::Itertools;
use std::collections::HashMap;
use crate::types::value::Value;

pub fn backward_chain(goal: &Vec<Fact<MkVal>>, data: &System) -> Vec<BoundModel> {
    let mut instantiable_cas_mdl = Vec::new();

    let req_models = all_req_models(data);
    for m_req in &req_models {
        if let Some(bound_m_req) = m_req.try_instantiate_with_icst(&data.current_state) {
            let imdl = m_req.right.pattern.as_imdl();
            instantiable_cas_mdl.push(imdl.instantiate(&bound_m_req.bindings, data));
        }
    }

    let casual_models = all_causal_models(data);
    let mut observed_goals = Vec::new();
    get_goal_requirements_for_goal(
        goal,
        &instantiable_cas_mdl,
        &casual_models,
        data,
        &mut observed_goals,
    )
}

/// The recursive part of backward chaining
fn get_goal_requirements_for_goal(
    goal: &Vec<Fact<MkVal>>,
    instantiable_cas_mdl: &Vec<BoundModel>,
    casual_models: &Vec<Mdl>,
    data: &System,
    observed_goals: &mut Vec<Vec<Fact<MkVal>>>,
) -> Vec<BoundModel> {
    let mut goal_requirements = Vec::new();

    let casual_goal_models = casual_models
        .iter()
        // Find all casual models where rhs matches a fact from the goal
        .filter(|m| {
            goal.iter().any(|goal_val| {
                let rhs_fact = Fact {
                    pattern: m.right.pattern.as_mk_val().clone(),
                    time_range: m.right.time_range.clone(),
                };
                rhs_fact.matches_fact(goal_val)
            })
        })
        // Find and insert values from the goal that are supposed to be assigned to mk.val on rhs of casual model that matches the goal
        .flat_map(|c_goal_model| {
            insert_bindings_for_rhs_from_goal(
                &BoundModel {
                    model: c_goal_model.clone(),
                    bindings: HashMap::new(),
                },
                goal,
            )
        })
        .collect_vec();

    for c_goal_model in casual_goal_models {
        // If the casual model can be reached directly from the current state, then we don't have to look further back
        if instantiable_cas_mdl.iter().any(|imdl_val| { imdl_val.model.model_id == c_goal_model.model.model_id }) {
            goal_requirements.push(c_goal_model.clone());
            continue;
        }

        // Skip this casual model if rhs matches the current state.
        // We don't have to consider the part of the goal that ia already satisfied in the current state
        let rhs_mk_val = c_goal_model.model.right.pattern.as_mk_val();
        let rhs_mk_val_value = rhs_mk_val
            .value
            .get_value_with_bindings(&c_goal_model.bindings);
        if let Some(mk_val_entity_key) = rhs_mk_val.entity_key(&c_goal_model.bindings) {
            if matches!(
            &rhs_mk_val_value,
            Some(v) if data.current_state.variables
                .get(&mk_val_entity_key)
                .map(|val| val == v)
                .unwrap_or(false))
            {
                continue;
            }
        };

        // Add this casual model as a goal requirement to use during forward chaining
        goal_requirements.push(c_goal_model.clone());

        // Requirement models where rhs matches casual model,
        // with all bindings that we got from backward chaining (from casual model) included
        let goal_req_models = all_req_models(data)
            .into_iter()
            .filter(|m| m.right.pattern.as_imdl().model_id == c_goal_model.model.model_id)
            .map(|m| m.backward_chain_known_bindings_from_imdl(&c_goal_model))
            .collect_vec();

        // Create sub goals from the requirement models
        for g_req in &goal_req_models {
            let sub_goal_cst = g_req.model.left.pattern.as_icst().expand_cst(data);
            let mut sub_goal = sub_goal_cst.facts.clone();

            insert_bindings_into_facts(&mut sub_goal, &g_req.bindings);

            let mut all_sub_goals = create_variations_of_sub_goal(&sub_goal, sub_goal_cst, data);
            all_sub_goals.insert(0, sub_goal);

            for sub_goal in &all_sub_goals {
                // Don't check goals that have been seen before, to prevent an infinite loop
                if observed_goals.iter().any(|g| are_goals_equal(g, sub_goal)) {
                    continue;
                }
                observed_goals.push(sub_goal.clone());

                goal_requirements.extend(get_goal_requirements_for_goal(
                    &sub_goal,
                    instantiable_cas_mdl,
                    casual_models,
                    data,
                    observed_goals,
                ));
            }
        }
    }

    goal_requirements
}

/// Create variations of the subgoal with possible binding assignments
/// E.g. if the goal is for a hand and obj to both be at pos p, then two subgoals will be made,
/// one with p as the current pos of the hand and another with p as the current pos of the obj
fn create_variations_of_sub_goal(
    goal: &Vec<Fact<MkVal>>,
    goal_cst: Cst,
    system: &System,
) -> Vec<Vec<Fact<MkVal>>> {
    let goal_cst = Cst {
        cst_id: "".to_string(),
        facts: goal.clone(),
        // Keep entity binding declarations for entities which have not been filled in
        entities: goal_cst
            .entities
            .into_iter()
            .filter(|e| {
                goal.iter()
                    .any(|f| f.pattern.entity_id.is_binding(&e.binding))
            })
            .collect(),
    };
    let bindings = goal_cst.binding_params();
    let mut possible_entity_binding = goal_cst.all_possible_entity_bindings(system);
    // Possible entity bindings can be zero if there are no entity declarations in cst
    if possible_entity_binding.is_empty() {
        possible_entity_binding.push(HashMap::new());
    }

    possible_entity_binding
        .iter()
        .flat_map(|entity_bindings| {
            bindings
                .iter()
                .filter(|b| !entity_bindings.contains_key(&**b))
                // Get all possible values for each binding and create a 2d list from them, e.g. [ [("a", 2.0), ("a", 5.0)], [("b", 7.0)] ]
                .map(|b| {
                    let res = goal.iter()
                        .filter(|f| f.pattern.value.is_binding(b))
                        .filter_map(|f| {
                            system.current_state
                                .variables
                                .get(&f.pattern.entity_key(&entity_bindings).unwrap())
                        })
                        .map(|v| (b.clone(), v.clone()))
                        .collect_vec();
                    res
                })
                // Make all possible combinations of binding values
                .multi_cartesian_product()
                // Combine value bindings with entity bindings and create facts containing them for the goal
                .map(|bindings| {
                    let mut bindings: HashMap<String, Value> =
                        bindings.into_iter().collect();
                    // Insert the entity bindings into the map as well
                    bindings.extend(entity_bindings.clone().into_iter().collect_vec());

                    goal_cst.fill_in_bindings(&bindings).facts
                })
        })
        .collect()
}

/// Turn bindings into values in facts based on a binding map
/// Ignores bindings that don't exist in the binding map (they will stay undbound)
fn insert_bindings_into_facts(
    facts: &mut Vec<Fact<MkVal>>,
    bindings: &HashMap<String, Value>,
) {
    for fact in facts {
        match &fact.pattern.value {
            PatternItem::Binding(b) if bindings.contains_key(b) => {
                fact.pattern.value = PatternItem::Value(bindings[b].clone());
            }
            _ => {}
        }
        match &fact.pattern.entity_id {
            EntityPatternValue::Binding(b) if bindings.contains_key(b) => {
                fact.pattern.entity_id =
                    EntityPatternValue::EntityId(bindings[b].as_entity_id().to_owned());
            }
            _ => {}
        }
    }
}

/// Get bindings from the goal and insert them into the model
/// If rhs of the model matches multiple facts of the goal,
/// then multiple models will be created, one for each
fn insert_bindings_for_rhs_from_goal(
    casual_model: &BoundModel,
    goal: &Vec<Fact<MkVal>>,
) -> Vec<BoundModel> {
    let mut result = Vec::new();
    for goal_fact in goal {
        let mk_val = casual_model.model.right.pattern.as_mk_val();
        let matches_rhs = match (&goal_fact.pattern.entity_id, &mk_val.entity_id) {
            (EntityPatternValue::EntityId(e1), EntityPatternValue::EntityId(e2)) => {
                e1 == e2 && goal_fact.pattern.var_name == mk_val.var_name
            }
            // If either is a binding, then we have no way of knowing if the entity matches or not (as it depends on the req model) so we assume that it is a match
            _ => goal_fact.pattern.var_name == mk_val.var_name,
        };
        if !matches_rhs {
            continue;
        }
        let mut casual_model = casual_model.clone();

        if let PatternItem::Binding(binding) = &mk_val.value {
            let goal_value = goal_fact.pattern.value.clone();

            // Only add binding if goal value is a real value
            match goal_value {
                PatternItem::Value(value) => {
                    casual_model
                        .bindings
                        .insert(binding.to_owned(), value);
                }
                _ => {}
            }
        }

        if let EntityPatternValue::Binding(binding) = &mk_val.entity_id {
            // Same with entity id
            match &goal_fact.pattern.entity_id {
                EntityPatternValue::EntityId(entity_id) => {
                    casual_model.bindings.insert(
                        binding.to_owned(),
                        Value::EntityId(entity_id.clone()),
                    );
                }
                _ => {}
            }
        }

        casual_model.compute_backward_bindings();
        result.push(casual_model.clone());
    }

    result
}
