use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::types::value::Value;

pub type Pattern = Vec<PatternItem>;

#[derive(Clone, Debug, PartialEq)]
pub enum PatternItem {
    Any,
    Binding(String),
    Value(Value),
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
            PatternItem::Value(v) => Some(v.clone())
        }
    }

    pub fn is_binding(&self, binding: &str) -> bool {
        matches!(self, PatternItem::Binding(b) if b == binding)
    }
}

impl Display for PatternItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            PatternItem::Any => "*".to_string(),
            PatternItem::Binding(b) => format!("{b}:"),
            PatternItem::Value(v) => v.to_string(),
        })?;

        Ok(())
    }
}

pub fn bindings_in_pattern(pattern: &Pattern) -> Vec<String> {
    pattern
        .iter()
        .filter_map(|i| match i {
            PatternItem::Binding(b) => Some(b.clone()),
            _ => None
        })
        .collect()
}