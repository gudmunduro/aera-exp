use crate::types::{
    cst::Cst, models::Mdl, EntityVariableKey, Time, TimePatternRange,
    TimePatternValue,
};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use itertools::Itertools;
use crate::types::cst::BoundCst;
use crate::types::value::Value;

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
    pub variables: HashMap<EntityVariableKey, Value>,
    pub instansiated_csts: HashMap<String, Vec<BoundCst>>,
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
pub struct RuntimeCommand {
    pub name: String,
    pub entity_id: String,
    pub params: Vec<Value>,
}

impl Display for RuntimeCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(cmd {} {} {})", self.name, self.entity_id, self.params.iter().map(|v| v.to_string()).join(" "))?;

        Ok(())
    }
}