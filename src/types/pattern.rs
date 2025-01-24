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