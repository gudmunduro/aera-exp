use crate::runtime::pattern_matching::{combine_pattern_bindings, compare_patterns, compute_assumptions, compute_instantiated_states, compute_state_predictions, extract_bindings_from_patterns, fill_in_pattern_with_bindings, PatternMatchResult};
use crate::types::cst::ICst;
use crate::types::functions::Function;
use crate::types::pattern::{
    bindings_in_pattern, Pattern,
};
use crate::types::runtime::{System, SystemState};
use crate::types::value::Value;
use crate::types::{Command, EntityVariableKey, Fact, MkVal, PatternItem};
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use tap::Tap;

#[derive(Clone, Debug)]
pub struct Mdl {
    pub model_id: String,
    pub left: Fact<MdlLeftValue>,
    pub right: Fact<MdlRightValue>,
    pub confidence: f64,
    pub forward_computed: Vec<(String, Function)>,
    pub backward_computed: Vec<(String, Function)>,
}

impl Mdl {
    pub fn binding_param(&self) -> Vec<String> {
        let left_pattern = match &self.left.pattern {
            MdlLeftValue::ICst(cst) => bindings_in_pattern(&cst.params),
            MdlLeftValue::Command(cmd) => cmd.get_bindings(),
            MdlLeftValue::MkVal(mk_val) => mk_val.get_bindings(),
        };
        let params_in_computed = self
            .forward_computed
            .iter()
            .flat_map(|(_, f)| f.binding_params());
        let right_params = match &self.right.pattern {
            MdlRightValue::IMdl(imdl) => bindings_in_pattern(&imdl.params),
            MdlRightValue::MkVal(mk_val) => mk_val.get_bindings(),
        };

        left_pattern
            .into_iter()
            .chain(params_in_computed.into_iter())
            .chain(right_params)
            // Bindings assigned to function results cannot be passes as parameters
            .filter(|b| !self.forward_computed.iter().any(|(k, _)| k == b))
            .unique()
            .collect()
    }

    pub fn fwd_guard_params(&self) -> Vec<&str> {
        self.forward_computed.iter().map(|(p, _)| p.as_str()).collect()
    }

    /// Attempt to instantiate this model using the lhs icst instruction
    pub fn try_instantiate_with_icst(&self, state: &SystemState) -> Vec<BoundModel> {
        let icst = match &self.left.pattern {
            MdlLeftValue::ICst(icst) => icst,
            _ => return Vec::new(),
        };
        let mut results = Vec::new();
        for instantiated_cst in state
            .instansiated_csts
            .get(&icst.cst_id)
            .unwrap_or(&Vec::new())
        {
            match instantiated_cst.match_and_get_bindings_for_icst(&icst) {
                PatternMatchResult::True(bindings) => {
                    results.push(
                        BoundModel {
                            bindings,
                            model: self.clone(),
                        }
                        .tap_mut(|m| m.compute_forward_bindings()),
                    );
                }
                PatternMatchResult::False => continue,
            }
        }

        results
    }

    /// Get a bound version of this model from rhs imdl
    pub fn backward_chain_known_bindings_from_imdl(&self, imdl: &IMdl) -> BoundModel {
        let self_imdl = self.right.pattern.as_imdl();
        let bindings = extract_bindings_from_patterns(&self_imdl.params, &imdl.params);

        BoundModel {
            model: self.clone(),
            bindings,
        }
        .tap_mut(|m| m.compute_backward_bindings())
    }

    pub fn is_casual_model(&self) -> bool {
        match self {
            Mdl {
                left:
                    Fact {
                        pattern: MdlLeftValue::Command(_),
                        ..
                    },
                right:
                    Fact {
                        pattern: MdlRightValue::MkVal(_),
                        ..
                    },
                ..
            } => true,
            _ => false,
        }
    }

    pub fn is_req_model(&self) -> bool {
        match self {
            Mdl {
                left:
                    Fact {
                        pattern: MdlLeftValue::ICst(_),
                        ..
                    },
                right:
                    Fact {
                        pattern: MdlRightValue::IMdl(_),
                        ..
                    },
                ..
            } => true,
            _ => false,
        }
    }

    pub fn is_assumption_model(&self) -> bool {
        match self {
            Mdl {
                left:
                    Fact {
                        pattern: MdlLeftValue::ICst(_),
                        ..
                    },
                right:
                    Fact {
                        pattern:
                            MdlRightValue::MkVal(MkVal {
                                assumption: true, ..
                            }),
                        ..
                    },
                ..
            } => true,
            _ => false,
        }
    }

    pub fn is_state_prediction(&self) -> bool {
        match self {
            Mdl {
                left:
                    Fact {
                        pattern: MdlLeftValue::ICst(_),
                        ..
                    },
                right:
                    Fact {
                        pattern:
                            MdlRightValue::MkVal(MkVal {
                                assumption: false, ..
                            }),
                        ..
                    },
                ..
            } => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum MdlLeftValue {
    ICst(ICst),
    Command(Command),
    MkVal(MkVal),
}

impl MdlLeftValue {
    pub fn as_icst(&self) -> &ICst {
        match self {
            MdlLeftValue::ICst(icst) => icst,
            _ => panic!("Lhs needs to be icst in model"),
        }
    }

    pub fn as_command(&self) -> &Command {
        match self {
            MdlLeftValue::Command(cmd) => cmd,
            _ => panic!("Lhs needs to be a command in model"),
        }
    }

    pub fn as_mk_val(&self) -> &MkVal {
        match self {
            MdlLeftValue::MkVal(mk_val) => mk_val,
            _ => panic!("Lhs needs to be mk.val in model"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum MdlRightValue {
    IMdl(IMdl),
    MkVal(MkVal),
}

impl MdlRightValue {
    pub fn as_imdl(&self) -> &IMdl {
        match self {
            MdlRightValue::IMdl(imdl) => imdl,
            _ => panic!("Rhs needs to be imdl in model"),
        }
    }

    pub fn as_filled_in_imdl(&self, bindings: &HashMap<String, Value>) -> IMdl {
        let mut imdl = self.as_imdl().clone();
        imdl.params = fill_in_pattern_with_bindings(imdl.params, bindings);
        imdl
    }

    pub fn as_mk_val(&self) -> &MkVal {
        match self {
            MdlRightValue::MkVal(mk_val) => mk_val,
            _ => panic!("Rhs needs to be mk.val in model"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IMdl {
    pub model_id: String,
    pub params: Pattern,
    pub fwd_guard_bindings: HashMap<String, Value>,
}

impl IMdl {
    pub fn new(model_id: String, params: Pattern) -> IMdl {
        IMdl {
            model_id,
            params,
            fwd_guard_bindings: HashMap::new(),
        }
    }

    pub fn with_fwd_guards(model_id: String, params: Pattern, fwd_guard_params: HashMap<String, Value>) -> IMdl {
        IMdl {
            model_id,
            params,
            fwd_guard_bindings: fwd_guard_params,
        }
    }

    pub fn map_bindings_to_model(
        &self,
        bindings: &HashMap<String, Value>,
        data: &System,
    ) -> HashMap<String, Value> {
        let model = data.models.get(&self.model_id).unwrap();
        model
            .binding_param()
            .iter()
            .zip(&self.params)
            .filter_map(|(binding_name, value)| {
                value
                    .get_value_with_bindings(bindings)
                    .map(|v| (binding_name.clone(), v))
            })
            .collect()
    }

    pub fn instantiate(&self, bindings: &HashMap<String, Value>, data: &System) -> BoundModel {
        let model = data
            .models
            .get(&self.model_id)
            .expect(&format!("Model in imdl does not exist {}", self.model_id))
            .clone();
        let mut bindings = self.map_bindings_to_model(bindings, data);
        bindings.extend(self.fwd_guard_bindings.clone());

        BoundModel { bindings, model }.tap_mut(|m| m.compute_forward_bindings())
    }

    pub fn merge_with(mut self, imdl: IMdl) -> IMdl {
        self.params = combine_pattern_bindings(self.params, imdl.params);
        self.fwd_guard_bindings.extend(imdl.fwd_guard_bindings);
        self
    }
}

// Hacky Eq implementation using partial eq function
impl Eq for IMdl {
}

impl Hash for IMdl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.model_id.hash(state);
        self.params.hash(state);
        self.fwd_guard_bindings.iter().collect_vec().hash(state);
    }
}


impl Display for IMdl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(imdl {} {} | [{}])",
            self.model_id,
            self.params.iter().map(|p| p.to_string()).join(" "),
            self.fwd_guard_bindings.iter().map(|(b, v)| format!("{b}: {v}")).join(", ")
        )?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BoundModel {
    pub bindings: HashMap<String, Value>,
    pub model: Mdl,
}

impl BoundModel {
    /// Create imdl that would be used to instantiate this model, including known binding values.
    /// Can be used to compare it to imdl on rhs of req. models
    pub fn imdl_for_model(&self) -> IMdl {
        let binding_params = self.model.binding_param();
        let param_pattern = binding_params
            .iter()
            .map(|b| match self.bindings.get(b) {
                Some(v) => PatternItem::Value(v.clone()),
                // There is no specific binding in the imdl, since those depend on the model the imdl is in
                None => PatternItem::Any,
            })
            .collect();

        let fwd_guard_bindings = self.model.fwd_guard_params()
            .into_iter()
            .filter_map(|b| Some((b.to_owned(), self.bindings.get(b)?.clone())))
            .collect();

        IMdl::with_fwd_guards(self.model.model_id.clone(), param_pattern, fwd_guard_bindings)
    }

    /// Predict what happens to SystemState after model is executed
    /// Only meant to be used for casual models, has no effect on other types of models
    /// At the moment this does not take into account changes that other models predict will take place when the same action is taken
    pub fn predict_state_change(
        &self,
        state: &SystemState,
        other_casual_models: &Vec<&BoundModel>,
        system: &System,
    ) -> Option<SystemState> {
        let MdlLeftValue::Command(cmd) = &self.model.left.pattern else {
            return None;
        };
        let MdlRightValue::MkVal(mk_val) = &self.model.right.pattern else {
            return None;
        };
        let Some(predicted_value) = mk_val
            .value
            .get_value_with_bindings(&self.bindings) else {
            return None;
        };
        let mut cmd = cmd.clone();
        cmd.params = fill_in_pattern_with_bindings(cmd.params, &self.bindings);

        // Find all other models that predict a state change based on the exact same command, and use those predictions as well
        let other_predicted_changes = other_casual_models
            .iter()
            .filter_map(|m| {
                let mut other_cmd = m.model.left.pattern.as_command().clone();
                other_cmd.params = fill_in_pattern_with_bindings(other_cmd.params, &m.bindings);

                let cmd_name_match = cmd.name == other_cmd.name;
                let params_match = compare_patterns(&cmd.params, &other_cmd.params, true, true);
                let entity_id_match = match (
                    cmd.entity_id.get_id_with_bindings(&self.bindings),
                    other_cmd.entity_id.get_id_with_bindings(&m.bindings),
                ) {
                    (Some(e1), Some(e2)) => e1 == e2,
                    (None, _) | (_, None) => true,
                };

                if cmd_name_match && params_match && entity_id_match {
                    let mut m = (*m).clone();
                    // Get bindings from the command in the "main" model and add them to this model
                    let additional_bindings = extract_bindings_from_patterns(
                        &other_cmd.params,
                        &cmd.params,
                    );
                    m.bindings.extend(additional_bindings);

                    m.compute_forward_bindings();

                    Some(m)
                } else {
                    None
                }
            })
            .filter_map(|m| {
                let mk_val = m.model.right.pattern.as_mk_val();
                let entity_id = mk_val.entity_id.get_id_with_bindings(&m.bindings)?;
                let value = mk_val.value.get_value_with_bindings(&m.bindings)?;
                Some((
                    EntityVariableKey {
                        entity_id,
                        var_name: mk_val.var_name.to_owned(),
                    },
                    value,
                ))
            })
            .collect_vec();

        // TODO: Also look at state prediction models

        let mut new_state = state.clone();
        new_state.variables.extend(other_predicted_changes);
        new_state.variables.insert(
            EntityVariableKey::new(
                &mk_val
                    .entity_id
                    .get_id_with_bindings(&self.bindings)
                    .expect("Entity binding missing when performing state change"),
                &mk_val.var_name,
            ),
            predicted_value,
        );
        new_state.instansiated_csts = compute_instantiated_states(system, &new_state);
        new_state
            .variables
            .extend(compute_state_predictions(&system, &new_state));
        new_state
            .variables
            .extend(compute_assumptions(&system, &new_state));
        // Compute instantiated csts again, now with assumption variables
        new_state.instansiated_csts = compute_instantiated_states(system, &new_state);

        Some(new_state)
    }

    /// Add bindings from `bindings` for variables which were not bound before
    pub fn fill_missing_bindings(&mut self, bindings: &HashMap<String, Value>) {
        self.bindings.extend(
            bindings
                .iter()
                .filter(|(b, _)| !self.bindings.contains_key(*b))
                .map(|(b, v)| (b.clone(), v.clone()))
                .collect_vec(),
        )
    }

    pub fn compute_forward_bindings(&mut self) {
        for (binding, function) in &self.model.forward_computed {
            if let Some(res) = function.evaluate(&self.bindings) {
                self.bindings.insert(binding.clone(), res);
            }
        }
    }

    pub fn compute_backward_bindings(&mut self) {
        for (binding, function) in &self.model.backward_computed {
            if let Some(res) = function.evaluate(&self.bindings) {
                self.bindings.insert(binding.clone(), res);
            }
        }
    }
}
