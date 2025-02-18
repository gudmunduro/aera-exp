use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use crate::runtime::pattern_matching::{all_req_models, compare_imdls, state_matches_facts};
use crate::types::models::{BoundModel, IMdl};
use crate::types::pattern::bindings_in_pattern;
use crate::types::runtime::{RuntimeCommand, System, SystemState};
use crate::types::{Fact, MkVal};
use itertools::Itertools;

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct ForwardChainNode {
    pub command: RuntimeCommand,
    pub children: Vec<Rc<ForwardChainNode>>,
    pub is_in_goal_path: bool,
    pub child_count: u64,
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
    goal_requirements: &Vec<IMdl>,
    state: &SystemState,
    data: &System,
    observed_states: &mut HashSet<ObservedState>,
) -> (Vec<Rc<ForwardChainNode>>, bool, u64) {
    if state_matches_facts(state, goal) {
        return (Vec::new(), true, 0);
    }

    let mut results = Vec::new();
    let mut is_in_goal_path = false;
    let mut total_child_count = 0;

    let insatiable_req_models = all_req_models(data)
        .into_iter()
        .filter_map(|m| m.try_instantiate_with_icst(&state))
        .collect_vec();

    // Casual goal models with all bindings filled in form both forward and backward chaining
    let mut final_casual_models = Vec::new();
    for req_model in &insatiable_req_models {
        for casual_model in goal_requirements
            .iter()
            .filter(|cm| compare_imdls(cm, &req_model.model.right.pattern.as_filled_in_imdl(&req_model.bindings), true, true))
        {
            let fwd_chained_imdl = req_model
                .model
                .right
                .pattern
                .as_filled_in_imdl(&req_model.bindings);

            // Fill in bindings that we got from backward chaining but not forward chaining
            let merged_imdl = fwd_chained_imdl.merge_with(casual_model.clone());
            let mut fwd_chained_model = merged_imdl
                .instantiate(&req_model.bindings, data);
            fwd_chained_model.compute_forward_bindings();

            final_casual_models.push(fwd_chained_model);
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
                // If the node for this state has been computed, then add it since we may have found an alternative (potentially better) path to it
                // If it has not been computed, then we have most likely found a cycle in the graph
                if let Some(node) = observed_states
                        .get(&observed_state)
                        .map(|s| s.node.as_ref())
                        .flatten() {
                    results.push(node.clone());
                    if node.is_in_goal_path {
                        is_in_goal_path = true;
                    }
                    total_child_count += node.child_count + 1;
                }

                continue;
            }
            observed_states.insert(observed_state);

            let (children, is_goal_path, child_count) =
                forward_chain(goal, goal_requirements, &next_state, data, observed_states);
            if is_goal_path {
                is_in_goal_path = true;
            }
            total_child_count += child_count + 1;

            let node = Rc::new(ForwardChainNode {
                command,
                children,
                is_in_goal_path: is_goal_path,
                child_count,
            });
            observed_states.insert(ObservedState::new(next_state.clone(), Some(node.clone())));
            results.push(node);
        };
    }

    (results, is_in_goal_path, total_child_count)
}
