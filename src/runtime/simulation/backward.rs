use crate::runtime::pattern_matching::{are_goals_equal, compare_imdls, compare_pattern_items, compare_patterns, extract_bindings_from_pattern, extract_bindings_from_patterns, extract_duplicate_bindings_from_pattern, extract_duplicate_bindings_from_pattern_and_values};
use crate::runtime::utils::{all_assumption_models, all_causal_models, all_req_models, all_state_prediction_models, MODEL_CONFIDENCE_THRESHOLD};
use crate::types::cst::{Cst, ICst};
use crate::types::models::{AbductionResult, IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::PatternItem;
use crate::types::runtime::System;
use crate::types::value::Value;
use crate::types::{EntityDeclaration, EntityPatternValue, Fact, MkVal, TimePatternRange};
use itertools::Itertools;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

const MAX_DEPTH: usize = 7;

pub fn backward_chain(goal: &Fact<MkVal>, data: &System) -> Vec<(IMdl, usize)> {
    let mut instantiable_cas_mdl = Vec::new();

    let req_models = all_req_models(data);
    for m_req in &req_models {
        for bound_m_req in m_req.try_instantiate_with_icst(&data.current_state) {
            let imdl = m_req.right.pattern.as_filled_in_imdl(&bound_m_req.bindings);
            instantiable_cas_mdl.push(imdl.clone());
        }
    }

    let mut casual_models = all_causal_models(data);
    casual_models.retain(|m| m.confidence() > MODEL_CONFIDENCE_THRESHOLD && m.success_count > 1);

    
    let mut state_prediction_models = all_assumption_models(data);
    state_prediction_models.extend(all_state_prediction_models(data));
    
    let mut observed_goals = HashSet::new();
    let mut observed_csts = HashMap::new();
    let goal_req_model_results = run_get_goal_requirements_for_goal(
        &goal,
        &instantiable_cas_mdl,
        &casual_models,
        &state_prediction_models,
        data,
        &mut observed_goals,
        &mut observed_csts,
    );
    
    goal_req_model_results
        .into_iter()
        // Remove duplicate results
        .unique()
        .collect()
}

pub fn run_get_goal_requirements_for_goal(
    goal: &Fact<MkVal>,
    instantiable_cas_mdl: &Vec<IMdl>,
    casual_models: &Vec<Mdl>,
    assumption_models: &Vec<Mdl>,
    data: &System,
    observed_goals: &mut HashSet<ObservedGoal>,
    observed_csts: &mut HashMap<ObservedCst, usize>,
) -> Vec<(IMdl, usize)> {
    let mut all_goal_requirements: Vec<(IMdl, usize)> = Vec::new();
    let mut queue: VecDeque<(Fact<MkVal>, usize)> = VecDeque::new();

    queue.push_back((goal.clone(), 0));

    while let Some((current_goal, depth)) = queue.pop_front() {
        let (mut goal_requirements, sub_goals, _) = get_goal_requirements_for_goal(
            &current_goal,
            instantiable_cas_mdl,
            casual_models,
            assumption_models,
            data,
            observed_goals,
            observed_csts,
            depth,
        );

        all_goal_requirements.append(&mut goal_requirements);
        if depth < MAX_DEPTH {
            for sub_goal in sub_goals {
                queue.push_back((sub_goal, depth + 1));
            }
        }
    }

    all_goal_requirements
}

/// The recursive part of backward chaining
/// TODO: Turn observed goals into hashmap to have depth as value of key
/// TODO: Modify into bfs (probably some kind of accumulate states to visit), and store depth that states were found at
/// TODO: then use the depth in forward chaining to prioritize the shortest path
fn get_goal_requirements_for_goal(
    goal: &Fact<MkVal>,
    instantiable_cas_mdl: &Vec<IMdl>,
    casual_models: &Vec<Mdl>,
    assumption_models: &Vec<Mdl>,
    data: &System,
    observed_goals: &mut HashSet<ObservedGoal>,
    observed_csts: &mut HashMap<ObservedCst, usize>,
    depth: usize,
) -> (Vec<(IMdl, usize)>, Vec<Fact<MkVal>>, bool) {
    if depth >= MAX_DEPTH {
        return (Vec::new(), Vec::new(), false);
    }

    let mut reached_current_state = false;
    let mut goal_requirements: Vec<(IMdl, usize)> = Vec::new();
    let mut subgoals = Vec::new();

    let abduction_results = casual_models
        .iter()
        .chain(assumption_models.iter())
        // Find and backward chain from all casual models where rhs matches a fact from the goal
        .filter_map(|m| {
            let bm = m.as_bound_model();
            bm.abduce(&goal.with_pattern(MdlRightValue::MkVal(goal.pattern.clone())), data)
        });

    for abduction_result in abduction_results {
        let goal_model_imdl = match &abduction_result {
            AbductionResult::SubGoal(_, _, imdl, _) | AbductionResult::IMdl(imdl) => imdl
        };

        // If the causal model can be reached directly from the current state, then we don't have to look further back
        if instantiable_cas_mdl.iter().any(|imdl_val| {
            compare_imdls(imdl_val, &goal_model_imdl, true, true)
        }) {
            reached_current_state = true;
            goal_requirements.push((goal_model_imdl.clone(), depth));
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
            goal_requirements.push((goal_model_imdl.clone(), depth));
        }

        let sub_goals = match abduction_result {
            AbductionResult::SubGoal(sub_goal, cst_id, _, has_bound_values) => {
                vec![(sub_goal, cst_id, goal_model_bm.model.clone(), has_bound_values)]
            }
            AbductionResult::IMdl(imdl) => {
                let imdl_fact = Fact::new(MdlRightValue::IMdl(imdl.clone()), TimePatternRange::wildcard());
                all_req_models(data)
                    .into_iter()
                    .filter_map(|m| Some((m.as_bound_model().abduce(&imdl_fact, data)?, m)))
                    .map(|(res, m)| match res {
                        AbductionResult::SubGoal(sub_goal, cst_id, _, icst) => (sub_goal, cst_id, m, icst),
                        AbductionResult::IMdl(imdl) => {
                            panic!("Model chain is too long, got {imdl} when expecting subgoal (icst or mk.val) lhs");
                        }
                    })
                    .collect()
            }
        };

        // Create sub goals from the requirement models
        for (sub_goal, sub_goal_cst_id, req_model, sub_goal_icst) in sub_goals {
            // (Performance optimizations) Skip csts that have been seen before
            if sub_goal_icst
                .as_ref()
                .map(|icst| observed_csts.get(&ObservedCst(icst.clone())))
                .flatten()
                .map(|observed_cst_depth| *observed_cst_depth <= depth)
                .unwrap_or(false) {
                continue;
            }
            if let Some(icst) = sub_goal_icst {
                observed_csts.insert(ObservedCst(icst), depth);
            }

            let sub_goal_entities = sub_goal_cst_id
                .map(|cst_id| &data.csts.get(&cst_id).unwrap().entities);

            let mut all_sub_goals = create_variations_of_sub_goal(&sub_goal, sub_goal_entities, data);
            // Only include the base subgoal if it has any concrete values, subgoals with only bindings are not useful
            if sub_goal.iter().any(|g| !g.pattern.is_value_fully_unbound()) {
                all_sub_goals.insert(0, sub_goal);
            }

            for sub_goal in all_sub_goals.into_iter().flatten() {
                // Don't check goals that have been seen before, to prevent an infinite loop
                // Re-check the observed goal if it was observed at a higher depth, since we may have reached the depth limit too early
                if matches!(observed_goals.get(&ObservedGoal::new(sub_goal.clone(), depth)), Some(g) if g.depth <= depth) {
                    continue;
                }
                let new_observed_goal = ObservedGoal::new(sub_goal.clone(), depth);
                observed_goals.remove(&new_observed_goal);
                observed_goals.insert(new_observed_goal);
                subgoals.push(sub_goal);
            }
        }
    }

    (goal_requirements, subgoals, reached_current_state)
}

/// Create variations of the subgoal with possible binding assignments
/// E.g. if the goal is for a hand and obj to both be at pos p, then two subgoals will be made,
/// one with p as the current pos of the hand and another with p as the current pos of the obj
pub(super) fn create_variations_of_sub_goal(
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
        success_count: 0,
        failure_count: 0,
    };
    let mut possible_entity_binding = goal_cst.all_possible_entity_bindings(system);
    // Possible entity bindings can be zero if there are no entity declarations in cst
    if possible_entity_binding.is_empty() {
        possible_entity_binding.push(HashMap::new());
    }

    possible_entity_binding
        .iter()
        .flat_map(|entity_bindings| {
            goal
                .iter()
                .permutations(goal.len())
                .map(|facts| create_binding_variation_for_fact_order(&facts, entity_bindings, system))
                .unique()
        })
        .unique()
        .map(|bindings| {
            let binding_map = bindings.into_iter().collect();
            goal_cst.fill_in_bindings(&binding_map).facts
        })
        .collect()
}

// Given the chosen order of facts, select binding values by filling in the bindings of each fact one by one until all bindings have a value
fn create_binding_variation_for_fact_order(facts: &Vec<&Fact<MkVal>>, entity_bindings: &HashMap<String, Value>, system: &System) -> Vec<(String, Value)> {
    let mut binding_map = entity_bindings.clone();
    for f in facts {
        let Some(key) = f.pattern.entity_key(&entity_bindings) else {
            continue;
        };
        let Some(sys_value) = system
            .current_state
            .variables
            .get(&key) else {
            continue;
        };
        // Extract and insert all bindings that are not already in the binding map
        for (b, v) in extract_duplicate_bindings_from_pattern_and_values(&f.pattern.value.pattern(), &vec![sys_value.clone()]) {
            if !binding_map.contains_key(&b) {
                binding_map.insert(b, v);
            }
        }
    }

    binding_map.into_iter().collect()
}

#[derive(Clone)]
struct ObservedGoal {
    goal: Fact<MkVal>,
    depth: usize,
}

impl ObservedGoal {
    pub fn new(goal: Fact<MkVal>, depth: usize) -> Self {
        Self { goal, depth }
    }
}

impl Hash for ObservedGoal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.goal.pattern.var_name.hash(state);
        matches!(self.goal.pattern.entity_id, EntityPatternValue::EntityId(_)).hash(state);
    }
}

impl PartialEq for ObservedGoal {
    fn eq(&self, other: &Self) -> bool {
        self.goal.pattern.matches_mk_val(&other.goal.pattern)
    }
}

impl Eq for ObservedGoal {}


#[derive(Clone)]
struct ObservedCst(ICst);

impl Hash for ObservedCst {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.cst_id.hash(state);
    }
}

impl PartialEq for ObservedCst {
    fn eq(&self, other: &Self) -> bool {
        self.0.cst_id == other.0.cst_id && self.0.params == other.0.params
    }
}

impl Eq for ObservedCst {}