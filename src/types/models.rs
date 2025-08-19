use crate::runtime::pattern_matching::{combine_pattern_bindings, compare_imdls, compare_patterns, extract_bindings_from_patterns, fill_in_pattern_with_bindings, PatternMatchResult};
use crate::types::cst::ICst;
use crate::types::functions::Function;
use crate::types::pattern::{
    bindings_in_pattern, Pattern,
};
use crate::types::runtime::{System, SystemState};
use crate::types::value::Value;
use crate::types::{Command, EntityVariableKey, Fact, MkVal, PatternItem, TimePatternRange};
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use tap::Tap;
use serde::{Deserialize, Serialize};
use crate::runtime::utils::{compute_assumptions, compute_instantiated_states, compute_state_predictions};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mdl {
    pub model_id: String,
    pub left: Fact<MdlLeftValue>,
    pub right: Fact<MdlRightValue>,
    pub confidence: f64,
    pub success_count: usize,
    pub forward_computed: Vec<(String, Function)>,
    pub backward_computed: Vec<(String, Function)>,
}

impl Mdl {
    pub fn binding_param(&self) -> Vec<String> {
        let left_pattern = match &self.left.pattern {
            MdlLeftValue::ICst(cst) => bindings_in_pattern(&cst.params),
            MdlLeftValue::Command(cmd) => cmd.get_bindings(),
            MdlLeftValue::MkVal(mk_val) => mk_val.get_bindings(),
            MdlLeftValue::IMdl(imdl) => bindings_in_pattern(&imdl.params)
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
                        pattern: MdlLeftValue::Command(_) | MdlLeftValue::IMdl(_),
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

    pub fn is_reuse_model(&self) -> bool {
        match self {
            Mdl {
                left:
                Fact {
                    pattern: MdlLeftValue::IMdl(_),
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

    pub fn as_bound_model(&self) -> BoundModel {
        BoundModel {
            model: self.clone(),
            bindings: HashMap::new(),
        }
    }
}

impl Display for Mdl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:(mdl [] []", self.model_id)?;
        writeln!(f, "  {}", &self.left)?;
        writeln!(f, "  {}", &self.right)?;

        if self.forward_computed.is_empty() {
            writeln!(f, "|[]")?;
        }
        else {
            writeln!(f, "[]")?;
        }
        for (binding, func) in &self.forward_computed {
            writeln!(f, "  {binding}:{func}")?;
        }

        if self.backward_computed.is_empty() {
            write!(f, "|[]")?;
        }
        else {
            writeln!(f, "[]")?;
        }
        for (binding, func) in &self.backward_computed {
            writeln!(f, "  {binding}:{func}")?;
        }

        write!(f, "); Confidence {}, Success count: {}", self.confidence, self.success_count)?;

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MdlLeftValue {
    ICst(ICst),
    IMdl(IMdl),
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

    /// Pattern match two model lhs values and extract bindings
    /// `self` is expected to be unmodified from the model as
    pub fn matches(&self, bindings: &HashMap<String, Value>, other: &MdlLeftValue) -> PatternMatchResult {
        use MdlLeftValue::*;
        match (self, other) {
            (ICst(a), ICst(b)) => {
                a.matches(bindings, b, true, false)
            }
            (Command(a), Command(b)) => {
                a.matches(bindings, b, true, false)
            }
            (MkVal(a), MkVal(b)) => {
                a.matches(bindings, b, true)
            }
            (IMdl(a), IMdl(b)) => {
                a.matches(bindings, b, true, false)
            }
            (_, _) => PatternMatchResult::False
        }
    }

    pub fn filled_in(&self, bindings: &HashMap<String, Value>) -> MdlLeftValue {
        use MdlLeftValue::*;
        match self {
            ICst(icst) => {
                let mut icst = icst.clone();
                icst.params = fill_in_pattern_with_bindings(icst.params, bindings);
                ICst(icst)
            }
            Command(command) => {
                let mut command = command.clone();
                command.entity_id.insert_binding_value(bindings);
                command.params = fill_in_pattern_with_bindings(command.params, bindings);
                Command(command)
            }
            MkVal(mk_val) => {
                let mut mk_val = mk_val.clone();
                mk_val.entity_id.insert_binding_value(bindings);
                mk_val.value.insert_binding_values(bindings);
                MkVal(mk_val)
            }
            IMdl(imdl) => {
                let mut imdl = imdl.clone();
                imdl.params = fill_in_pattern_with_bindings(imdl.params.clone(), bindings);
                IMdl(imdl)
            }
        }
    }
}

impl Display for MdlLeftValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            MdlLeftValue::ICst(icst) => icst.to_string(),
            MdlLeftValue::IMdl(imdl) => imdl.to_string(),
            MdlLeftValue::Command(command) => command.to_string(),
            MdlLeftValue::MkVal(mk_val) => mk_val.to_string(),
        };
        write!(f, "{string}")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

    /// Pattern match two model rhs values and extract bindings
    /// `self` is expected to be unmodified from the model as
    pub fn matches(&self, bindings: &HashMap<String, Value>, other: &MdlRightValue) -> PatternMatchResult {
        use MdlRightValue::*;
        match (self, other) {
            (IMdl(a), IMdl(b)) => {
                a.matches(bindings, b, true, false)
            }
            (MkVal(a), MkVal(b)) => {
                a.matches(bindings, b, true)
            }
            (_, _) => PatternMatchResult::False
        }
    }

    pub fn filled_in(&self, bindings: &HashMap<String, Value>) -> MdlRightValue {
        use MdlRightValue::*;
        match self {
            IMdl(imdl) => {
                let mut imdl = imdl.clone();
                imdl.params = fill_in_pattern_with_bindings(imdl.params, bindings);
                IMdl(imdl)
            }
            MkVal(mk_val) => {
                let mut mk_val = mk_val.clone();
                mk_val.entity_id.insert_binding_value(bindings);
                mk_val.value.insert_binding_values(bindings);
                MkVal(mk_val)
            }
        }
    }
}

impl Display for MdlRightValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            MdlRightValue::IMdl(imdl) => imdl.to_string(),
            MdlRightValue::MkVal(mk_val) => mk_val.to_string(),
        };
        write!(f, "{string}")?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum AbductionResult {
    SubGoal(Vec<Fact<MkVal>>, Option<String>, IMdl, Option<ICst>),
    IMdl(IMdl),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

    pub fn get_model<'a>(&self, system: &'a System) -> &'a Mdl {
        system.models.get(&self.model_id).expect(&format!("Model in imdl does not exist {}", self.model_id))
    }

    pub fn merge_with(mut self, imdl: IMdl) -> IMdl {
        self.params = combine_pattern_bindings(self.params, imdl.params);
        self.fwd_guard_bindings.extend(imdl.fwd_guard_bindings);
        self
    }

    pub fn matches(&self, bindings: &HashMap<String, Value>, other: &IMdl, allow_unbound: bool, allow_different_length: bool,) -> PatternMatchResult {
        if self.model_id == other.model_id
            && compare_patterns(&fill_in_pattern_with_bindings(self.params.clone(), bindings), &other.params, allow_unbound, allow_different_length) {
            PatternMatchResult::True(extract_bindings_from_patterns(&self.params, &other.params))
        }
        else {
            PatternMatchResult::False
        }
    }

    pub fn filled_in(&self, bindings: &HashMap<String, Value>) -> IMdl {
        let mut imdl = self.clone();
        imdl.params = fill_in_pattern_with_bindings(imdl.params.clone(), bindings);
        imdl
    }
}

// Hacky Eq implementation using partial eq function
impl Eq for IMdl {
}

impl Hash for IMdl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.model_id.hash(state);
    }
}


impl Display for IMdl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(imdl {} [{}] | {})",
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

    pub fn deduce(&self, input: &Fact<MdlLeftValue>, anti_requirements: &Vec<&IMdl>) -> Option<Fact<MdlRightValue>> {
        let PatternMatchResult::True(mut bindings) = self.model.left.pattern.matches(&self.bindings, &input.pattern) else {
            return None;
        };
        // Combine bindings from input facts and those that were already in the model
        bindings.extend(self.bindings.clone());
        let mut model = BoundModel {
            model: self.model.clone(),
            bindings
        };
        model.compute_forward_bindings();

        // Check if this model matches any anti-requirement
        let self_imdl = model.imdl_for_model();
        if anti_requirements.iter().any(|req| compare_imdls(req, &self_imdl, true, true)) {
            return None;
        }

        Some(self.model.right.with_pattern(model.filled_in_rhs()))
    }

    pub fn abduce(&self, input: &Fact<MdlRightValue>, system: &System) -> Option<AbductionResult> {
        // TODO: Implement abduction on anti-models
        if self.model.right.anti {
            return None;
        }
        let PatternMatchResult::True(mut bindings) = self.model.right.pattern.matches(&self.bindings, &input.pattern) else {
            return None;
        };
        // Combine bindings from input facts and those that were already in the model
        bindings.extend(self.bindings.clone());
        let mut model = BoundModel {
            model: self.model.clone(),
            bindings
        };
        model.compute_backward_bindings();

        match &self.model.left.pattern {
            MdlLeftValue::ICst(icst) => {
                let mut icst = icst.clone();
                icst.params = fill_in_pattern_with_bindings(icst.params, &model.bindings);
                let subgoal_cst = icst.expand_cst(&system);
                Some(AbductionResult::SubGoal(subgoal_cst.facts, Some(icst.cst_id.clone()), model.imdl_for_model(), Some(icst)))
            }
            MdlLeftValue::MkVal(mk_val) => {
                let mut mk_val = mk_val.clone();
                mk_val.entity_id.insert_binding_value(&model.bindings);
                mk_val.value.insert_binding_values(&model.bindings);
                Some(AbductionResult::SubGoal(vec![self.model.left.with_pattern(mk_val)], None, model.imdl_for_model(), None))
            }
            _ => {
                Some(AbductionResult::IMdl(model.imdl_for_model()))
            }
        }
    }

    pub fn filled_in_lhs(&self) -> MdlLeftValue {
        self.model.left.pattern.filled_in(&self.bindings)
    }

    pub fn filled_in_rhs(&self) -> MdlRightValue {
        self.model.right.pattern.filled_in(&self.bindings)
    }

    pub fn extend_bindings_with_lhs_input(&self, input: &Fact<MdlLeftValue>) -> Option<BoundModel> {
        let PatternMatchResult::True(mut bindings) = self.model.left.pattern.matches(&self.bindings, &input.pattern) else {
            return None;
        };
        // Combine bindings from input facts and those that were already in the model
        bindings.extend(self.bindings.clone());
        let mut model = BoundModel {
            model: self.model.clone(),
            bindings
        };
        model.compute_forward_bindings();

        Some(model)
    }

    /// Predict what happens to SystemState after model is executed
    /// Only meant to be used for casual models, has no effect on other types of models
    /// At the moment this does not take into account changes that other models predict will take place when the same action is taken
    pub fn predict_state_change(
        &self,
        state: &SystemState,
        anti_requirements: &Vec<&IMdl>,
        instantiated_casual_models: &Vec<BoundModel>,
        system: &System,
    ) -> Option<SystemState> {
        let MdlRightValue::MkVal(mk_val) = &self.model.right.pattern else {
            return None;
        };
        // If this is a reuse model, call predict state change on the reused model (the command model)
        // Prediction from reuse models will also be included
        if self.model.is_reuse_model() {
            return self.get_reused_model(instantiated_casual_models, system)
                .map(|m| m.predict_state_change(state, anti_requirements, instantiated_casual_models, system))
                .flatten();
        }
        // Don't predict if this model matches an anti-requirement
        let self_imdl = self.imdl_for_model();
        if anti_requirements.iter().any(|req| compare_imdls(req, &self_imdl, true, true)) {
            return None;
        }
        let Some(predicted_value) = mk_val
            .value
            .get_value_with_bindings(&self.bindings) else {
            return None;
        };
        let self_lhs = self.model.left.with_pattern(self.filled_in_lhs());


        let imdl_lhs = Fact::new(MdlLeftValue::IMdl(self_imdl), TimePatternRange::wildcard());
        // TODO: Consider handling anti-requirements in deduce
        // TODO: Taking other state changes into account doesn't really make sense, we can end up with wrong knowledge always ruining the solution
        let other_state_changes = instantiated_casual_models
            .iter()
            .filter_map(|m| m.deduce(&imdl_lhs, &anti_requirements).or_else(|| m.deduce(&self_lhs, &anti_requirements)))
            .filter_map(|rhs| {
                match rhs.pattern {
                    MdlRightValue::MkVal(mk_val) => {
                        if let (Some(entity_id), Some(value)) = (
                            mk_val.entity_id.get_id_with_bindings(&HashMap::new()),
                            mk_val.value.get_value_with_bindings(&HashMap::new())
                        ) {
                            Some((EntityVariableKey::new(&entity_id, &mk_val.var_name), value))
                        }
                        else {
                            //log::error!("Reuse model produced unbound variables during forward chaining. Rhs: {mk_val}");
                            None
                        }
                    }
                    MdlRightValue::IMdl(imdl) => {
                        log::error!("Found reuse model with rhs {imdl} when expecting state change (mk.val)");
                        None
                    }
                }
            })
            .collect_vec();

        let mut new_state = state.clone();
        new_state.variables.extend(other_state_changes);
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
        /*new_state
            .variables
            .extend(compute_state_predictions(&system, &new_state));
        new_state
            .variables
            .extend(compute_assumptions(&system, &new_state));*/
        // Compute instantiated csts again, now with assumption variables
        // new_state.instansiated_csts = compute_instantiated_states(system, &new_state);

        Some(new_state)
    }

    /// For casual model, gets the command that would be executed to cause the predicted state change
    /// That is either lhs or the lhs of a model that is being reused
    pub fn get_casual_model_command(&self, instantiated_casual_models: &Vec<BoundModel>, system: &System) -> Option<Command> {
        use MdlLeftValue::*;
        match &self.model.left.pattern {
            Command(command) => {
                let mut command = command.clone();
                command.params = fill_in_pattern_with_bindings(command.params, &self.bindings);
                command.entity_id.insert_binding_value(&self.bindings);
                Some(command)
            }
            IMdl(imdl) => {
                let imdl = imdl.filled_in(&self.bindings);
                instantiated_casual_models
                    .iter()
                    .filter_map(|m| {
                        let instantiated_imdl = m.imdl_for_model();
                        if compare_imdls(&imdl, &instantiated_imdl, true, false) {
                            let reused_model = imdl.clone().merge_with(instantiated_imdl).instantiate(&HashMap::new(), system);
                            reused_model.get_casual_model_command(&instantiated_casual_models, &system)
                        }
                        else {
                            None
                        }
                    })
                    .next()
            }
            _ => None
        }
    }

    pub fn get_reused_model(&self, instantiated_casual_models: &Vec<BoundModel>, system: &System) -> Option<BoundModel> {
        use MdlLeftValue::*;
        match &self.model.left.pattern {
            IMdl(imdl) => {
                let imdl = imdl.filled_in(&self.bindings);
                instantiated_casual_models
                    .iter()
                    .filter_map(|m| {
                        let instantiated_imdl = m.imdl_for_model();
                        if compare_imdls(&imdl, &instantiated_imdl, true, false) {
                            let reused_model = imdl.clone().merge_with(instantiated_imdl).instantiate(&HashMap::new(), system);
                            Some(reused_model)
                        }
                        else {
                            None
                        }
                    })
                    .next()
            }
            _ => None
        }
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
