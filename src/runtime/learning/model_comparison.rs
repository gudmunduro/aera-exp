use crate::runtime::pattern_matching::{
    compare_commands, compare_imdls, compare_pattern_items,
};
use crate::types::cst::Cst;
use crate::types::functions::Function;
use crate::types::models::{BoundModel, IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::{Pattern, PatternItem};
use crate::types::runtime::System;
use crate::types::{EntityDeclaration, EntityPatternValue, Fact, MatchesFact, MkVal};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use crate::types::value::Value;

pub fn compare_model_effects(
    cst: &Cst,
    req_model: &Mdl,
    casual_model: &Mdl,
    other_cst: &Cst,
    other_req_model: &Mdl,
    other_causal_model: &Mdl,
    system: &System,
) -> Option<Cst> {
    // Start by comparing casual models directly (any two bindings are equal)
    // Create binding map from req_model to other_req_model (for imdl bindings)
    // Every binding in the binding map needs to exist in a fact that appears in both composite states
    // Additionally, the entity class of PE must be same in both cst's

    let are_casual_models_equal = compare_casual_models(casual_model, other_causal_model);
    if !are_casual_models_equal {
        return None;
    }

    // Create a binding map mapping from model1 bindings to model2 bindings
    let (imdl1, imdl2) = match (&req_model.right.pattern, &other_req_model.right.pattern) {
        (MdlRightValue::IMdl(imdl1), MdlRightValue::IMdl(imdl2))
            if imdl1.params.len() == imdl2.params.len() =>
        {
            (imdl1, imdl2)
        }
        _ => return None,
    };
    let mut binding_map = HashMap::new();
    if !construct_binding_map(imdl1, imdl2, &mut binding_map) {
        return None;
    }

    // Check if every binding appears in a fact that is in both csts
    let mut new_cst = Cst::new(cst.cst_id.clone());
    let mut matching_binding_set = HashSet::new();
    let mut combined_cst_binding_map = CombinedCstBindingMap::new();
    let expected_bindings = binding_map.values().cloned().collect::<HashSet<_>>();
    for e in &cst.entities {
        let Some(mapped_binding) = binding_map.get(&e.binding) else {
            // Check if the other cst still has a declaration of the same class, and if it does add a new entry to the binding map.
            // This is so we will keep facts that are in both csts but whose bindings are not in the imdl
            if let Some(e2) = other_cst.entities.iter().find(|e2| e2.class == e.class) {
                // Make sure the binding is not already in a mapping
                if !binding_map.values().contains(&e2.binding) {
                    binding_map.insert(e.binding.clone(), e2.binding.clone());
                    matching_binding_set.insert(e.binding.clone());
                    let new_binding = combined_cst_binding_map.get_new_var_name(&e.binding, &e2.binding);
                    new_cst.entities.push(EntityDeclaration::new(&new_binding, &e.class));
                }
            }
            continue;
        };
        if other_cst.entities.iter().any(|e| &e.binding == mapped_binding && &e.class == &e.class) {
            matching_binding_set.insert(mapped_binding.clone());
            let new_binding = combined_cst_binding_map.get_new_var_name(&e.binding, &mapped_binding);
            new_cst.entities.push(EntityDeclaration::new(&new_binding, &e.class));
        }
    }
    for f in &cst.facts {
        let Some(f_mapped) = map_fact_bindings(f, &binding_map) else {
            log::debug!("Skipped fact {f} due to missing binding mappings");
            continue;
        };
        if let Some(f2) = other_cst.facts.iter().find(|f2| {
            f_mapped.pattern.var_name == f2.pattern.var_name
                && compare_cst_fact_pattern_items(&f_mapped.pattern.value, &f2.pattern.value)
                && f_mapped.pattern.entity_id == f2.pattern.entity_id
                && f_mapped.anti == f2.anti
        }) {
            matching_binding_set.extend(get_fact_binding_set(&f_mapped));
            new_cst.facts.push(merge_facts(f, f2, &mut combined_cst_binding_map));
        }
        else {
            log::debug!("Skipped fact {f} for not being equal");
        }
    }
    if matching_binding_set.symmetric_difference(&expected_bindings).count() > 0 {
        return None;
    }

    // TODO: An entity declaration that apppears only in cst, not imdl, and other facts using said declaration, could still be relevant
    // TODO: And we are currently dropping those facts, even if they are equivelant in both csts

    Some(new_cst)
}

fn map_fact_bindings(
    f: &Fact<MkVal>,
    binding_map: &HashMap<String, String>,
) -> Option<Fact<MkVal>> {
    let mut f = f.clone();

    f.pattern.entity_id = match f.pattern.entity_id {
        EntityPatternValue::Binding(binding) => {
            let Some(new_binding) = &binding_map.get(&binding) else {
                return None;
            };
            EntityPatternValue::Binding(new_binding.to_string())
        }
        e @ EntityPatternValue::EntityId(_) => e,
    };
    f.pattern.value = map_pattern_item_bindings(&f.pattern.value, binding_map)?;

    Some(f)
}

fn map_pattern_item_bindings(
    p: &PatternItem,
    binding_map: &HashMap<String, String>,
) -> Option<PatternItem> {
    match p {
        PatternItem::Any | PatternItem::Value(_) => Some(p.clone()),
        PatternItem::Binding(binding) => {
            // Temporary solution to allow comparing facts where not all bindings are in binding map
            Some(binding_map.get(binding).cloned().map(PatternItem::Binding).unwrap_or(PatternItem::Any))
        }
        PatternItem::Vec(v) => {
            let res: Option<Vec<PatternItem>> = v
                .iter()
                .map(|p| map_pattern_item_bindings(p, binding_map))
                .collect();
            res.map(PatternItem::Vec)
        }
    }
}

fn get_fact_binding_set(fact: &Fact<MkVal>) -> HashSet<String> {
    let mut binding_set = HashSet::new();

    if let EntityPatternValue::Binding(b) = &fact.pattern.entity_id {
        binding_set.insert(b.clone());
    }
    get_pattern_item_binding_set(&fact.pattern.value, &mut binding_set);

    binding_set
}

fn get_pattern_item_binding_set(p: &PatternItem, binding_set: &mut HashSet<String>) {
    match p {
        PatternItem::Binding(b) => {
            binding_set.insert(b.clone());
        }
        PatternItem::Vec(v) => {
            for p in v {
                get_pattern_item_binding_set(p, binding_set);
            }
        }
        PatternItem::Value(_) => {}
        PatternItem::Any => {}
    }
}

fn construct_binding_map(
    imdl1: &IMdl,
    imdl2: &IMdl,
    binding_map: &mut HashMap<String, String>,
) -> bool {
    for (p1, p2) in imdl1.params.iter().zip(&imdl2.params) {
        if !add_to_binding_map(p1, p2, binding_map) {
            return false;
        }
    }

    true
}

fn add_to_binding_map(
    p1: &PatternItem,
    p2: &PatternItem,
    binding_map: &mut HashMap<String, String>,
) -> bool {
    match (p1, p2) {
        (PatternItem::Value(v1), PatternItem::Value(v2)) => v1 == v2,
        (PatternItem::Binding(b1), PatternItem::Binding(b2)) => {
            binding_map.insert(b1.clone(), b2.clone());
            true
        }
        (PatternItem::Vec(v1), PatternItem::Vec(v2)) => v1
            .iter()
            .zip(v2)
            .all(|(v1, v2)| add_to_binding_map(v1, v2, binding_map)),
        (PatternItem::Any, PatternItem::Any) => true,
        (_, _) => false,
    }
}

fn compare_casual_models(model1: &Mdl, model2: &Mdl) -> bool {
    let lhs_equal = match (&model1.left.pattern, &model2.left.pattern) {
        (MdlLeftValue::IMdl(imdl1), MdlLeftValue::IMdl(imdl2)) => {
            compare_imdls(imdl1, imdl2, true, false)
        }
        (MdlLeftValue::Command(cmd1), MdlLeftValue::Command(cmd2)) => {
            compare_commands(cmd1, cmd2, true, false)
        }
        _ => false,
    };
    let rhs_equal = match (&model1.right.pattern, &model2.right.pattern) {
        (MdlRightValue::MkVal(mk_val1), MdlRightValue::MkVal(mk_val2)) => {
            mk_val1.matches_mk_val(mk_val2)
        }
        _ => false,
    };
    // Assumes same order of functions
    let forward_equal = model1
        .forward_computed
        .iter()
        .zip(&model2.forward_computed)
        .all(|((_, f1), (_, f2))| compare_functions(f1, f2));
    let backward_equal = model1
        .backward_computed
        .iter()
        .zip(&model2.backward_computed)
        .all(|((_, f1), (_, f2))| compare_functions(f1, f2));

    lhs_equal && rhs_equal && forward_equal && backward_equal
}

fn compare_functions(function1: &Function, function2: &Function) -> bool {
    match (function1, function2) {
        (Function::Value(p1), Function::Value(p2)) => compare_pattern_items(p1, p2, true),
        (Function::Add(v1, v2), Function::Add(v21, v22)) => {
            compare_functions(v1, v21) && compare_functions(v2, v22)
        }
        (Function::Sub(v1, v2), Function::Sub(v21, v22)) => {
            compare_functions(v1, v21) && compare_functions(v2, v22)
        }
        (Function::Mul(v1, v2), Function::Mul(v21, v22)) => {
            compare_functions(v1, v21) && compare_functions(v2, v22)
        }
        (Function::Div(v1, v2), Function::Div(v21, v22)) => {
            compare_functions(v1, v21) && compare_functions(v2, v22)
        }
        (Function::List(l1), Function::List(l2)) => {
            l1.len() == l2.len() && l1.iter().zip(l2).all(|(f1, f2)| compare_functions(f1, f2))
        }
        (Function::ConvertToEntityId(f1), Function::ConvertToEntityId(f2)) => {
            compare_functions(f1, f2)
        }
        (Function::ConvertToNumber(f1), Function::ConvertToNumber(f2)) => compare_functions(f1, f2),
        (_, _) => false,
    }
}


pub fn compare_cst_fact_patterns(
    pattern1: &Pattern,
    pattern2: &Pattern,
) -> bool {
    if pattern1.len() != pattern2.len() {
        return false;
    }

    pattern1.iter().zip(pattern2).all(|(p1, p2)| compare_cst_fact_pattern_items(p1, p2))
}

pub fn compare_cst_fact_pattern_items(
    pattern_item1: &PatternItem,
    pattern_item2: &PatternItem,
) -> bool {
    match pattern_item1 {
        PatternItem::Any => true,
        PatternItem::Binding(b1) =>  match pattern_item2 {
            PatternItem::Binding(b2) => b1 == b2,
            _ => true
        },
        PatternItem::Value(v1) => match pattern_item2 {
            PatternItem::Any => true,
            PatternItem::Binding(_) => false,
            PatternItem::Value(v2) => v1 == v2,
            PatternItem::Vec(v2) => match v1 {
                Value::Vec(v1) => v1 == v2,
                _ => false,
            },
        },
        PatternItem::Vec(v1) => match pattern_item2 {
            PatternItem::Any => true,
            PatternItem::Binding(_) => false,
            PatternItem::Vec(v2) => compare_cst_fact_patterns(v1, v2),
            PatternItem::Value(Value::Vec(v2)) => v2 == v1,
            PatternItem::Value(_) => false,
        }
    }
}

fn merge_facts(f1: &Fact<MkVal>, f2: &Fact<MkVal>, combined_cst_binding_map: &mut CombinedCstBindingMap) -> Fact<MkVal> {
    let merged_entity_ids = match (&f1.pattern.entity_id, &f2.pattern.entity_id) {
        (EntityPatternValue::Binding(b1), EntityPatternValue::Binding(b2)) => EntityPatternValue::Binding(combined_cst_binding_map.get_new_var_name(b1, b2)),
        // Assume both entity ids are the same
        (EntityPatternValue::EntityId(e1), EntityPatternValue::EntityId(e2)) => EntityPatternValue::EntityId(e1.clone()),
        _ => panic!("Cannot merge entity id with binding"),
    };
    let merged_value = merge_pattern_items_for_csts(&f1.pattern.value, &f2.pattern.value, combined_cst_binding_map);

    f1.with_pattern(MkVal {
        entity_id: merged_entity_ids,
        var_name: f1.pattern.var_name.clone(),
        value: merged_value,
        assumption: f1.pattern.assumption,
    })
}

fn merge_pattern_items_for_csts(v1: &PatternItem, v2: &PatternItem, combined_cst_binding_map: &mut CombinedCstBindingMap) -> PatternItem {
    match (v1, v2) {
        (PatternItem::Any, PatternItem::Any) => PatternItem::Any,
        (PatternItem::Binding(b1), PatternItem::Binding(b2)) => {
            let new_binding = combined_cst_binding_map.get_new_var_name(b1, b2);
            PatternItem::Binding(new_binding)
        },
        (PatternItem::Value(v1), PatternItem::Value(v2)) => {
            // We assume the values are always the same
            PatternItem::Value(v1.clone())
        },
        (PatternItem::Vec(v1), PatternItem::Vec(v2)) => PatternItem::Vec(v1.iter().zip(v2).map(|(v1, v2)| merge_pattern_items_for_csts(v1, v2, combined_cst_binding_map)).collect()),
        _ => panic!("Different patterns found, cannot merge")
    }
}

// Binding map that maps a pair of bindings (from two csts) into a single binding
struct CombinedCstBindingMap {
    map: HashMap<(String, String), String>,
    prefixed_var_counter: HashMap<String, usize>,
}

impl CombinedCstBindingMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            prefixed_var_counter: HashMap::new(),
        }
    }

    pub fn add_new_var(&mut self, v1: &str, v2: &str) -> String {
        let var_name = match v1 {
            // Special cases
            "PE" | "CMD_E" => v1.to_string(),
            _ => {
                // Always prefer prefix of v1
                let prefix = get_var_prefix(v1);
                let var_index = self.prefixed_var_counter.entry(prefix.clone()).or_insert(0);
                let var_name = format!("{prefix}{var_index}");
                *var_index += 1;
                var_name
            }
        };


        let (mv1, mv2) = order_vars(v1, v2);
        self.map.insert((mv1.to_string(), mv2.to_string()), var_name.to_string());
        var_name
    }

    pub fn get_new_var_name(&mut self, v1: &str, v2: &str) -> String {
        let (mv1, mv2) = order_vars(v1, v2);
        match self.map.get(&(mv1.to_string(), mv2.to_string())) {
            Some(v) => v.clone(),
            None => {
                self.add_new_var(v1, v2)
            }
        }
    }
}

fn order_vars<'a>(v1: &'a str, v2: &'a str) -> (&'a str, &'a str) {
    if v1 < v2 {
        (v1, v2)
    }
    else {
        (v2, v1)
    }
}

fn get_var_prefix(var: &str) -> String {
    if var.starts_with("P") {
        "P".to_string()
    }
    else if var.starts_with("CMD") {
        "CMD".to_string()
    }
    else if var.starts_with("C") {
        "C".to_string()
    }
    else {
        "v".to_string()
    }
}
