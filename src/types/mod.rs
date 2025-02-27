use std::collections::HashMap;
use anyhow::bail;
use crate::runtime::pattern_matching::bind_values_to_pattern;
use crate::types::pattern::{Pattern, PatternItem};
use crate::types::runtime::RuntimeCommand;
use crate::types::value::Value;

pub mod runtime;
pub mod models;
pub mod cst;
pub mod pattern;
pub mod functions;
pub mod value;

// Time is stored in milliseconds
type Time = u64;

#[derive(Clone, Debug)]
pub struct Command {
    pub name: String,
    pub entity_id: EntityPatternValue,
    pub params: Pattern,
}

impl Command {
    pub fn to_runtime_command(&self, bindings: &HashMap<String, Value>) -> anyhow::Result<RuntimeCommand> {
        let params = bind_values_to_pattern(&self.params, bindings);

        if params.len() < self.params.len() {
            bail!("Cannot get command {} from model. Bindings missing for params", &self.name);
        }

        Ok(RuntimeCommand {
            name: self.name.clone(),
            entity_id: self.entity_id.get_id_with_bindings(bindings).unwrap(),
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

impl MatchesFact<MkVal> for Fact<MkVal> {
    fn matches_fact(&self, fact: &Fact<MkVal>) -> bool {
        self.pattern.matches_mk_val(&fact.pattern)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MkVal {
    pub entity_id: EntityPatternValue,
    pub var_name: String,
    pub value: PatternItem,
}

impl MkVal {
    /// Checks if two mk.val are equal, assumes bindings are equivalent to wildcard
    pub fn matches_mk_val(&self, mk_val: &MkVal) -> bool {
        let matches_value = match (&self.value, &mk_val.value) {
            (PatternItem::Any | PatternItem::Binding(_), _) | (_, PatternItem::Any | PatternItem::Binding(_)) => true,
            (PatternItem::Value(v1), PatternItem::Value(v2)) => v1 == v2
        };
        let matches_entity = match (&self.entity_id, &mk_val.entity_id) {
            (EntityPatternValue::Binding(_), _) | (_, EntityPatternValue::Binding(_)) => true,
            (EntityPatternValue::EntityId(e1), EntityPatternValue::EntityId(e2)) => e1 == e2
        };
        matches_entity && self.var_name == mk_val.var_name && matches_value
    }

    pub fn entity_key(&self, bindings: &HashMap<String, Value>) -> Option<EntityVariableKey> {
        Some(EntityVariableKey {
            entity_id: self.entity_id.get_id_with_bindings(bindings)?,
            var_name: self.var_name.clone(),
        })
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
    Any,
    Binding(String)
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

#[derive(Clone, Debug, PartialEq)]
pub enum EntityPatternValue {
    Binding(String),
    EntityId(String),
}

impl EntityPatternValue {
    pub fn get_id_with_bindings(&self, bindings: &HashMap<String, Value>) -> Option<String> {
        match self {
            EntityPatternValue::Binding(b) => match bindings.get(b)? {
                Value::EntityId(id) => Some(id.clone()),
                v => panic!("Binding {b} expected to have type entity, but is {v:?}")
            }
            EntityPatternValue::EntityId(id) => Some(id.clone())
        }
    }

    pub fn is_binding(&self, binding: &str) -> bool {
        matches!(self, EntityPatternValue::Binding(b) if b == binding)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityDeclaration {
    pub binding: String,
    pub class: String,
}

impl EntityDeclaration {
    pub fn new(binding: &str, class: &str) -> EntityDeclaration {
        EntityDeclaration { binding: binding.to_owned(), class: class.to_owned() }
    }
}