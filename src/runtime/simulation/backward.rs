use crate::runtime::pattern_matching::{all_assumption_models, all_causal_models, all_req_models, all_state_prediction_models, are_goals_equal, compare_imdls, extract_bindings_from_patterns, extract_duplicate_bindings_from_pattern};
use crate::types::cst::Cst;
use crate::types::models::{BoundModel, IMdl, Mdl};
use crate::types::pattern::PatternItem;
use crate::types::runtime::System;
use crate::types::value::Value;
use crate::types::{EntityPatternValue, Fact, MatchesFact, MkVal};
use itertools::Itertools;
use std::collections::HashMap;

const MAX_DEPTH: usize = 10;

pub fn backward_chain(goal: &Vec<Fact<MkVal>>, data: &System) -> Vec<IMdl> {
    let mut instantiable_cas_mdl = Vec::new();

    let req_models = all_req_models(data);
    for m_req in &req_models {
        for bound_m_req in m_req.try_instantiate_with_icst(&data.current_state) {
            let imdl = m_req.right.pattern.as_filled_in_imdl(&bound_m_req.bindings);
            instantiable_cas_mdl.push(imdl.clone());
        }
    }

    let casual_models = all_causal_models(data);
    let mut state_prediction_models = all_assumption_models(data);
    state_prediction_models.extend(all_state_prediction_models(data));
    let mut observed_goals = Vec::new();
    let (goal_req_models, _) = get_goal_requirements_for_goal(
        goal,
        &instantiable_cas_mdl,
        &casual_models,
        &state_prediction_models,
        data,
        &mut observed_goals,
        0,
    );
    goal_req_models
        .into_iter()
        .unique()
        .collect()
}

/// The recursive part of backward chaining
fn get_goal_requirements_for_goal(
    goal: &Vec<Fact<MkVal>>,
    instantiable_cas_mdl: &Vec<IMdl>,
    casual_models: &Vec<Mdl>,
    assumption_models: &Vec<Mdl>,
    data: &System,
    observed_goals: &mut Vec<Vec<Fact<MkVal>>>,
    depth: usize,
) -> (Vec<IMdl>, bool) {
    if depth >= MAX_DEPTH {
        return (Vec::new(), false);
    }

    let mut reached_current_state = false;
    let mut goal_requirements: Vec<IMdl> = Vec::new();

    let goal_models = casual_models
        .iter()
        .chain(assumption_models.iter())
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
        .map(|m| m.imdl_for_model())
        .collect_vec();

    for goal_model in goal_models {
        // If the casual model can be reached directly from the current state, then we don't have to look further back
        if instantiable_cas_mdl.iter().any(|imdl_val| {
            compare_imdls(imdl_val, &goal_model, true, true)
        }) {
            reached_current_state = true;
            goal_requirements.push(goal_model.clone());
            continue;
        }

        let goal_model_bm = goal_model.instantiate(&HashMap::new(), &data);

        // Skip this casual model if rhs matches the current state.
        // We don't have to consider the part of the goal that ia already satisfied in the current state
        let rhs_mk_val = goal_model_bm.model.right.pattern.as_mk_val();
        let rhs_mk_val_value = rhs_mk_val
            .value
            .get_value_with_bindings(&goal_model_bm.bindings);
        if let Some(mk_val_entity_key) = rhs_mk_val.entity_key(&goal_model_bm.bindings) {
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

        if goal_model_bm.model.is_casual_model() {
            // Add this casual model as a goal requirement to use during forward chaining
            goal_requirements.push(goal_model.clone());
        }

        let sub_goal_models = if goal_model_bm.model.is_casual_model() {
            // Requirement models where rhs matches casual model,
            // with all bindings that we got from backward chaining (from casual model) included
            all_req_models(data)
                .into_iter()
                .filter(|m| compare_imdls(&m.right.pattern.as_imdl(), &goal_model, true, true))
                .map(|m| m.backward_chain_known_bindings_from_imdl(&goal_model))
                .collect_vec()
        }
        else {
            // Reuse same model instead of backward chaining if it is an assumption model
            vec![goal_model_bm]
        };

        // Create sub goals from the requirement models
        for g_req in &sub_goal_models {
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

                let (sub_goal_req_models, sub_goal_reached_current_state) =
                    get_goal_requirements_for_goal(
                        &sub_goal,
                        instantiable_cas_mdl,
                        casual_models,
                        assumption_models,
                        data,
                        observed_goals,
                        depth + 1,
                    );

                // Prune dead ends from backward chaining tree (paths that cannot be used to reach the current state)
                if sub_goal_reached_current_state {
                    goal_requirements.extend(sub_goal_req_models);
                    reached_current_state = true;
                }
            }
        }
    }

    (goal_requirements, reached_current_state)
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
                    let res = goal
                        .iter()
                        .filter(|f| f.pattern.value.contains_binding(b))
                        .flat_map(|f| {
                            // TODO: Was needed to support entity bindings of assumptions, check if subgoal variations are properly created in that case
                            let Some(key) = f.pattern.entity_key(&entity_bindings) else {
                                return Vec::new();
                            };
                            let Some(sys_value) = system
                                .current_state
                                .variables
                                .get(&key) else {
                                return Vec::new();
                            };

                            extract_duplicate_bindings_from_pattern(&f.pattern.value.pattern(), &vec![PatternItem::Value(sys_value.clone())])
                        })
                        .collect_vec();
                    res
                })
                // Make all possible combinations of binding values
                .multi_cartesian_product()
                // Combine value bindings with entity bindings and create facts containing them for the goal
                .map(|bindings| {
                    let mut bindings: HashMap<String, Value> = bindings.into_iter().collect();
                    // Insert the entity bindings into the map as well
                    bindings.extend(entity_bindings.clone().into_iter().collect_vec());

                    goal_cst.fill_in_bindings(&bindings).facts
                })
        })
        .collect()
}

/// Turn bindings into values in facts based on a binding map
/// Ignores bindings that don't exist in the binding map (they will stay undbound)
fn insert_bindings_into_facts(facts: &mut Vec<Fact<MkVal>>, bindings: &HashMap<String, Value>) {
    for fact in facts {
        fact.pattern.value.insert_binding_values(bindings);
        match &fact.pattern.entity_id {
            EntityPatternValue::Binding(b) if bindings.contains_key(b) && matches!(bindings[b], Value::EntityId(_)) => {
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

        let mk_val_bindings = extract_bindings_from_patterns(&mk_val.value.pattern(), &goal_fact.pattern.value.pattern());
        casual_model.bindings.extend(mk_val_bindings);

        if let EntityPatternValue::Binding(binding) = &mk_val.entity_id {
            // Same with entity id
            match &goal_fact.pattern.entity_id {
                EntityPatternValue::EntityId(entity_id) => {
                    casual_model
                        .bindings
                        .insert(binding.to_owned(), Value::EntityId(entity_id.clone()));
                }
                _ => {}
            }
        }

        casual_model.compute_backward_bindings();
        result.push(casual_model.clone());
    }

    result
}