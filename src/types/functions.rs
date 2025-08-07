use crate::types::pattern::PatternItem;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::types::value::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Function {
    Value(PatternItem),
    Add(Box<Function>, Box<Function>),
    Sub(Box<Function>, Box<Function>),
    Mul(Box<Function>, Box<Function>),
    Div(Box<Function>, Box<Function>),
    List(Vec<Function>),
    ConvertToEntityId(Box<Function>),
    ConvertToNumber(Box<Function>),
}

impl Function {
    pub fn evaluate(&self, bindings: &HashMap<String, Value>) -> Option<Value> {
        match self {
            Function::Value(v) => v.get_value_with_bindings(bindings),
            Function::Add(v1, v2) => {
                let (v1, v2) = (v1.evaluate(bindings)?, v2.evaluate(bindings)?);
                Function::validate_same_type_for_op(&v1, &v2)?;
                Some(v1 + v2)
            },
            Function::Sub(v1, v2) => {
                let (v1, v2) = (v1.evaluate(bindings)?, v2.evaluate(bindings)?);
                Function::validate_same_type_for_op(&v1, &v2)?;
                Some(v1 - v2)
            },
            Function::Mul(v1, v2) => {
                let (v1, v2) = (v1.evaluate(bindings)?, v2.evaluate(bindings)?);
                Function::validate_same_type_for_op(&v1, &v2)?;
                Some(v1 * v2)
            },
            Function::Div(v1, v2) => {
                let (v1, v2) = (v1.evaluate(bindings)?, v2.evaluate(bindings)?);
                Function::validate_same_type_for_op(&v1, &v2)?;
                Some(v1 / v2)
            },
            Function::List(items) => Some(Value::Vec(
                items.iter().filter_map(|f| f.evaluate(bindings)).collect(),
            )),
            Function::ConvertToEntityId(f) => {
                let str_id = f.evaluate(bindings)?.try_to_string()?;

                Some(Value::EntityId(str_id))
            },
            Function::ConvertToNumber(f) => {
                let str_id = match f.evaluate(bindings)? {
                    v @ Value::UncertainNumber(_, _) => return Some(v),
                    v @ Value::Number(_) => return Some(v),
                    v @ Value::ConstantNumber(_) => return Some(v),
                    Value::String(s) => s.clone(),
                    Value::EntityId(s) => s.clone(),
                    Value::Vec(_) => return None,
                };

                Some(Value::Number(str_id.parse().ok()?))
            },
        }
    }

    pub fn validate_same_type_for_op(v1: &Value, v2: &Value) -> Option<()> {
        match (v1, v2) {
            (Value::Vec(v1), Value::Vec(v2)) if v1.len() == v2.len() => Some(()),
            (Value::Number(_) | Value::UncertainNumber(_, _), Value::Number(_) | Value::UncertainNumber(_, _)) => Some(()),
            (Value::String(_), Value::String(_)) => Some(()),
            (Value::EntityId(_), Value::EntityId(_)) => Some(()),
            _ => None,
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
            Function::ConvertToEntityId(f) => f.binding_params(),
            Function::ConvertToNumber(f) => f.binding_params(),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Value(v) => write!(f, "{v}")?,
            Function::Add(f1, f2) => write!(f, "(+ {f1} {f2})")?,
            Function::Sub(f1, f2) => write!(f, "(- {f1} {f2})")?,
            Function::Mul(f1, f2) => write!(f, "(* {f1} {f2})")?,
            Function::Div(f1, f2) => write!(f, "(/ {f1} {f2})")?,
            Function::List(l) => write!(f, "[{}]", l.iter().map(|v| v.to_string()).join(" "))?,
            Function::ConvertToEntityId(func) => write!(f, "(toEntityId {func})")?,
            Function::ConvertToNumber(func) => write!(f, "(toNumber {func})")?
        }

        Ok(())
    }
}