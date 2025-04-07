use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use anyhow::{anyhow, bail};
use crate::runtime::pattern_matching::{bind_values_to_pattern, compare_pattern_items, compare_patterns, extract_bindings_from_patterns, fill_in_pattern_with_bindings, PatternMatchResult};
use crate::types::pattern::{bindings_in_pattern, Pattern, PatternItem};
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
            entity_id: self.entity_id.get_id_with_bindings(bindings).ok_or(anyhow!("Entity binding for command missing"))?,
            params,
        })
    }

    pub fn get_bindings(&self) -> Vec<String> {
        if let EntityPatternValue::Binding(b) = &self.entity_id {
            // Add entity id binding as well so it appears first in params
            [vec![b.clone()], bindings_in_pattern(&self.params)].concat()
        } else {
            bindings_in_pattern(&self.params)
        }
    }

    pub fn matches(&self, bindings: &HashMap<String, Value>, other: &Command, allow_unbound: bool, allow_different_length: bool,) -> PatternMatchResult {
        if self.name == other.name
            && compare_patterns(&fill_in_pattern_with_bindings(self.params.clone(), bindings), &other.params, allow_unbound, allow_different_length) {
            PatternMatchResult::True(extract_bindings_from_patterns(&self.params, &other.params))
        }
        else {
            PatternMatchResult::False
        }
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
    pub fn new(pattern: T, time_range: TimePatternRange) -> Fact<T> {
        Fact {
            pattern,
            time_range
        }
    }

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
    // If true, this mk.val will not have to come from the controller
    // but will instead be assumed to be true if a model predicts it for the current state
    pub assumption: bool,
}

impl MkVal {
    /// Checks if two mk.val are equal, assumes bindings are equivalent to wildcard
    pub fn matches_mk_val(&self, mk_val: &MkVal) -> bool {
        let matches_entity = match (&self.entity_id, &mk_val.entity_id) {
            (EntityPatternValue::Binding(_), _) | (_, EntityPatternValue::Binding(_)) => true,
            (EntityPatternValue::EntityId(e1), EntityPatternValue::EntityId(e2)) => e1 == e2
        };
        matches_entity && self.var_name == mk_val.var_name && compare_pattern_items(&self.value, &mk_val.value, true)
    }

    pub fn matches(&self, bindings: &HashMap<String, Value>, other: &MkVal, allow_unbound: bool) -> PatternMatchResult {
        let matches_var = self.var_name == other.var_name;
        let matches_entity = match (&self.entity_id, &other.entity_id) {
            (EntityPatternValue::Binding(_), _) | (_, EntityPatternValue::Binding(_)) => allow_unbound,
            (EntityPatternValue::EntityId(e1), EntityPatternValue::EntityId(e2)) => e1 == e2
        };
        // Early return optimization since compare patterns in expensive
        if !matches_var || !matches_entity {
            return PatternMatchResult::False;
        }

        let mut value = self.value.clone();
        value.insert_binding_values(bindings);

        if self.var_name == other.var_name
            && compare_pattern_items(&value, &other.value, allow_unbound) {
            PatternMatchResult::True(extract_bindings_from_patterns(&self.value.pattern(), &other.value.pattern()))
        }
        else {
            PatternMatchResult::False
        }
    }

    pub fn entity_key(&self, bindings: &HashMap<String, Value>) -> Option<EntityVariableKey> {
        Some(EntityVariableKey {
            entity_id: self.entity_id.get_id_with_bindings(bindings)?,
            var_name: self.var_name.clone(),
        })
    }

    pub fn get_bindings(&self) -> Vec<String> {
        if let EntityPatternValue::Binding(b) = &self.entity_id {
            // Add entity id binding as well so it appears first in params
            [vec![b.clone()], self.value.get_bindings()].concat()
        } else {
            self.value.get_bindings()
        }
    }
}

impl Display for MkVal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(mk.val {} {} {})",
            &self.entity_id,
            &self.var_name,
            &self.value,
        )?;

        Ok(())
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

    pub fn wildcard() -> TimePatternRange {
        TimePatternRange::new(TimePatternValue::Any, TimePatternValue::Any)
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
                Value::String(id) => Some(id.clone()),
                Value::Number(id) => Some((*id as i32).to_string()),
                v => panic!("Binding {b} expected to have type entity, but is {v:?}")
            }
            EntityPatternValue::EntityId(id) => Some(id.clone())
        }
    }

    pub fn is_binding(&self, binding: &str) -> bool {
        matches!(self, EntityPatternValue::Binding(b) if b == binding)
    }

    pub fn insert_binding_value(&mut self, bindings: &HashMap<String, Value>) {
        match self {
            EntityPatternValue::Binding(b) => {
                if let Some(Value::EntityId(id)) = bindings.get(b) {
                    *self = EntityPatternValue::EntityId(id.to_owned());
                }
                else {
                    log::error!("Binding missing when trying to fill in entity binding")
                }
            }
            EntityPatternValue::EntityId(_) => {}
        }
    }
}

impl Display for EntityPatternValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityPatternValue::Binding(b) => write!(f, "{b}:"),
            EntityPatternValue::EntityId(id) => write!(f, "{id}")
        }
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