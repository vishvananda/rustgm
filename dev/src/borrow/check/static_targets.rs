//! Solved caller equalities select callable identities without strengthening a callee proof.
use crate::{
    body::ir::Body,
    borrow::regions,
    types::{Kind, RegionId, Store, instance::Instance, substitute::Bound},
};
use std::collections::{HashMap, HashSet};

pub(super) fn prepare(proof: &regions::Body) -> Result<Option<Box<Bound>>, &'static str> {
    let required = proof
        .solution
        .required_static()
        .map_err(|_| "static callable proof query exhausted")?;
    if required.is_empty() {
        return Ok(None);
    }
    let required: HashSet<_> = required.into_iter().collect();
    let mut replacements = HashMap::new();
    let mut changed = false;
    for (original, region) in proof.identities.variables() {
        let replacement = if required.contains(&region) {
            changed = true;
            RegionId::STATIC
        } else {
            original
        };
        replacements.insert(original, replacement);
    }
    for (index, region) in proof.identities.universals().enumerate() {
        let original = RegionId::bound(
            0,
            u16::try_from(index).map_err(|_| "too many callable proof universals")?,
        )?;
        let replacement = if required.contains(&region) {
            changed = true;
            RegionId::STATIC
        } else {
            original
        };
        replacements.insert(original, replacement);
    }
    Ok(changed.then(|| Box::new(Bound::restore_regions(replacements))))
}

pub(super) fn resolve(
    types: &mut Store,
    body: &mut Body,
    fold: &mut Bound,
) -> Result<(), &'static str> {
    if body.resolved_callables.is_some() {
        return Err("callable targets already belong to a completed proof");
    }
    for value in &body.values {
        if !matches!(types.kind(value.ty), Some(Kind::FunctionItem { .. })) {
            continue;
        }
        let selected = fold.apply(types, value.ty)?;
        if selected != value.ty {
            body.resolved_callables
                .get_or_insert_with(Default::default)
                .insert(
                    Instance::new(types, value.ty)?,
                    Instance::new(types, selected)?,
                );
        }
    }
    Ok(())
}
