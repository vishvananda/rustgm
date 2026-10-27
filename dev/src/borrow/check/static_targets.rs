//! Solved caller equalities select callable identities without strengthening a callee proof.
use crate::support::hash::{HashMap, HashSet};
use crate::{
    body::ir::Body,
    borrow::regions,
    types::{Kind, RegionId, Store, instance::Instance, substitute::Bound},
};

pub(super) fn prepare(proof: &regions::Body) -> Result<Option<Box<Bound>>, &'static str> {
    let required = proof
        .solution
        .required_static()
        .map_err(|_| "static callable proof query exhausted")?;
    if required.is_empty() {
        return Ok(None);
    }
    let required: HashSet<_> = required.into_iter().collect();
    let mut replacements = HashMap::default();
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
    if body.has_resolved_callables() {
        return Err("callable targets already belong to a completed proof");
    }
    let mut work = 0;
    let targets = select(types, body, fold, usize::MAX, &mut work)?;
    if !targets.is_empty() {
        body.install_callable_targets(targets)?;
    }
    Ok(())
}

/// The caller supplies the exact solved regions only after completing its loans.
/// No solution means no replacement; structural item arguments grant no proof.
pub(crate) fn solved(
    types: &mut Store,
    body: &Body,
    proof: &regions::Body,
    bytes: usize,
    work: &mut usize,
) -> Result<HashMap<Instance, Instance>, &'static str> {
    if proof.key != body.key() || proof.source != body.source || proof.instance != body.instance {
        return Err("solved callable regions belong to another actual body or instance");
    }
    let Some(mut fold) = prepare(proof)? else {
        return Ok(HashMap::default());
    };
    if body.has_resolved_callables() {
        return Err("callable targets already belong to a completed proof");
    }
    select(types, body, &mut fold, bytes, work)
}

fn select(
    types: &mut Store,
    body: &Body,
    fold: &mut Bound,
    bytes: usize,
    work: &mut usize,
) -> Result<HashMap<Instance, Instance>, &'static str> {
    let mut targets = HashMap::default();
    for value in &body.values {
        *work = work.saturating_add(1);
        if !matches!(types.kind(value.ty), Some(Kind::FunctionItem { .. })) {
            continue;
        }
        let before = fold.steps;
        let selected = fold.apply(types, value.ty);
        *work = work.saturating_add(fold.steps.saturating_sub(before));
        let selected = selected?;
        if selected == value.ty {
            continue;
        }
        let original = Instance::new(types, value.ty)?;
        if !targets.contains_key(&original) {
            if crate::support::retained::new_map::<Instance, Instance>(targets.len() + 1) > bytes {
                return Err("solved callable targets exceed snapshot storage budget");
            }
            targets
                .try_reserve(1)
                .map_err(|_| "solved callable target allocation failed")?;
            if crate::support::retained::map(&targets) > bytes {
                return Err("solved callable target payload exceeds snapshot storage budget");
            }
        }
        targets.insert(original, Instance::new(types, selected)?);
    }
    Ok(targets)
}
