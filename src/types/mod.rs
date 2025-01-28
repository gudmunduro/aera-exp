use std::collections::HashMap;
use anyhow::bail;
use crate::runtime::pattern_matching::bind_values_to_pattern;
use crate::types::pattern::{Pattern, PatternItem};
use crate::types::runtime::{AssignedMkVal, RuntimeCommand, RuntimeValue};

pub mod runtime;
pub mod models;
pub mod cst;
pub mod pattern;

type Time = f64;

#[derive(Clone, Debug)]
pub struct Command {
    pub name: String,
    pub params: Pattern,
}

impl Command {
    pub fn to_runtime_command(&self, bindings: &HashMap<String, RuntimeValue>) -> anyhow::Result<RuntimeCommand> {
        let params = bind_values_to_pattern(&self.params, bindings);

        if params.len() < self.params.len() {
            bail!("Cannot get command {} from model. Bindings missing for params", &self.name);
        }

        Ok(RuntimeCommand {
            name: self.name.clone(),
            params,
        })
    }
}

pub trait MatchesFact<T: Clone> {
    fn matches_fact(&self, fact: &Fact<T>) -> bool;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fact<T: Clone> {
    pub pattern: T,
    pub time_range: TimePatternRange,
}

impl<T: Clone> Fact<T> {
    pub fn with_pattern<T2: Clone>(&self, pattern: T2) -> Fact<T2> {
        Fact {
            pattern,
            time_range: self.time_range.clone()
        }
    }
}

impl MatchesFact<AssignedMkVal> for Fact<MkVal> {
    fn matches_fact(&self, fact: &Fact<AssignedMkVal>) -> bool {
        // TODO: Handle time
        self.pattern.matches_assigned_mk_val(&fact.pattern)
    }
}

impl MatchesFact<MkVal> for Fact<MkVal> {
    fn matches_fact(&self, fact: &Fact<MkVal>) -> bool {
        // TODO: Handle time
        self.pattern.matches_mk_val(&fact.pattern)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MkVal {
    pub entity_id: String,
    pub var_name: String,
    pub value: PatternItem,
}

impl MkVal {
    pub fn assign_value(&self, value: &RuntimeValue) -> AssignedMkVal {
        AssignedMkVal::from_mk_val(self, value)
    }

    /// Checks if two mk.val are equal, assumes bindings are equivalent to wildcard
    pub fn matches_mk_val(&self, mk_val: &MkVal) -> bool {
        let matches_value = match (&self.value, &mk_val.value) {
            (PatternItem::Any | PatternItem::Binding(_), _) | (_, PatternItem::Any | PatternItem::Binding(_)) => true,
            (PatternItem::Value(v1), PatternItem::Value(v2)) => v1 == v2
        };
        self.entity_id == mk_val.entity_id && self.var_name == mk_val.var_name && matches_value
    }

    pub fn matches_assigned_mk_val(&self, mk_val: &AssignedMkVal) -> bool {
        let matches_value = match &self.value {
            PatternItem::Any => true,
            PatternItem::Binding(_) => true,
            PatternItem::Value(value) => mk_val.value == value
        };
        self.entity_id == mk_val.entity_id && self.var_name == mk_val.var_name && matches_value
    }

    pub fn entity_key(&self) -> EntityVariableKey {
        EntityVariableKey {
            entity_id: self.entity_id.clone(),
            var_name: self.var_name.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimePatternRange {
    pub from: TimePatternValue,
    pub to: TimePatternValue,
}

impl TimePatternRange {
    pub fn new(from: TimePatternValue, to: TimePatternValue) -> TimePatternRange {
        TimePatternRange { from, to }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TimePatternValue {
    Time(Time),
    Any
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityVariableKey {
    pub entity_id: String,
    pub var_name: String,
}

impl EntityVariableKey {
    pub fn new(entity_id: &str, variable: &str) -> EntityVariableKey {
        EntityVariableKey { entity_id: entity_id.to_string(), var_name: variable.to_string() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Goal {
    name: String,
    time_range: TimePatternRange,
}