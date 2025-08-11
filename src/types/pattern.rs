use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::types::value::Value;

pub type Pattern = Vec<PatternItem>;

#[derive(Clone, Debug, PartialEq, Hash, Serialize, Deserialize)]
pub enum PatternItem {
    Any,
    Binding(String),
    Value(Value),
    Vec(Vec<PatternItem>)
}

impl PatternItem {
    pub fn as_value(&self) -> &Value {
        match self  {
            PatternItem::Value(v) => v,
            _ => panic!("Pattern item needs to be a value"),
        }
    }

    pub fn get_value_with_bindings(&self, bindings: &HashMap<String, Value>) -> Option<Value> {
        match self {
            PatternItem::Any => None,
            PatternItem::Binding(b) => bindings.get(b).cloned(),
            PatternItem::Value(v) => Some(v.clone()),
            PatternItem::Vec(v) => Some(Value::Vec(v.iter().map(|e| e.get_value_with_bindings(bindings)).collect::<Option<Vec<_>>>()?))
        }
    }

    pub fn contains_binding(&self, binding: &str) -> bool {
        match self {
            PatternItem::Binding(b) => b == binding,
            PatternItem::Vec(p) => p.iter().any(|i| i.contains_binding(binding)),
            PatternItem::Value(_) | PatternItem::Any => false
        }
    }

    pub fn get_bindings(&self) -> Vec<String> {
        match self {
            PatternItem::Binding(b) => vec![b.clone()],
            PatternItem::Vec(v) => v.iter().flat_map(|v| v.get_bindings()).collect_vec(),
            PatternItem::Any | PatternItem::Value(_) => Vec::new(),
        }
    }

    pub fn insert_binding_values(&mut self, bindings: &HashMap<String, Value>) {
        match self {
            PatternItem::Binding(b) if bindings.contains_key(b) => {
                *self = PatternItem::Value(bindings[b].clone());
            },
            PatternItem::Vec(v) => {
                v.iter_mut().for_each(|e| e.insert_binding_values(bindings));
            }
            PatternItem::Binding(_) | PatternItem::Any | PatternItem::Value(_) => {}
        }
    }
    pub fn insert_pattern_binding_values(&mut self, bindings: &HashMap<String, PatternItem>) {
        match self {
            PatternItem::Binding(b) if bindings.contains_key(b) => {
                *self = bindings[b].clone();
            },
            PatternItem::Vec(v) => {
                v.iter_mut().for_each(|e| e.insert_pattern_binding_values(bindings));
            }
            PatternItem::Binding(_) | PatternItem::Any | PatternItem::Value(_) => {}
        }
    }
    
    /// Check if the pattern has no concrete value
    pub fn is_fully_unbound(&self) -> bool {
        match self {
            PatternItem::Binding(_) | PatternItem::Any => true,
            PatternItem::Vec(v) => v.iter().all(|e| e.is_fully_unbound()),
            PatternItem::Value(_) => false
        }
    }

    pub fn pattern(&self) -> Pattern {
        vec![self.clone()]
    }
}

impl Display for PatternItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            PatternItem::Any => "::".to_string(),
            PatternItem::Binding(b) => format!("{b}:"),
            PatternItem::Value(v) => v.to_string(),
            PatternItem::Vec(v) => format!("[{}]", v.iter().map(|e| e.to_string()).join(" ")),
        })?;

        Ok(())
    }
}

pub fn bindings_in_pattern(pattern: &Pattern) -> Vec<String> {
    pattern
        .iter()
        .flat_map(|i| i.get_bindings())
        .collect()
}

pub fn flatten_pattern_vecs(pattern: Pattern) -> Pattern {
    pattern.into_iter()
        .flat_map(|i| flatten_pattern_item_vecs(i))
        .collect()
}

pub fn flatten_pattern_item_vecs(pattern_item: PatternItem) -> Vec<PatternItem> {
    match pattern_item {
        PatternItem::Vec(p) => flatten_pattern_vecs(p),
        i => vec![i]
    }
}