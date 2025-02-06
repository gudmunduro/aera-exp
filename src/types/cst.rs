use crate::runtime::pattern_matching::PatternMatchResult;
use crate::types::pattern::{Pattern};
use crate::types::runtime::{AssignedMkVal, System, SystemState};
use crate::types::{EntityDeclaration, EntityPatternValue, Fact, MkVal, PatternItem};
use itertools::Itertools;
use std::collections::HashMap;
use crate::types::value::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct Cst {
    pub cst_id: String,
    pub facts: Vec<Fact<MkVal>>,
    pub entities: Vec<EntityDeclaration>,
}

impl Cst {
    pub fn new(cst_id: String) -> Cst {
        Cst {
            cst_id,
            facts: Vec::new(),
            entities: Vec::new(),
        }
    }

    pub fn binding_params(&self) -> Vec<String> {
        let entity_bindings = self.entities.iter().map(|e| e.binding.clone());

        let fact_bindings = self
            .facts
            .iter()
            .filter_map(|fact| match &fact.pattern.value {
                PatternItem::Binding(name) => Some(name.clone()),
                _ => None,
            })
            .unique();

        entity_bindings.chain(fact_bindings).collect()
    }

    pub fn fill_in_bindings(&self, bindings: &HashMap<String, Value>) -> Cst {
        let mut facts = self.facts.clone();

        for fact in &mut facts {
            match &fact.pattern.value {
                PatternItem::Binding(b) => {
                    if let Some(binding_val) = bindings.get(b) {
                        fact.pattern.value = PatternItem::Value(binding_val.clone());
                    }
                }
                _ => {}
            }
            match &fact.pattern.entity_id {
                EntityPatternValue::Binding(b) => {
                    if let Some(binding_val) = bindings.get(b) {
                        fact.pattern.entity_id =
                            EntityPatternValue::EntityId(binding_val.as_entity_id().to_owned());
                    }
                }
                _ => {}
            }
        }

        Cst {
            cst_id: self.cst_id.clone(),
            facts,
            entities: self.entities.clone(),
        }
    }

    pub fn all_possible_entity_bindings(
        &self,
        system: &System,
    ) -> Vec<HashMap<String, Value>> {
        self.entities
            .iter()
            .filter_map(|decl| {
                system
                    .entities_in_classes
                    .get(&decl.class)
                    .map(|e| (&decl.binding, e))
            })
            .map(|(binding, entities)| {
                entities
                    .iter()
                    .map(|e| (binding.clone(), Value::EntityId(e.clone())))
                    .collect_vec()
            })
            .multi_cartesian_product()
            .map(|mappings| mappings.into_iter().collect())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ICst {
    pub cst_id: String,
    pub pattern: Pattern,
}

impl ICst {
    /// "Instantiate" cst using the pattern, so bindings are turned into params (which could be other bindings)
    pub fn expand_cst(&self, data: &System) -> Cst {
        let mut cst = data
            .csts
            .get(&self.cst_id)
            .expect(&format!(
                "Invalid cst id {}. Cst was likely deleted but model with icst was not",
                self.cst_id
            ))
            .clone();
        let binding_params = cst
            .binding_params()
            .into_iter()
            .zip(&self.pattern)
            .collect::<HashMap<_, _>>();
        cst.facts = cst
            .facts
            .into_iter()
            .map(|mut f| {
                match &f.pattern.value {
                    PatternItem::Binding(name) => {
                        f.pattern.value = binding_params[name].clone();
                    }
                    _ => {}
                }
                match &f.pattern.entity_id {
                    EntityPatternValue::Binding(name) => {
                        f.pattern.entity_id = match binding_params[name] {
                            PatternItem::Binding(b) => EntityPatternValue::Binding(b.clone()),
                            PatternItem::Value(Value::EntityId(id)) => EntityPatternValue::EntityId(id.clone()),
                            _ => f.pattern.entity_id
                        };
                    }
                    _ => {}
                }

                f
            })
            .collect();
        cst.entities = cst.entities
            .into_iter()
            .filter_map(|e| {
                // If the parameter is another binding we rename it,
                // if it is an entity id, we just remove it since the entity declaration no longer makes sense then
                match &binding_params[&e.binding] {
                    PatternItem::Binding(name) => Some(EntityDeclaration { binding: name.to_owned(), ..e }),
                    PatternItem::Value(_) => None,
                    PatternItem::Any => panic!("Wildcard not allowed in param pattern")
                }
            })
            .collect();

        cst
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoundCst {
    pub bindings: HashMap<String, Value>,
    pub cst: Cst,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InstantiatedCst {
    pub cst_id: String,
    pub facts: Vec<Fact<AssignedMkVal>>,
    pub binding_params: Vec<String>,
    pub entity_bindings: Vec<InstantiatedCstEntityBinding>,
}

impl InstantiatedCst {
    pub fn can_instantiate_state(cst: &Cst, state: &SystemState, system: &System) -> bool {
        !Self::try_instantiate_from_current_state(cst, state, system).is_empty()
    }

    pub fn try_instantiate_from_current_state(
        cst: &Cst,
        state: &SystemState,
        system: &System,
    ) -> Vec<InstantiatedCst> {
        let mut instantiated_csts: Vec<InstantiatedCst> = Vec::new();

        for entity_bindings in cst.all_possible_entity_bindings(system) {
            let facts: Option<Vec<Fact<AssignedMkVal>>> = cst
                .facts
                .iter()
                .map(|f| {
                    let var_value = state
                        .variables
                        .get(&f.pattern.entity_key(&entity_bindings)?)?;

                    match &f.pattern.value {
                        // If a fact in the cst is a value that does not match the variable value
                        PatternItem::Value(v) if v != var_value => {
                            return None;
                        }
                        // If it is a binding for an entity (binding with a known value), that does not match the var value
                        PatternItem::Binding(b) if entity_bindings.contains_key(b) && entity_bindings[b] != *var_value => {
                            return None;
                        }
                        _ => {}
                    }

                    Some(
                        f.with_pattern(
                            f.pattern.assign_value(
                                var_value,
                                &entity_bindings,
                            ),
                        ),
                    )
                })
                .collect();

            let Some(facts) = facts else {
                continue;
            };

            // Make sure that variables with same bindings have same value
            let binding_params = cst.binding_params();
            let are_bindings_correct = binding_params.iter().all(|b| {
                facts
                    .iter()
                    .filter(
                        |f| matches!(&f.pattern.pattern_value, PatternItem::Binding(fb) if b == fb),
                    )
                    .map(|f| &f.pattern.value)
                    .all_equal()
            });

            if !are_bindings_correct {
                continue;
            }

            instantiated_csts.push(InstantiatedCst {
                cst_id: cst.cst_id.clone(),
                facts,
                binding_params,
                entity_bindings: cst
                    .entities
                    .iter()
                    .map(|e| {
                        InstantiatedCstEntityBinding::new(
                            e.binding.to_string(),
                            entity_bindings[&e.binding].as_entity_id().to_owned(),
                        )
                    })
                    .collect(),
            });
        }

        instantiated_csts
    }

    pub fn matches_param_pattern(&self, pattern: &Pattern) -> PatternMatchResult {
        let mut bindings = HashMap::new();
        for (b, p) in self.binding_params.iter().zip(pattern) {
            match p {
                PatternItem::Binding(name) => {
                    bindings.insert(name.clone(), self.get_binding_value(b).unwrap());
                }
                PatternItem::Value(value) => {
                    if self.get_binding_value(b).unwrap() != *value {
                        return PatternMatchResult::False;
                    }
                }
                // We don't need to do anything on wildcard patterns
                PatternItem::Any => {}
            }
        }

        PatternMatchResult::True(bindings)
    }

    fn get_binding_value(&self, binding: &str) -> Option<Value> {
        // If this binding is for a value inside a fact
        if let Some(fact) = self.facts.iter().find(|f| matches!(&f.pattern.pattern_value, PatternItem::Binding(b) if b == binding)) {
            Some(fact.pattern.value.clone())
        }
        // If this binding is for an entity
        else if let Some(entity_binding) = self.entity_bindings.iter().find(|e| e.binding == binding) {
            Some(Value::EntityId(entity_binding.entity_id.clone()))
        }
        else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InstantiatedCstEntityBinding {
    pub binding: String,
    pub entity_id: String,
}

impl InstantiatedCstEntityBinding {
    pub fn new(binding: String, entity_id: String) -> Self {
        Self { binding, entity_id }
    }
}