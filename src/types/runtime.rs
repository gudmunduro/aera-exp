use crate::types::cst::InstantiatedCst;
use crate::types::pattern::{PatternItem, PatternValue};
use crate::types::{
    cst::Cst, models::Mdl, EntityPatternValue, EntityVariableKey, MkVal, Time, TimePatternRange,
    TimePatternValue,
};
use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Sub};

pub struct System {
    pub current_state: SystemState,
    pub models: HashMap<String, Mdl>,
    pub csts: HashMap<String, Cst>,
    pub entities_in_classes: HashMap<String, Vec<String>>,
}

impl System {
    pub fn new() -> System {
        System {
            current_state: SystemState {
                variables: HashMap::new(),
                instansiated_csts: HashMap::new(),
                time: SystemTime::Exact(0),
            },
            models: HashMap::new(),
            csts: HashMap::new(),
            entities_in_classes: HashMap::new(),
        }
    }

    pub fn create_entity(&mut self, entity_id: &str, class: &str) {
        let class = match self.entities_in_classes.get_mut(class) {
            None => {
                self.entities_in_classes
                    .insert(class.to_string(), Vec::new());
                self.entities_in_classes.get_mut(class).unwrap()
            }
            Some(c) => c,
        };

        class.push(entity_id.to_owned());
    }
}

#[derive(Clone, Debug)]
pub struct SystemState {
    pub variables: HashMap<EntityVariableKey, RuntimeValue>,
    pub instansiated_csts: HashMap<String, Vec<InstantiatedCst>>,
    pub time: SystemTime,
}

impl SystemState {
    pub fn new() -> SystemState {
        SystemState {
            variables: HashMap::new(),
            instansiated_csts: HashMap::new(),
            time: SystemTime::Exact(0),
        }
    }
}

impl PartialEq for SystemState {
    fn eq(&self, other: &SystemState) -> bool {
        self.variables == other.variables
    }
}

// The current state of the system will always have an exact time,
// but during simulation the time can be a range
#[derive(Clone, Debug, PartialEq)]
pub enum SystemTime {
    Exact(Time),
    Range(Time, Time),
}

impl SystemTime {
    // When comparing two ranges, it is considered a match even if only a part of the ranges overlap
    pub fn matches_pattern(&self, pattern: &TimePatternRange) -> bool {
        let (start, end) = match self {
            SystemTime::Exact(t) => (*t, *t),
            SystemTime::Range(t1, t2) => (*t1, *t2),
        };

        let pattern_start = match &pattern.from {
            TimePatternValue::Time(t) => *t,
            TimePatternValue::Any => 0,
            TimePatternValue::Binding(_) => panic!("Bindings not allowed when comparing time"),
        };
        let pattern_end = match &pattern.from {
            TimePatternValue::Time(t) => *t,
            TimePatternValue::Any => u64::MAX,
            TimePatternValue::Binding(_) => panic!("Bindings not allowed when comparing time"),
        };
        let pattern_range = pattern_start..pattern_end;

        pattern_range.contains(&start) || pattern_range.contains(&end)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeValue {
    Number(f64),
    String(String),
    List(Vec<RuntimeValue>),
    EntityId(String),
}

impl RuntimeValue {
    pub fn as_number(&self) -> f64 {
        match &self {
            RuntimeValue::Number(v) => *v,
            _ => panic!("Value excepted to be a number"),
        }
    }
    pub fn as_list(&self) -> &Vec<RuntimeValue> {
        match &self {
            RuntimeValue::List(l) => l,
            _ => panic!("Value excepted to be a list"),
        }
    }
    pub fn as_entity_id(&self) -> &str {
        match &self {
            RuntimeValue::EntityId(id) => id,
            _ => panic!("Value excepted to be an entity id"),
        }
    }
}

impl PartialEq<PatternValue> for RuntimeValue {
    fn eq(&self, other: &PatternValue) -> bool {
        match (self, other) {
            (RuntimeValue::Number(n), PatternValue::Number(n2)) => (n - n2).abs() < 0.1,
            (RuntimeValue::String(s), PatternValue::String(s2)) => s == s2,
            (RuntimeValue::List(l), PatternValue::List(l2)) => l == l2,
            (RuntimeValue::EntityId(id), PatternValue::EntityId(id2)) => id == id2,
            _ => false,
        }
    }
}

impl PartialEq<PatternItem> for RuntimeValue {
    fn eq(&self, other: &PatternItem) -> bool {
        match other {
            PatternItem::Any => true,
            // Value is assumed to always be a match for binding, which may not be correct in all cases
            PatternItem::Binding(_) => true,
            PatternItem::Value(value) => self == value,
        }
    }
}

impl From<PatternValue> for RuntimeValue {
    fn from(value: PatternValue) -> Self {
        match value {
            PatternValue::Number(n) => RuntimeValue::Number(n),
            PatternValue::String(s) => RuntimeValue::String(s),
            PatternValue::List(l) => {
                RuntimeValue::List(l.into_iter().map(|e| RuntimeValue::from(e)).collect())
            }
            PatternValue::EntityId(id) => RuntimeValue::EntityId(id),
        }
    }
}

impl Add<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn add(self, rhs: RuntimeValue) -> Self::Output {
        match self {
            RuntimeValue::Number(n) => RuntimeValue::Number(n + rhs.as_number()),
            RuntimeValue::List(l) => RuntimeValue::List(
                l.into_iter()
                    .zip(rhs.as_list())
                    .map(|(e1, e2)| e1 + e2.clone())
                    .collect(),
            ),
            _ => panic!("Value does not support addition"),
        }
    }
}

impl Sub<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn sub(self, rhs: RuntimeValue) -> Self::Output {
        match self {
            RuntimeValue::Number(n) => RuntimeValue::Number(n - rhs.as_number()),
            RuntimeValue::List(l) => RuntimeValue::List(
                l.into_iter()
                    .zip(rhs.as_list())
                    .map(|(e1, e2)| e1 - e2.clone())
                    .collect(),
            ),
            _ => panic!("Value does not support subtraction"),
        }
    }
}

impl Mul<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn mul(self, rhs: RuntimeValue) -> Self::Output {
        match self {
            RuntimeValue::Number(n) => RuntimeValue::Number(n * rhs.as_number()),
            RuntimeValue::List(l) => RuntimeValue::List(
                l.into_iter()
                    .zip(rhs.as_list())
                    .map(|(e1, e2)| e1 * e2.clone())
                    .collect(),
            ),
            _ => panic!("Value does not support multiplication"),
        }
    }
}

impl Div<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn div(self, rhs: RuntimeValue) -> Self::Output {
        match self {
            RuntimeValue::Number(n) => RuntimeValue::Number(n / rhs.as_number()),
            RuntimeValue::List(l) => RuntimeValue::List(
                l.into_iter()
                    .zip(rhs.as_list())
                    .map(|(e1, e2)| e1 / e2.clone())
                    .collect(),
            ),
            _ => panic!("Value does not support division"),
        }
    }
}

#[derive(Clone)]
pub struct RuntimeVariable {
    pub name: String,
    pub value: RuntimeValue,
}

impl RuntimeVariable {
    pub fn new(name: String, value: RuntimeValue) -> RuntimeVariable {
        RuntimeVariable { name, value }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssignedMkVal {
    pub entity_id: String,
    pub var_name: String,
    pub pattern_value: PatternItem,
    pub value: RuntimeValue,
}

impl AssignedMkVal {
    pub fn from_mk_val(
        mk_val: &MkVal,
        value: &RuntimeValue,
        entity_bindings: &HashMap<String, RuntimeValue>,
    ) -> AssignedMkVal {
        AssignedMkVal {
            entity_id: mk_val
                .entity_id
                .get_id_with_bindings(entity_bindings)
                .unwrap(),
            var_name: mk_val.var_name.clone(),
            pattern_value: mk_val.value.clone(),
            value: value.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeCommand {
    pub name: String,
    pub entity_id: String,
    pub params: Vec<RuntimeValue>,
}
