use crate::types::pattern::PatternItem;
use crate::types::runtime::RuntimeValue;
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Value(PatternItem),
    Add(Box<Function>, Box<Function>),
    Sub(Box<Function>, Box<Function>),
    Mul(Box<Function>, Box<Function>),
    Div(Box<Function>, Box<Function>),
}

impl Function {
    pub fn evaluate(&self, bindings: &HashMap<String, RuntimeValue>) -> Option<RuntimeValue> {
        match self {
            Function::Value(v) => v.get_value_with_bindings(bindings),
            Function::Add(v1, v2) => Some(v1.evaluate(bindings)? + v2.evaluate(bindings)?),
            Function::Sub(v1, v2) => Some(v1.evaluate(bindings)? - v2.evaluate(bindings)?),
            Function::Mul(v1, v2) => Some(v1.evaluate(bindings)? * v2.evaluate(bindings)?),
            Function::Div(v1, v2) => Some(v1.evaluate(bindings)? / v2.evaluate(bindings)?),
        }
    }

    pub fn binding_params(&self) -> Vec<String> {
        match self {
            Function::Value(PatternItem::Binding(b)) => vec![b.clone()],
            Function::Value(_) => vec![],
            Function::Add(v1, v2)
            | Function::Sub(v1, v2)
            | Function::Mul(v1, v2)
            | Function::Div(v1, v2) => {
                v1
                    .binding_params()
                    .into_iter()
                    .chain(v2.binding_params())
                    .unique()
                    .collect()
            }
        }
    }
}
