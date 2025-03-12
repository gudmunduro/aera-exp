use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul, Sub};
use itertools::Itertools;
use crate::types::pattern::PatternItem;
use crate::utils::{float_cmp, float_eq};
use crate::utils::math::probability_density;

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    UncertainNumber(f64, f64),
    String(String),
    Vec(Vec<Value>),
    EntityId(String),
}

impl Value {
    pub fn as_number(&self) -> f64 {
        match &self {
            Value::Number(v) => *v,
            _ => panic!("Value excepted to be a number"),
        }
    }
    pub fn as_vec(&self) -> &Vec<Value> {
        match &self {
            Value::Vec(v) => v,
            _ => panic!("Value excepted to be a vector"),
        }
    }
    pub fn as_entity_id(&self) -> &str {
        match &self {
            Value::EntityId(id) => id,
            _ => panic!("Value excepted to be an entity id"),
        }
    }
}

impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(n), Value::Number(n2)) => float_cmp(*n, *n2, 0.1),
            (Value::Number(n), Value::UncertainNumber(m, s))
            | (Value::UncertainNumber(m, s), Value::Number(n)) => {
                probability_density(*n, *m, *s) > 0.001
            }
            (Value::UncertainNumber(m1, s1), Value::UncertainNumber(m2, s2)) if float_eq(*s1, *s2) => {
                probability_density(*m1, *m2, *s1) > 0.001
            }
            (Value::String(s), Value::String(s2)) => s == s2,
            (Value::Vec(v), Value::Vec(v2)) => v == v2,
            (Value::EntityId(id), Value::EntityId(id2)) => id == id2,
            _ => false,
        }
    }
}

impl PartialEq<PatternItem> for Value {
    fn eq(&self, other: &PatternItem) -> bool {
        match other {
            PatternItem::Any => true,
            // Value is assumed to always be a match for binding, which may not be correct in all cases
            PatternItem::Binding(_) => true,
            PatternItem::Value(value) => self == value,
            PatternItem::Vec(vec) => match self {
                Value::Vec(vec2) => vec.len() == vec2.len()
                    && vec.iter().zip(vec2).all(|(a, b)| b == a),
                _ => false,
            }
        }
    }
}

impl Add<Value> for Value {
    type Output = Value;

    fn add(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 + n2),
            (Value::Vec(v1), Value::Vec(v2)) => Value::Vec(
                v1.into_iter()
                    .zip(v2)
                    .map(|(e1, e2)| e1 + e2)
                    .collect(),
            ),
            (Value::UncertainNumber(m, s), Value::Number(n)) => Value::UncertainNumber(m + n, s),
            (Value::Number(n), Value::UncertainNumber(m, s)) => Value::UncertainNumber(n + m, s),
            _ => panic!("Value does not support addition"),
        }
    }
}

impl Sub<Value> for Value {
    type Output = Value;

    fn sub(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 - n2),
            (Value::Vec(v1), Value::Vec(v2)) => Value::Vec(
                v1.into_iter()
                    .zip(v2)
                    .map(|(e1, e2)| e1 - e2)
                    .collect(),
            ),
            (Value::UncertainNumber(m, s), Value::Number(n)) => Value::UncertainNumber(m - n, s),
            (Value::Number(n), Value::UncertainNumber(m, s)) => Value::UncertainNumber(n - m, s),
            (Value::UncertainNumber(m1, s1), Value::UncertainNumber(m2, s2)) if float_eq(s1, s2) => Value::UncertainNumber(m1 - m2, s1),
            (v1, v2) => panic!("Value does not support subtraction ({v1:?} - {v2:?})"),
        }
    }
}

impl Mul<Value> for Value {
    type Output = Value;

    fn mul(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 * n2),
            (Value::Vec(v1), Value::Vec(v2)) => Value::Vec(
                v1.into_iter()
                    .zip(v2)
                    .map(|(e1, e2)| e1 * e2)
                    .collect(),
            ),
            (Value::UncertainNumber(m, s), Value::Number(n)) => Value::UncertainNumber(m * n, s),
            (Value::Number(n), Value::UncertainNumber(m, s)) => Value::UncertainNumber(n * m, s),
            _ => panic!("Value does not support multiplication"),
        }
    }
}

impl Div<Value> for Value {
    type Output = Value;

    fn div(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 / n2),
            (Value::Vec(v1), Value::Vec(v2)) => Value::Vec(
                v1.into_iter()
                    .zip(v2)
                    .map(|(e1, e2)| e1 / e2)
                    .collect(),
            ),
            (Value::UncertainNumber(m, s), Value::Number(n)) => Value::UncertainNumber(m / n, s),
            (Value::Number(n), Value::UncertainNumber(m, s)) => Value::UncertainNumber(n / m, s),
            _ => panic!("Value does not support division"),
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Number(v) => ((v * 10.0) as i64).hash(state),
            Value::UncertainNumber(m, s) => [(m * 10.0) as i64, (s * 10.0) as i64].hash(state),
            Value::String(s) => s.hash(state),
            Value::Vec(v) => v.hash(state),
            Value::EntityId(e) => e.hash(state)
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Value::Number(n) => n.to_string(),
            Value::UncertainNumber(m, s) => format!("(uncertain {m} {s})"),
            Value::String(s) => format!("\"{}\"", s.to_owned()),
            Value::Vec(v) => format!("[{}]", v.iter().map(|e| e.to_string()).join(" ")),
            Value::EntityId(id) => id.to_owned()
        })?;

        Ok(())
    }
}