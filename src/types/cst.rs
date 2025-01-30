use crate::runtime::pattern_matching::PatternMatchResult;
use crate::types::pattern::{Pattern, PatternValue};
use crate::types::runtime::{AssignedMkVal, RuntimeValue, System, SystemState};
use crate::types::{EntityDeclaration, EntityPatternValue, Fact, MkVal, PatternItem};
use itertools::Itertools;
use std::collections::HashMap;

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

    pub fn fill_in_bindings(&self, bindings: &HashMap<String, RuntimeValue>) -> Cst {
        let mut facts = self.facts.clone();

        for fact in &mut facts {
            match &fact.pattern.value {
                PatternItem::Binding(b) => {
                    if let Some(binding_val) = bindings.get(b) {
                        fact.pattern.value = PatternItem::Value(binding_val.clone().into());
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
    ) -> Vec<HashMap<String, RuntimeValue>> {
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
                    .map(|e| (binding.clone(), RuntimeValue::EntityId(e.clone())))
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
                            PatternItem::Value(PatternValue::EntityId(id)) => EntityPatternValue::EntityId(id.clone()),
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
    pub bindings: HashMap<String, RuntimeValue>,
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
                    Some(
                        f.with_pattern(
                            f.pattern.assign_value(
                                state
                                    .variables
                                    .get(&f.pattern.entity_key(&entity_bindings)?)?,
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

    pub fn matches_param_pattern(
        &self,
        pattern: &Pattern,
        assigned_bindings: &HashMap<String, RuntimeValue>,
    ) -> PatternMatchResult {
        // TODO: Simplify function so it does not work with prior bindings on model, as that feature is never used and makes everything more complicated
        if pattern.len() != self.binding_params.len() {
            return PatternMatchResult::False;
        }

        let mut returned_binding_values = assigned_bindings.clone();
        let mut binding_values = HashMap::new();
        for (p, b) in pattern.iter().zip(&self.binding_params) {
            match p {
                PatternItem::Binding(name) => {
                    binding_values.insert(
                        name.to_owned(),
                        match returned_binding_values.get(name) {
                            Some(value) => BindingValue::BoundVariable(name.clone(), value.clone()),
                            None => BindingValue::UnboundVariable(name.clone()),
                        },
                    );
                }
                PatternItem::Any => {
                    binding_values.insert(b.to_owned(), BindingValue::Any);
                }
                PatternItem::Value(value) => {
                    binding_values
                        .insert(b.to_owned(), BindingValue::Value(value.to_owned().into()));
                }
            }
        }

        for entity_binding in &self.entity_bindings {
            match binding_values.get(&entity_binding.binding) {
                Some(b) => match b {
                    BindingValue::Any => {
                        // Automatic match
                    }
                    BindingValue::Value(value) => {
                        if value.as_entity_id() != entity_binding.entity_id {
                            return PatternMatchResult::False;
                        }
                    }
                    BindingValue::BoundVariable(_, value) => {
                        if value.as_entity_id() != entity_binding.entity_id {
                            return PatternMatchResult::False;
                        }
                    }
                    BindingValue::UnboundVariable(name) => {
                        returned_binding_values
                            .insert(name.clone(), RuntimeValue::EntityId(entity_binding.entity_id.clone()));
                    }
                }
                None => return PatternMatchResult::False,
            }
        }

        for fact in self.facts.iter() {
            match &fact.pattern.pattern_value {
                PatternItem::Binding(binding_name) => {
                    match binding_values.get(binding_name) {
                        Some(b) => match b {
                            BindingValue::Any => {
                                // Automatic match
                            }
                            BindingValue::Value(value) => {
                                if value != &fact.pattern.value {
                                    return PatternMatchResult::False;
                                }
                            }
                            BindingValue::BoundVariable(_, value) => {
                                if value != &fact.pattern.value {
                                    return PatternMatchResult::False;
                                }
                            }
                            BindingValue::UnboundVariable(name) => {
                                // If the variable is used in more then one place (and therefore previously bound)
                                // we need to make sure it has the same value everywhere
                                if let Some(value) = returned_binding_values.get(name) {
                                    if value != &fact.pattern.value {
                                        return PatternMatchResult::False;
                                    }
                                } else {
                                    returned_binding_values
                                        .insert(name.clone(), fact.pattern.value.clone());
                                }
                            }
                        },
                        None => return PatternMatchResult::False,
                    }
                }
                // Any and value should not need to be checked
                PatternItem::Any => {}
                PatternItem::Value(v) => {
                    if fact.pattern.value == *v {
                        panic!("Cst does not match current state when matching with model (cst should have never been instantiated) ({})", self.cst_id)
                    }
                }
            }
        }

        PatternMatchResult::True(returned_binding_values)
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

// TODO: Find new place for this enum (or remove after simplifying function)

#[derive(Clone)]
enum BindingValue {
    Any,
    Value(RuntimeValue),
    #[allow(unused)]
    BoundVariable(String, RuntimeValue),
    UnboundVariable(String),
}
