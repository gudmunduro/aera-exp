use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::string::ToString;
use std::time::Instant;
use crate::runtime::pattern_matching::{compare_imdls, state_matches_facts};
use crate::types::models::{BoundModel, IMdl, MdlLeftValue, MdlRightValue};
use crate::types::runtime::{RuntimeCommand, System, SystemState};
use crate::types::{Command, EntityPatternValue, EntityVariableKey, Fact, MkVal, TimePatternRange};
use itertools::Itertools;
use crate::runtime::utils::all_req_models;
use crate::types::cst::BoundCst;
use crate::types::pattern::PatternItem;
use crate::types::value::Value;
use crate::visualize::visualize_forward_chaining;

const MAX_FWD_CHAIN_DEPTH: u64 = 6;
const TIME_LIMIT_SECS: u64 = 60*10;

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct ForwardChainNode {
    pub command: RuntimeCommand,
    pub children: Vec<Rc<ForwardChainNode>>,
    pub is_in_goal_path: bool,
    pub min_goal_depth: u64,
}

#[derive(Debug, Clone)]
pub struct ForwardChainState {
    pub observed_states: HashSet<ObservedState>,
    pub min_solution_depth: u64,
    pub solution_found: bool,
    pub start_time: Instant,
}

impl ForwardChainState {
    pub fn new() -> ForwardChainState {
        ForwardChainState {
            observed_states: HashSet::new(),
            min_solution_depth: MAX_FWD_CHAIN_DEPTH,
            solution_found: false,
            start_time: Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObservedState {
    pub state: SystemState,
    pub node: Option<Rc<ForwardChainNode>>,
    pub reachable_from_depth: u64,
    pub is_goal_reachable: bool,
}

impl ObservedState {
    pub fn new(state: SystemState, node: Option<Rc<ForwardChainNode>>, reachable_from_depth: u64, is_goal_reachable: bool) -> Self {
        Self {
            state,
            node,
            reachable_from_depth,
            is_goal_reachable,
        }
    }
}

impl Hash for ObservedState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut variables = self.state.variables.iter().collect_vec();
        variables.sort_by_key(|(e, _)| (&e.entity_id, &e.var_name));
        variables.hash(state);
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
        forward_chain_state.solution_found = true;
        return (Vec::new(), true, 0);
    }
    if depth >= forward_chain_state.min_solution_depth {
        return (Vec::new(), false, u64::MAX);
    }
    if forward_chain_state.solution_found && forward_chain_state.start_time.elapsed().as_secs() > TIME_LIMIT_SECS {
        return (Vec::new(), false, u64::MAX);
    }

    let mut results = Vec::new();
    let mut is_in_goal_path = false;
    let mut node_min_goal_depth = u64::MAX;

    // Get all casual models that can be instantiated with forward chaining
    let fwd_chained_casual_models = compute_instantiate_casual_models(state, system);

    let (insatiable_casual_models, final_casual_models)
        = compute_merged_forward_backward_models(&fwd_chained_casual_models, goal_requirements, system);

    for casual_model in final_casual_models.iter().sorted_by_key(|m| -(m.model.confidence * 100.0) as i32) {
        if let Some(command) = casual_model
            .get_casual_model_command(&insatiable_casual_models, &system)
            .map(|c| c.to_runtime_command(&casual_model.bindings).ok())
            .flatten()
        {
            let Some(next_state) = casual_model.predict_state_change(
                &state,
                &fwd_chained_casual_models.iter().filter(|(_, anti)| *anti).map(|(imdl, _)| imdl).collect(),
                &insatiable_casual_models,
                system,
            ) else {
              continue;
            };

            // Don't look at next state if prediction changes nothing or if we have already seen this state
            let observed_state = ObservedState::new(next_state.clone(), None, depth, false);
            if state == &next_state {
                continue;
            }
            if let Some(existing_observed_state) = forward_chain_state.observed_states.get(&observed_state) {
                // Re-evaluate this state if we reached it at a lower depth, since goal paths could have been skipped due to depth limit
                if existing_observed_state.reachable_from_depth <= depth || existing_observed_state.is_goal_reachable {
                    // If the node for this state has been computed, then add it since we may have found an alternative (potentially better) path to it
                    // If it has not been computed, then we have most likely found a cycle in the graph
                    if let Some(node) = existing_observed_state.node.as_ref() {
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

            let new_observed_state = ObservedState::new(next_state.clone(), Some(node.clone()), depth, is_goal_path);
            forward_chain_state.observed_states.remove(&new_observed_state);
            forward_chain_state.observed_states.insert(new_observed_state);
            results.push(node);
        };
    }

    (results, is_in_goal_path, node_min_goal_depth)
}

pub(super) fn compute_merged_forward_backward_models(fwd_chained_casual_models: &Vec<(IMdl, bool)>, goal_requirements: &Vec<IMdl>, system: &System) -> (Vec<BoundModel>, Vec<BoundModel>) {
    let mut insatiable_casual_models = Vec::new();
    // Casual goal models with all bindings filled in form both forward and backward chaining
    let mut final_casual_models = Vec::new();
    for (fwd_chained_imdl, is_anti_fact) in fwd_chained_casual_models {
        if *is_anti_fact {
            continue;
        }

        // Get backward chained casual models
        for casual_model in goal_requirements
            .iter()
            .filter(|cm| compare_imdls(cm, &fwd_chained_imdl, true, true))
        {
            // Fill in bindings that we got from backward chaining but not forward chaining
            let merged_imdl = fwd_chained_imdl.clone().merge_with(casual_model.clone());
            let mut fwd_chained_model = merged_imdl
                .instantiate(&HashMap::new(), system);

            // There might be a better way to do this,
            // but currently this is the first time that all backward guards can be computed,
            // since some of them need bindings that we get from forward chaining
            fwd_chained_model.compute_backward_bindings();
            fwd_chained_model.compute_forward_bindings();

            final_casual_models.push(fwd_chained_model);
        }

        // Create a list of all instantiable casual models
        let casual_model = fwd_chained_imdl.instantiate(&HashMap::new(), system);
        insatiable_casual_models.push(casual_model);
    }

    (insatiable_casual_models, final_casual_models)
}

pub(super) fn compute_instantiate_casual_models(state: &SystemState, system: &System) -> Vec<(IMdl, bool)> {
    let instantiated_composite_states = state.instansiated_csts
        .iter()
        .flat_map(|(_, csts)| csts.iter().map(BoundCst::icst_for_cst))
        .collect_vec();

    all_req_models(system)
        .into_iter()
        .flat_map(|m| {
            let bm = m.as_bound_model();
            instantiated_composite_states
                .iter()
                .filter_map(|icst| {
                    bm.deduce(&Fact::new(MdlLeftValue::ICst(icst.clone()), TimePatternRange::wildcard()), &Vec::new())
                })
                .collect_vec()
        })
        .map(|rhs| match rhs.pattern {
            MdlRightValue::IMdl(imdl) => (imdl, rhs.anti),
            MdlRightValue::MkVal(_) => {
                panic!("Rhs of requirement model cannot be mk.val")
            }
        })
        .collect_vec()
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

pub fn predict_all_changes_of_command(command: &RuntimeCommand, system: &System) -> Vec<(EntityVariableKey, Value, IMdl)> {
    let lhs_cmd = Fact::new(MdlLeftValue::Command(command.to_command()), TimePatternRange::wildcard());
    let fwd_chained_casual_models = compute_instantiate_casual_models(&system.current_state, system);

    let anti_requirements = compute_instantiate_casual_models(&system.current_state, &system)
        .into_iter()
        .filter(|(_, anti)| *anti)
        .map(|(imdl, _)| imdl)
        .collect_vec();
    let anti_requirements_ref = anti_requirements
        .iter()
        .collect_vec();
    fwd_chained_casual_models
        .iter()
        .filter(|(_, anti)| !*anti)
        .filter_map(|(mdl, _)| {
            let bound_mdl = mdl.instantiate(&HashMap::new(), system);
            if bound_mdl.model.is_reuse_model() {
                let rhs = fwd_chained_casual_models
                    .iter()
                    .filter(|(_, anti)| !*anti)
                    .filter_map(|(mdl2, _)| {
                        let mdl2 = mdl2.instantiate(&HashMap::new(), &system).extend_bindings_with_lhs_input(&lhs_cmd)?.imdl_for_model();
                        bound_mdl.deduce(&Fact::new(MdlLeftValue::IMdl(mdl2), TimePatternRange::wildcard()), &anti_requirements_ref)
                    })
                    .next()?;
                Some((rhs, bound_mdl.imdl_for_model()))
            }
            else if bound_mdl.model.is_casual_model() {
                Some((bound_mdl.deduce(&lhs_cmd, &anti_requirements_ref)?, bound_mdl.imdl_for_model()))
            }
            else {
                None
            }
        })
        .filter_map(|(rhs, imdl)| match &rhs.pattern {
            MdlRightValue::MkVal(f) => Some(
                (
                    EntityVariableKey::new(&f.entity_id.get_id_with_bindings(&HashMap::new())?, &f.var_name),
                    f.value.get_value_with_bindings(&HashMap::new())?,
                    imdl
                )
            ),
            _ => None
        })
        .collect()
}