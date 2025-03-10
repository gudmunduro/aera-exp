use crate::types::pattern::PatternItem;
use itertools::Itertools;
use std::collections::HashMap;
use crate::types::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Value(PatternItem),
    Add(Box<Function>, Box<Function>),
    Sub(Box<Function>, Box<Function>),
    Mul(Box<Function>, Box<Function>),
    Div(Box<Function>, Box<Function>),
    List(Vec<Function>),
}

impl Function {
    pub fn evaluate(&self, bindings: &HashMap<String, Value>) -> Option<Value> {
        match self {
            Function::Value(v) => v.get_value_with_bindings(bindings),
            Function::Add(v1, v2) => Some(v1.evaluate(bindings)? + v2.evaluate(bindings)?),
            Function::Sub(v1, v2) => Some(v1.evaluate(bindings)? - v2.evaluate(bindings)?),
            Function::Mul(v1, v2) => Some(v1.evaluate(bindings)? * v2.evaluate(bindings)?),
            Function::Div(v1, v2) => Some(v1.evaluate(bindings)? / v2.evaluate(bindings)?),
            Function::List(items) => Some(Value::Vec(
                items.iter().filter_map(|f| f.evaluate(bindings)).collect(),
            )),
        }
    }

    pub fn binding_params(&self) -> Vec<String> {
        match self {
            Function::Value(p) => p.get_bindings(),
            Function::Add(v1, v2)
            | Function::Sub(v1, v2)
            | Function::Mul(v1, v2)
            | Function::Div(v1, v2) => v1
                .binding_params()
                .into_iter()
                .chain(v2.binding_params())
                .unique()
                .collect(),
            Function::List(l) => l.iter().flat_map(|f| f.binding_params()).collect(),
        }
    }
}
