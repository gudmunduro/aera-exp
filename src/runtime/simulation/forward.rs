use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use crate::runtime::pattern_matching::{all_req_models, state_matches_facts};
use crate::types::models::BoundModel;
use crate::types::pattern::bindings_in_pattern;
use crate::types::runtime::{RuntimeCommand, System, SystemState};
use crate::types::{EntityVariableKey, Fact, MkVal};
use itertools::Itertools;

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct ForwardChainNode {
    pub command: RuntimeCommand,
    pub children: Vec<Rc<ForwardChainNode>>,
    pub is_in_goal_path: bool,
}

#[derive(Debug, Clone)]
pub struct ObservedState {
    pub state: SystemState,
    pub node: Option<Rc<ForwardChainNode>>,
}

impl ObservedState {
    pub fn new(state: SystemState, node: Option<Rc<ForwardChainNode>>) -> Self {
        Self {
            state,
            node
        }
    }
}

impl Hash for ObservedState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state.variables.iter().collect_vec().hash(state);
    }
}

impl PartialEq for ObservedState {
    fn eq(&self, other: &Self) -> bool {
        self.state.variables == other.state.variables
    }
}

impl Eq for ObservedState {}

pub fn forward_chain(
    goal: &Vec<Fact<MkVal>>,
    goal_requirements: &Vec<BoundModel>,
    state: &SystemState,
    data: &System,
    observed_states: &mut HashSet<ObservedState>,
) -> (Vec<Rc<ForwardChainNode>>, bool) {
    if state_matches_facts(state, goal) {
        return (Vec::new(), true);
    }

    let mut results = Vec::new();
    let mut is_in_goal_path = false;

    let insatiable_req_models = all_req_models(data)
        .into_iter()
        .filter_map(|m| m.try_instantiate_with_icst(&state))
        .collect_vec();

    // Casual goal models with all bindings filled in form both forward and backward chaining
    let mut final_casual_models = Vec::new();
    for req_model in &insatiable_req_models {
        for casual_model in goal_requirements
            .iter()
            .filter(|cm| cm.model.model_id == req_model.model.right.pattern.as_imdl().model_id)
        {
            // 2.1
            let backward_chained_model = req_model
                .model
                .backward_chain_known_bindings_from_imdl(casual_model);

            // Can req_model instantiate casual_model with equal values
            // We only need to look at known bindings from req_model that appear in the pattern
            // (unknown bindings that appear only in rhs can be filled in with backward chaining)
            let pattern_bindings =
                bindings_in_pattern(&req_model.model.right.pattern.as_imdl().params);
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
                fwd_chained_model.fill_missing_bindings(&casual_model.bindings);
                fwd_chained_model.compute_forward_bindings();

                final_casual_models.push(fwd_chained_model);
            }
        }
    }

    for casual_model in &final_casual_models {
        if let Ok(command) = casual_model
            .model
            .left
            .pattern
            .as_command()
            .to_runtime_command(&casual_model.bindings)
        {
            let other_casual_models = final_casual_models
                .iter()
                .filter(|m| {
                    !(m.model.model_id == casual_model.model.model_id
                        && m.bindings == casual_model.bindings)
                })
                .collect();
            let next_state = casual_model.predict_state_change(
                &state,
                &other_casual_models,
                data,
            );
            // Don't look at next state if prediction changes nothing or if we have already seen this state
            let observed_state = ObservedState::new(next_state.clone(), None);
            if state == &next_state || observed_states.contains(&observed_state) {
                let saved_observed_state = observed_states.get(&observed_state).unwrap();
                // If the node for this state has been computed, then add it since we may have found an alternative (potentially better) path to it
                // If it has not been computed, then we have most likely found a cycle in the graph
                if let Some(node) = saved_observed_state.node.as_ref() {
                    results.push(node.clone());
                    if node.is_in_goal_path {
                        is_in_goal_path = true;
                    }
                }

                continue;
            }
            observed_states.insert(observed_state);

            let (children, is_goal_path) =
                forward_chain(goal, goal_requirements, &next_state, data, observed_states);
            if is_goal_path {
                is_in_goal_path = true;
            }

            let node = Rc::new(ForwardChainNode {
                command,
                children,
                is_in_goal_path: is_goal_path,
            });
            observed_states.insert(ObservedState::new(next_state.clone(), Some(node.clone())));
            results.push(node);
        };
    }

    (results, is_in_goal_path)
}
