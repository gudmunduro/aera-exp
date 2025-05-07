use crate::runtime::pattern_matching::{
    compare_commands, compare_imdls, compare_pattern_items, compare_patterns,
};
use crate::types::cst::Cst;
use crate::types::functions::Function;
use crate::types::models::{BoundModel, IMdl, Mdl, MdlLeftValue, MdlRightValue};
use crate::types::pattern::{Pattern, PatternItem};
use crate::types::runtime::System;
use crate::types::{EntityDeclaration, EntityPatternValue, Fact, MatchesFact, MkVal};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

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
    for f in &cst.facts {
        let Some(f_mapped) = map_fact_bindings(f, &binding_map) else {
            continue;
        };
        if other_cst.facts.iter().any(|f2| {
            f_mapped.pattern.var_name == f2.pattern.var_name
                && f_mapped.pattern.value == f2.pattern.value
                && f_mapped.pattern.entity_id == f2.pattern.entity_id
                && f_mapped.anti == f2.anti
        }) {
            matching_binding_set.extend(get_fact_binding_set(f));
            new_cst.facts.push(f.clone());
        }
    }
    if matching_binding_set.symmetric_difference(&binding_map.values().cloned().collect::<HashSet<_>>()).count() > 0 {
        return None;
    }

    // Check if every entity in imdl has the same class in cst
    let matching_entity_classes = cst.entities.iter()
        .filter_map(|e| {
            binding_map.get(&e.binding).map(|b| (b, &e.class))
        })
        .collect_vec();
    let entity_classes_match = matching_entity_classes.iter()
        .all(|(binding, class)| other_cst.entities.iter().any(|e| &e.binding == *binding && &e.class == *class));
    if !entity_classes_match {
        return None;
    }
    new_cst.entities = matching_entity_classes.into_iter().map(|(binding, class)| EntityDeclaration::new(binding, class)).collect();

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
            binding_map.get(binding).cloned().map(PatternItem::Binding)
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
