use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use crate::runtime::pattern_matching::{all_req_models, compare_imdls, state_matches_facts};
use crate::types::models::{IMdl};
use crate::types::runtime::{RuntimeCommand, System, SystemState};
use crate::types::{Fact, MkVal};
use itertools::Itertools;

const MAX_FWD_CHAIN_DEPTH: u64 = 20;

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct ForwardChainNode {
    pub command: RuntimeCommand,
    pub children: Vec<Rc<ForwardChainNode>>,
    pub is_in_goal_path: bool,
    pub min_goal_depth: u64,
}

#[derive(Debug, Clone)]
pub struct ObservedState {
    pub state: SystemState,
    pub node: Option<Rc<ForwardChainNode>>,
    pub reachable_from_depth: u64,
}

#[derive(Debug, Clone)]
pub struct ForwardChainState {
    pub observed_states: HashSet<ObservedState>,
    pub min_solution_depth: u64,
}

impl ForwardChainState {
    pub fn new() -> ForwardChainState {
        ForwardChainState {
            observed_states: HashSet::new(),
            min_solution_depth: MAX_FWD_CHAIN_DEPTH,
        }
    }
}

impl ObservedState {
    pub fn new(state: SystemState, node: Option<Rc<ForwardChainNode>>, reachable_from_depth: u64) -> Self {
        Self {
            state,
            node,
            reachable_from_depth,
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

pub fn forward_chain(goal: &Vec<Fact<MkVal>>, goal_requirements: &Vec<IMdl>, system: &System,) -> Vec<RuntimeCommand> {
    let (forward_chain_graph, _, _) = forward_chain_rec(goal, goal_requirements, &system.current_state, system, &mut ForwardChainState::new(), 0);
    let path = commit_to_path(&forward_chain_graph);
    path
}

fn forward_chain_rec(
    goal: &Vec<Fact<MkVal>>,
    goal_requirements: &Vec<IMdl>,
    state: &SystemState,
    system: &System,
    forward_chain_state: &mut ForwardChainState,
    depth: u64,
) -> (Vec<Rc<ForwardChainNode>>, bool, u64) {
    if state_matches_facts(state, goal) {
        forward_chain_state.min_solution_depth = forward_chain_state.min_solution_depth.min(depth);
        return (Vec::new(), true, 0);
    }
    if depth >= forward_chain_state.min_solution_depth {
        return (Vec::new(), false, u64::MAX);
    }

    let mut results = Vec::new();
    let mut is_in_goal_path = false;
    let mut node_min_goal_depth = u64::MAX;

    let insatiable_req_models = all_req_models(system)
        .into_iter()
        .flat_map(|m| m.try_instantiate_with_icst(&state))
        .collect_vec();

    let mut insatiable_casual_models = Vec::new();
    // Casual goal models with all bindings filled in form both forward and backward chaining
    let mut final_casual_models = Vec::new();
    for req_model in &insatiable_req_models {
        // Get backward chained casual models
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
                .instantiate(&req_model.bindings, system);

            // There might be a better way to do this,
            // but currently this is the first time that all backward guards can be computed,
            // since some of them need bindings that we get from forward chaining
            fwd_chained_model.compute_backward_bindings();
            fwd_chained_model.compute_forward_bindings();

            final_casual_models.push(fwd_chained_model);
        }

        // Create a list of all instantiable casual models
        let casual_model = req_model.model.right.pattern.as_imdl().instantiate(&req_model.bindings, &system);
        insatiable_casual_models.push(casual_model);
    }

    for casual_model in &final_casual_models {
        if let Ok(command) = casual_model
            .model
            .left
            .pattern
            .as_command()
            .to_runtime_command(&casual_model.bindings)
        {
            let other_casual_models = insatiable_casual_models
                .iter()
                .filter(|m| {
                    !(m.model.model_id == casual_model.model.model_id
                        && m.bindings == casual_model.bindings)
                })
                .collect();
            let Some(next_state) = casual_model.predict_state_change(
                &state,
                &other_casual_models,
                system,
            ) else {
              continue;
            };

            // Don't look at next state if prediction changes nothing or if we have already seen this state
            let observed_state = ObservedState::new(next_state.clone(), None, depth);
            if state == &next_state {
                continue;
            }
            if forward_chain_state.observed_states.contains(&observed_state) {
                // Re-evaluate this state if we reached it at a lower depth, since goal paths could have been skipped due to depth limit
                if forward_chain_state.observed_states.get(&observed_state).unwrap().reachable_from_depth <= depth {
                    // If the node for this state has been computed, then add it since we may have found an alternative (potentially better) path to it
                    // If it has not been computed, then we have most likely found a cycle in the graph
                    if let Some(node) = forward_chain_state.observed_states
                        .get(&observed_state)
                        .map(|s| s.node.as_ref())
                        .flatten() {
                        results.push(Rc::new(ForwardChainNode {
                            command,
                            children: node.children.clone(),
                            is_in_goal_path: node.is_in_goal_path,
                            min_goal_depth: node.min_goal_depth,
                        }));
                        if node.is_in_goal_path {
                            node_min_goal_depth = node_min_goal_depth.min(node.min_goal_depth.saturating_add(1));
                            is_in_goal_path = true;
                        }
                    }

                    continue;
                }

                // Remove the existing observed state to insert a new one with a new depth limit
                forward_chain_state.observed_states.remove(&observed_state);
            }
            forward_chain_state.observed_states.insert(observed_state);

            let (children, is_goal_path, min_goal_depth) =
                forward_chain_rec(goal, goal_requirements, &next_state, system, forward_chain_state, depth + 1);
            if is_goal_path {
                node_min_goal_depth = node_min_goal_depth.min(min_goal_depth.saturating_add(1));
                is_in_goal_path = true;
            }

            let node = Rc::new(ForwardChainNode {
                command,
                children,
                is_in_goal_path: is_goal_path,
                min_goal_depth: min_goal_depth.saturating_add(1),
            });

            let new_observed_state = ObservedState::new(next_state.clone(), Some(node.clone()), depth);
            forward_chain_state.observed_states.remove(&new_observed_state);
            forward_chain_state.observed_states.insert(new_observed_state);
            results.push(node);
        };
    }

    (results, is_in_goal_path, node_min_goal_depth)
}

fn commit_to_path(forward_chain_result: &Vec<Rc<ForwardChainNode>>) -> Vec<RuntimeCommand> {
    let Some(best_path_node) = forward_chain_result.iter()
        .sorted_by_key(|n| n.min_goal_depth)
        .filter(|n| n.is_in_goal_path)
        .next() else {
        return Vec::new();
    };

    if best_path_node.children.is_empty() {
        vec![best_path_node.command.clone()]
    } else {
        let mut path = commit_to_path(&best_path_node.children);
        path.insert(0, best_path_node.command.clone());
        path
    }

}
