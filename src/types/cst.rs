use crate::runtime::pattern_matching::{combine_pattern_bindings, compare_patterns, extract_bindings_from_patterns, fill_in_pattern_with_bindings, pattern_item_matches_value_with_bindings, PatternMatchResult};
use crate::types::pattern::{Pattern};
use crate::types::runtime::{System, SystemState};
use crate::types::{EntityDeclaration, EntityPatternValue, Fact, MkVal, PatternItem};
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
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
            .flat_map(|fact| fact.pattern.value.get_bindings());

        entity_bindings.chain(fact_bindings).unique().collect()
    }

    pub fn fill_in_bindings(&self, bindings: &HashMap<String, Value>) -> Cst {
        let mut facts = self.facts.clone();

        for fact in &mut facts {
            fact.pattern.value.insert_binding_values(bindings);
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
    pub params: Pattern,
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
            .zip(&self.params)
            .map(|(b, v)| (b, v.to_owned()))
            .collect::<HashMap<_, _>>();
        cst.facts = cst
            .facts
            .into_iter()
            .map(|mut f| {
                f.pattern.value.insert_pattern_binding_values(&binding_params);
                match &f.pattern.entity_id {
                    EntityPatternValue::Binding(name) => {
                        f.pattern.entity_id = match &binding_params[name] {
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
                    // Possibly should panic, this should never happen
                    PatternItem::Vec(_) => None,
                    PatternItem::Any => panic!("Wildcard not allowed in param pattern")
                }
            })
            .collect();

        cst
    }

    pub fn matches(&self, bindings: &HashMap<String, Value>, other: &ICst, allow_unbound: bool, allow_different_length: bool,) -> PatternMatchResult {
        if self.cst_id == other.cst_id
            && compare_patterns(&fill_in_pattern_with_bindings(self.params.clone(), bindings), &other.params, allow_unbound, allow_different_length) {
            PatternMatchResult::True(extract_bindings_from_patterns(&self.params, &other.params))
        }
        else {
            PatternMatchResult::False
        }
    }
}

impl Display for ICst {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(icst {} {})", self.cst_id, self.params.iter().map(|p| p.to_string()).join(" "))?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoundCst {
    pub bindings: HashMap<String, Value>,
    pub cst: Cst,
}

impl BoundCst {
    pub fn try_instantiate_from_state(
        cst: &Cst,
        state: &SystemState,
        system: &System,
    ) -> Vec<BoundCst> {
        let mut instantiated_csts: Vec<BoundCst> = Vec::new();

        'entity_loop: for entity_bindings in cst.all_possible_entity_bindings(system) {
            let mut binding_map = HashMap::new();
            binding_map.extend(entity_bindings.clone());

            for fact in &cst.facts {
                // Don't initialize model if variable does not even have a value
                // Unwrapping entity key should never panic because all_possible_entity_bindings() should
                // return maps with all entities bound
                let Some(key) = &fact.pattern.entity_key(&binding_map) else {
                    continue 'entity_loop;
                };
                let Some(current_value) = state.variables.get(key) else {
                    continue 'entity_loop;
                };

                match pattern_item_matches_value_with_bindings(&fact.pattern.value, &current_value, binding_map) {
                    PatternMatchResult::True(updated_bindings) => {
                        binding_map = updated_bindings;
                    }
                    PatternMatchResult::False => {
                        continue 'entity_loop;
                    }
                }
            }

            instantiated_csts.push(BoundCst {
                bindings: binding_map,
                cst: cst.clone(),
            });
        }

        instantiated_csts
    }

    /// Create icst that would be used to instantiate this composite state, including known binding values.
    /// Can be used to compare it to icst on lhs of req. models
    pub fn icst_for_cst(&self) -> ICst {
        let binding_params = self.cst.binding_params();
        let param_pattern = binding_params.iter()
            .map(|b| match self.bindings.get(b) {
                Some(v) => PatternItem::Value(v.clone()),
                // There is no specific binding in the icst, since those depend on the model the icst is in
                None => PatternItem::Any
            })
            .collect();

        ICst {
            cst_id: self.cst.cst_id.clone(),
            params: param_pattern,
        }
    }

    // Check if this matches a ICst declaration, and if so extracts binding for that declaration
    pub fn match_and_get_bindings_for_icst(&self, icst: &ICst) -> PatternMatchResult {
        let self_icst = self.icst_for_cst();

        if icst.cst_id == self_icst.cst_id && compare_patterns(&self_icst.params, &icst.params, true, true) {
            let bindings = extract_bindings_from_patterns(&icst.params, &self_icst.params);
            PatternMatchResult::True(bindings)
        }
        else {
            PatternMatchResult::False
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