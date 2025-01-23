use std::collections::HashMap;
use crate::types::{cst::Cst, models::Mdl, EntityVariableKey, MkVal};
use crate::types::cst::InstantiatedCst;
use crate::types::pattern::{PatternItem, PatternValue};

pub struct RuntimeData {
    pub current_state: SystemState,
    pub models: HashMap<String, Mdl>,
    pub csts: HashMap<String, Cst>,
}

impl RuntimeData {
    pub fn new() -> RuntimeData {
        RuntimeData {
            current_state: SystemState {
                variables: HashMap::new(),
                instansiated_csts: HashMap::new(),
            },
            models: HashMap::new(),
            csts: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemState {
    pub variables: HashMap<EntityVariableKey, RuntimeValue>,
    pub instansiated_csts: HashMap<String, InstantiatedCst>,
}

impl SystemState {
    pub fn new() -> SystemState {
        SystemState {
            variables: HashMap::new(),
            instansiated_csts: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeValue {
    Number(f64),
    String(String)
}

impl PartialEq<PatternValue> for RuntimeValue {
    fn eq(&self, other: &PatternValue) -> bool {
        match (self, other) {
            (RuntimeValue::Number(n), PatternValue::Number(n2)) => n == n2,
            (RuntimeValue::String(s), PatternValue::String(s2)) => s == s2,
            _ => false
        }
    }
}

impl PartialEq<&PatternValue> for RuntimeValue {
    fn eq(&self, other: &&PatternValue) -> bool {
        match (self, other) {
            (RuntimeValue::Number(n), PatternValue::Number(n2)) => n == n2,
            (RuntimeValue::String(s), PatternValue::String(s2)) => s == s2,
            _ => false
        }
    }
}

impl PartialEq<PatternItem> for RuntimeValue {
    fn eq(&self, other: &PatternItem) -> bool {
        match other {
            PatternItem::Any => true,
            // TODO: Variable matching
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
    pub fn from_mk_val(mk_val: &MkVal, value: &RuntimeValue) -> AssignedMkVal {
        AssignedMkVal {
            entity_id: mk_val.entity_id.clone(),
            var_name: mk_val.var_name.clone(),
            pattern_value: mk_val.value.clone(),
            value: value.clone(),
        }
    }
}
