//! A dispatched trait method keeps one selection at every lifetime a body names it.
use super::super::{
    ir::Body,
    regions::{Contract, Kind},
};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    sema::{
        dispatch::{Dispatcher, Failure, Preparation},
        model::Declarations,
    },
    types::{ItemId, Kind as Type, TypeId},
};

/// Steps of one body's walk; a coarse guard against a runaway type graph.
const WALK_STEPS: usize = 1 << 30;

/// A body that dispatches a trait method value can name the same method
/// again at other lifetimes: a generic callee's `F`, inferred from the
/// argument of `.map(Input::conv)`, copies the value's item with fresh
/// regions into the call's result and the place that stores it. Every copy
/// selects the same implementation as the value, so the one fold rewrites
/// each occurrence and no relation compares a trait member with its
/// selected implementation.
pub(super) fn select(
    facts: &mut Declarations,
    (body, contract): (&Body, &Contract),
    dispatcher: &mut Dispatcher,
    replacements: &mut HashMap<TypeId, TypeId>,
    preparation: &mut Option<&mut dyn Preparation>,
) -> Result<(), Failure> {
    let item = |facts: &Declarations, ty| match facts.types.kind(ty) {
        Some(Type::FunctionItem { item, .. }) => Some(item),
        _ => None,
    };
    let items = replacements
        .keys()
        .filter_map(|&ty| item(facts, ty))
        .collect::<HashSet<ItemId>>();
    if items.is_empty() {
        return Ok(());
    }
    let mut roots = Vec::with_capacity(body.locals.len() + body.values.len() + 1);
    roots.push(body.result);
    roots.extend(body.locals.iter().map(|local| local.ty));
    roots.extend(body.values.iter().map(|value| value.ty));
    for obligation in &contract.obligations {
        if let Kind::Coercion {
            actual, expected, ..
        } = obligation.kind
        {
            roots.extend([actual, expected]);
        }
    }
    roots.retain(|&ty| facts.types.contains_function_items(ty) == Some(true));
    if roots.is_empty() {
        return Ok(());
    }
    let steps = std::cell::Cell::new(0_usize);
    let pay = |work: usize, _| {
        steps.set(steps.get().saturating_add(work));
        if steps.get() > WALK_STEPS {
            return Err("named trait methods exceed their runaway limit");
        }
        Ok(())
    };
    let methods = crate::sema::opaque::definitions::trait_methods(
        &facts.types,
        &facts.associated,
        &roots,
        &pay,
    )?;
    for method in methods {
        if replacements.contains_key(&method)
            || item(facts, method).is_none_or(|item| !items.contains(&item))
        {
            continue;
        }
        let selected = match preparation {
            Some(service) => dispatcher.select_prepared(facts, method, &mut **service)?,
            None => dispatcher.select(facts, method)?,
        };
        if selected != method {
            replacements.insert(method, selected);
        }
    }
    Ok(())
}
