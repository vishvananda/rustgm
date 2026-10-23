//! Memoized callable edges are independent from the lexical body-owner forest.
use super::{Checked, DefId, DefKind, Export, GenericRequest, ItemId, Rvalue};
use crate::{
    support::{
        crates::{ExternalDef, ExternalKind},
        hash::{HashMap, HashSet},
    },
    types::Kind,
};
pub(super) fn collect(
    export: &mut Export<'_>,
    checked: &Checked<'_>,
    roots: &[GenericRequest],
) -> Result<Option<Vec<GenericRequest>>, &'static str> {
    export.reserve(
        roots
            .len()
            .saturating_mul(4 * (size_of::<GenericRequest>() + size_of::<(DefId, usize)>())),
    )?;
    let mut requests = roots.to_vec();
    let mut indices = HashMap::with_capacity_and_hasher(
        roots.len(),
        crate::support::hash::BuildFxHasher::default(),
    );
    for request in roots {
        let cost = comparison_cost(export, &request.external)?;
        if indices.insert(request.definition, cost).is_some() {
            return Err("generic roots repeat an actual declaration");
        }
    }
    let mut ports = HashSet::default();
    let mut used = false;
    let mut cursor = 0;
    while cursor < requests.len() {
        let caller = requests[cursor].clone();
        let body = &checked
            .get(ItemId(caller.definition.0))
            .ok_or("missing checked callable dependency original")?
            .body;
        for value in &body.values {
            export.charge(1)?;
            if let Rvalue::Call { .. } = value.kind {
                used = true;
            }
            let Rvalue::Function(target) = value.kind else {
                continue;
            };
            used = true;
            let Some(Kind::FunctionItem { item, .. }) = export.facts.types.kind(value.ty) else {
                return Err("function dependency lost its complete selected item");
            };
            if item.0 != target.0 {
                return Err("function dependency changed its exact target");
            }
            if ports.contains(&target) {
                continue;
            }
            export.reserve(4 * size_of::<DefId>())?;
            ports.insert(target);
            validate(export, target)?;
            if indices.contains_key(&target) {
                continue;
            }
            let external = identity(export, &caller, target)?;
            export.reserve(
                4 * (size_of::<GenericRequest>()
                    + size_of::<(DefId, usize)>()
                    + size_of::<DefId>()),
            )?;
            indices.insert(target, comparison_cost(export, &external)?);
            export.private.insert(target);
            requests.push(GenericRequest {
                definition: target,
                external,
            });
        }
        cursor += 1;
    }
    if !used {
        return Ok(None);
    }
    order(&mut requests, &indices, &mut export.steps)?;
    Ok(Some(requests))
}
fn validate(export: &Export<'_>, target: DefId) -> Result<(), &'static str> {
    let resolution = &export.report.resolution;
    let actual = resolution
        .definitions
        .get(target.0 as usize)
        .ok_or("unknown callable dependency declaration")?;
    let scope = resolution
        .scopes
        .get(actual.scope.0 as usize)
        .ok_or("callable dependency has no actual scope")?;
    if actual.kind != DefKind::Function
        || actual.node.is_none()
        || resolution.module_scopes[scope.module.0 as usize] != actual.scope
    {
        return Err("callable dependency requires its actual source module-level declaration");
    }
    let scheme = export
        .facts
        .lifetimes
        .functions
        .get(&target)
        .ok_or("callable dependency has no completed lifetime scheme")?;
    if scheme.total != 0
        || scheme.captures != 0
        || !scheme.early.is_empty()
        || !scheme.bounds.is_empty()
        || export
            .facts
            .generics
            .parameters_in_scope(target)
            .next()
            .is_none()
    {
        return Err("callable dependency requires a type-generic zero-region original");
    }
    Ok(())
}

fn identity(
    export: &mut Export<'_>,
    caller: &GenericRequest,
    target: DefId,
) -> Result<ExternalDef, &'static str> {
    let resolution = &export.report.resolution;
    let actual = &resolution.definitions[target.0 as usize];
    if actual.scope != resolution.definitions[caller.definition.0 as usize].scope {
        return Err("private helper export requires the same defining module");
    }
    let names = caller.external.path();
    let name = export.report.package.names.get(actual.name);
    let mut growth = names.len().saturating_mul(size_of::<String>());
    for value in names {
        export.charge(value.len().saturating_add(1))?;
        growth = growth.saturating_add(value.len());
    }
    export.reserve(
        growth
            .saturating_add(name.len())
            .saturating_add(2 * size_of::<usize>()),
    )?;
    let mut path = names.to_vec();
    let last = path
        .last_mut()
        .ok_or("callable dependency lacks a defining module path")?;
    name.clone_into(last);
    ExternalDef::new(
        caller.external.owner().clone(),
        path,
        ExternalKind::Function,
    )
}

fn comparison_cost(export: &mut Export<'_>, identity: &ExternalDef) -> Result<usize, &'static str> {
    let mut cost = identity
        .owner()
        .name()
        .len()
        .saturating_add(identity.owner().disambiguator().len());
    for name in identity.path() {
        export.charge(1)?;
        cost = cost.saturating_add(name.len().saturating_add(1));
    }
    Ok(cost)
}

fn order(
    requests: &mut [GenericRequest],
    indices: &HashMap<DefId, usize>,
    steps: &mut usize,
) -> Result<(), &'static str> {
    for request in requests.iter() {
        super::charge(steps, 1)?;
        if !indices.contains_key(&request.definition) {
            return Err("canonical dependency has no observed comparison key");
        }
    }
    let mut comparisons = 0_usize;
    requests.sort_by(|a, b| {
        // Charge a conservative complete-key comparison bound. Key sizes were
        // observed once; sorting never walks bodies or recomputes key lengths.
        comparisons = comparisons
            .saturating_add(1)
            .saturating_add(indices.get(&a.definition).copied().unwrap_or(usize::MAX))
            .saturating_add(indices.get(&b.definition).copied().unwrap_or(usize::MAX));
        a.external.cmp(&b.external)
    });
    super::charge(steps, comparisons)?;
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/lexicographic.rs"]
mod tests;
