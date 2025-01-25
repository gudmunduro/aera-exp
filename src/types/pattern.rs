use std::collections::HashMap;
use crate::types::runtime::RuntimeValue;

pub type Pattern = Vec<PatternItem>;

#[derive(Clone, Debug, PartialEq)]
pub enum PatternItem {
    Any,
    Binding(String),
    Value(PatternValue),
}

impl PatternItem {
    pub fn as_value(&self) -> &PatternValue {
        match self  {
            PatternItem::Value(v) => v,
            _ => panic!("Pattern item needs to be a value"),
        }
    }

    pub fn get_value_with_bindings(&self, bindings: &HashMap<String, RuntimeValue>) -> Option<RuntimeValue> {
        match self {
            PatternItem::Any => panic!("Cannot get value from wildcard pattern"),
            PatternItem::Binding(b) => bindings.get(b).cloned(),
            PatternItem::Value(v) => Some(v.clone().into())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PatternValue {
    String(String),
    Number(f64),
}

impl From<RuntimeValue> for PatternValue {
    fn from(value: RuntimeValue) -> Self {
        match value {
            RuntimeValue::Number(n) => PatternValue::Number(n),
            RuntimeValue::String(s) => PatternValue::String(s),
        }
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