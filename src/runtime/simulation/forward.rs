use itertools::Itertools;
use crate::runtime::pattern_matching::{all_req_models, state_matches_facts};
use crate::types::{Fact, MkVal};
use crate::types::models::BoundModel;
use crate::types::pattern::bindings_in_pattern;
use crate::types::runtime::{RuntimeCommand, RuntimeData, SystemState};

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct ForwardChainNode {
    command: RuntimeCommand,
    children: Vec<ForwardChainNode>,
    is_in_goal_path: bool,
}

pub fn forward_chain(
    goal: &Vec<Fact<MkVal>>,
    goal_requirements: &Vec<BoundModel>,
    state: &SystemState,
    data: &RuntimeData,
    observed_states: &mut Vec<SystemState>,
) -> (Vec<ForwardChainNode>, bool) {
    if state_matches_facts(state, goal) {
        return (Vec::new(), true);
    }

    let mut results = Vec::new();
    let mut is_in_goal_path = false;

    let insatiable_req_models = all_req_models(data)
        .into_iter()
        .filter_map(|m| m.try_instantiate_with_icst(&state))
        .collect_vec();

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

                if let Ok(command) = fwd_chained_model
                    .model
                    .left
                    .pattern
                    .as_command()
                    .to_runtime_command(&fwd_chained_model.bindings)
                {
                    let next_state = fwd_chained_model.predict_state_change(&state, data);
                    // Don't look at next state if prediction changes nothing or if we have already seen this state
                    if state == &next_state || observed_states.contains(&next_state) {
                        continue;
                    }
                    observed_states.push(next_state.clone());

                    let (children, is_goal_path) =
                        forward_chain(goal, goal_requirements, &next_state, data, observed_states);
                    if is_goal_path {
                        is_in_goal_path = true;

                        let node = ForwardChainNode {
                            command,
                            children,
                            is_in_goal_path: is_goal_path,
                        };
                        results.push(node);
                    }
                };
            }
        }
    }

    (results, is_in_goal_path)
}
