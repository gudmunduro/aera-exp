use crate::runtime::pattern_matching::{are_goals_equal, compare_imdls, extract_duplicate_bindings_from_pattern};
use crate::runtime::utils::{all_assumption_models, all_causal_models, all_req_models, all_state_prediction_models};
use crate::types::cst::Cst;
use crate::types::models::{AbductionResult, IMdl, Mdl, MdlRightValue};
use crate::types::pattern::PatternItem;
use crate::types::runtime::System;
use crate::types::value::Value;
use crate::types::{EntityDeclaration, Fact, MkVal, TimePatternRange};
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

    let abduction_results = casual_models
        .iter()
        .chain(assumption_models.iter())
        // Find and backward chain from all casual models where rhs matches a fact from the goal
        .flat_map(|m| {
            let bm = m.as_bound_model();
            goal.iter()
                .filter_map(|goal_val| {
                    bm.abduce(&goal_val.with_pattern(MdlRightValue::MkVal(goal_val.pattern.clone())), data)
                })
                .collect_vec()
        })
        .collect_vec();

    for abduction_result in abduction_results {
        let goal_model_imdl = match &abduction_result {
            AbductionResult::SubGoal(_, _, imdl) | AbductionResult::IMdl(imdl) => imdl
        };

        // If the casual model can be reached directly from the current state, then we don't have to look further back
        if instantiable_cas_mdl.iter().any(|imdl_val| {
            compare_imdls(imdl_val, &goal_model_imdl, true, true)
        }) {
            reached_current_state = true;
            goal_requirements.push(goal_model_imdl.clone());
            continue;
        }

        let goal_model_bm = goal_model_imdl.instantiate(&HashMap::new(), &data);

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
            goal_requirements.push(goal_model_imdl.clone());
        }

        let sub_goals = match abduction_result {
            AbductionResult::SubGoal(sub_goal, cst_id, _) => {
                vec![(sub_goal, cst_id)]
            }
            AbductionResult::IMdl(imdl) => {
                let imdl_fact = Fact::new(MdlRightValue::IMdl(imdl.clone()), TimePatternRange::wildcard());
                all_req_models(data)
                    .into_iter()
                    .filter_map(|m| m.as_bound_model().abduce(&imdl_fact, data))
                    .map(|res| match res {
                        AbductionResult::SubGoal(sub_goal, cst_id, _) => (sub_goal, cst_id),
                        AbductionResult::IMdl(imdl) => {
                            panic!("Model chain is too long, got {imdl} when expecting subgoal (icst or mk.val) lhs");
                        }
                    })
                    .collect()
            }
        };

        // Create sub goals from the requirement models
        for (sub_goal, sub_goal_cst_id) in sub_goals {
            let sub_goal_entities = sub_goal_cst_id
                .map(|cst_id| &data.csts.get(&cst_id).unwrap().entities);
            let mut all_sub_goals = create_variations_of_sub_goal(&sub_goal, sub_goal_entities, data);
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
    sub_goal_entities: Option<&Vec<EntityDeclaration>>,
    system: &System,
) -> Vec<Vec<Fact<MkVal>>> {
    let goal_cst = Cst {
        cst_id: "".to_string(),
        facts: goal.clone(),
        // Keep entity binding declarations for entities which have not been filled in
        entities: sub_goal_entities
            .unwrap_or(&Vec::new())
            .into_iter()
            .filter(|e| {
                goal.iter()
                    .any(|f| f.pattern.entity_id.is_binding(&e.binding))
            })
            .map(|e| e.to_owned())
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
