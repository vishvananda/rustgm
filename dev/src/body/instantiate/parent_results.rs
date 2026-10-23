//! A proved selected parent transports only its prepared anonymous request roots.
use super::Declarations;
use crate::support::hash::{HashMap, HashSet};
use crate::{
    body::{
        ir::{Action, Body, BodyKey, Owner, Rvalue, ValueId},
        regions::{Contract, Kind},
    },
    sema::{
        inline_constants::Context,
        values::{Instance, execution::SourceComputation},
    },
    types::{
        RegionId, TypeId,
        substitute::{Bound, RegionEquations},
    },
};
const WORK: usize = 1_000_000;
const BYTES: usize = 64 * 1024 * 1024;
#[cfg(test)]
#[path = "../../../tests/invariants/parent_requests.rs"]
mod tests;

pub(crate) fn prepare(
    facts: &mut Declarations,
    body: &Body,
    contract: &Contract,
    witness: SourceComputation,
) -> Result<Vec<(ValueId, TypeId)>, &'static str> {
    let mut work = 0;
    let result = (|| {
        let selected = witness.resolve(facts, witness.result(facts)?)?;
        let parent = match body.owner {
            Owner::InitializerInstance { instance, .. } => {
                instance == selected.selected
                    && body.source == selected.original.into()
                    && contract.key == BodyKey::InitializerInstance(instance)
            }
            Owner::AnonymousInstance(instance) => {
                crate::body::ir::BodyOrigin::from(instance.source()) == body.source
                    && instance.enclosing() == Context::SourceInitializer(witness)
                    && crate::sema::inline_constants::InlineInstance::select(
                        facts,
                        instance.source(),
                        instance.enclosing(),
                    )? == instance
            }
            _ => false,
        };
        if !parent
            || contract.source != body.source
            || contract.key != body.key()
            || body.instance.is_some()
            || contract.output.is_some()
        {
            return Err("inline requests require their exact proved source parent");
        }
        let mut roots = roots(facts, body, witness, &mut work)?;
        if roots.is_empty() {
            return Ok(roots);
        }
        let scratch = crate::support::retained::vector(&roots);
        let mut fold = folder(
            facts,
            contract,
            selected.selected,
            &roots,
            scratch,
            &mut work,
        )?;
        let result = (|| {
            for (_, ty) in &mut roots {
                *ty = fold.apply(&mut facts.types, *ty)?;
            }
            Ok::<_, &'static str>(())
        })();
        work = work.saturating_add(fold.steps);
        result?;
        Ok(roots)
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    result
}
fn roots(
    facts: &Declarations,
    body: &Body,
    witness: SourceComputation,
    work: &mut usize,
) -> Result<Vec<(ValueId, TypeId)>, &'static str> {
    let mut roots = Vec::new();
    for statement in &body.statements {
        *work = work.saturating_add(1);
        let Action::Evaluate(id) = statement.action else {
            continue;
        };
        let value = body
            .values
            .get(id.0 as usize)
            .ok_or("missing prepared inline value")?;
        if let Rvalue::InlineInstance(instance) = value.kind {
            if instance.enclosing() != Context::SourceInitializer(witness)
                || !facts
                    .inline_constants
                    .as_deref()
                    .and_then(|r| r.get(instance.source()))
                    .is_some_and(|row| {
                        crate::body::ir::BodyOrigin::from(row.parent_body) == body.source
                    })
                || crate::sema::inline_constants::InlineInstance::select(
                    facts,
                    instance.source(),
                    instance.enclosing(),
                )? != instance
            {
                return Err("prepared inline request belongs to another source parent");
            }
            if roots.len() == roots.capacity() {
                let next = roots.capacity().saturating_mul(2).max(4);
                check(*work, next.saturating_mul(size_of::<(ValueId, TypeId)>()))?;
                roots.reserve_exact(next - roots.len());
            }
            roots.push((id, value.ty));
        }
    }
    check(*work, 0)?;
    Ok(roots)
}
fn observe(
    facts: &mut Declarations,
    roots: &[(ValueId, TypeId)],
    selected: Option<Instance>,
    scratch: usize,
    work: &mut usize,
) -> Result<Bound, &'static str> {
    let mut fold =
        Bound::observe_budgeted_regions(WORK.saturating_sub(*work), BYTES.saturating_sub(scratch))?;
    let result = (|| {
        for (_, ty) in roots {
            fold.apply(&mut facts.types, *ty)?;
        }
        if let Some(instance) = selected {
            fold.apply_arguments(&mut facts.types, instance.arguments)?;
        }
        Ok::<_, &'static str>(())
    })();
    *work = work.saturating_add(fold.steps);
    result?;
    Ok(fold)
}
fn folder(
    facts: &mut Declarations,
    contract: &Contract,
    selected: Instance,
    roots: &[(ValueId, TypeId)],
    scratch: usize,
    work: &mut usize,
) -> Result<Bound, &'static str> {
    // Observe the two namespaces separately. Actual arguments can numerically
    // coincide with original private proof IDs; they are inserted only AFTER
    // the exact original alias has been restored to its declaration slot.
    let observer = observe(facts, roots, None, scratch, work)?;
    let (bytes, visits) = observer.retained_allocation();
    *work = work.saturating_add(visits);
    let arguments = observe(
        facts,
        &[],
        Some(selected),
        scratch.saturating_add(bytes),
        work,
    )?;
    let (argument_bytes, visits) = arguments.retained_allocation();
    *work = work.saturating_add(visits);
    let count = observer.observed_regions().count();
    let external_count = arguments.observed_regions().count();
    let (owned_count, equations) = counts(contract, work)?;
    let argument_count = facts
        .types
        .arguments(selected.arguments)
        .ok_or("missing selected parent arguments")?
        .len();
    let extra = Bound::projection_closure_preflight(equations, 0)
        .saturating_add(crate::support::retained::new_map::<RegionId, ()>(
            owned_count,
        ))
        .saturating_add(crate::support::retained::new_map::<RegionId, ()>(
            external_count,
        ))
        .saturating_add(crate::support::retained::new_map::<RegionId, RegionId>(
            count,
        ))
        .saturating_add(argument_count.saturating_mul(128));
    check(
        *work,
        scratch
            .saturating_add(bytes)
            .saturating_add(argument_bytes)
            .saturating_add(extra),
    )?;
    let external: HashSet<_> = arguments.observed_regions().collect();
    *work = work
        .saturating_add(external_count)
        .saturating_add(argument_count);
    drop(arguments);
    let actual = selected.selected_arguments(facts)?;
    let (owned, mut equations) = identities(contract, owned_count, work)?;
    let mut replacements =
        HashMap::with_capacity_and_hasher(count, crate::support::hash::BuildFxHasher::default());
    for region in observer.observed_regions() {
        let replacement = replacement(
            region,
            &external,
            &owned,
            &actual.regions,
            &mut equations,
            work,
        )?;
        replacements.insert(region, replacement);
    }
    drop(observer);
    drop(external);
    drop(equations);
    drop(owned);
    Bound::restore_budgeted_regions(
        replacements,
        WORK.saturating_sub(*work),
        BYTES.saturating_sub(scratch),
    )
}
fn counts(contract: &Contract, work: &mut usize) -> Result<(usize, usize), &'static str> {
    *work = work.saturating_add(contract.obligations.len());
    let mut owned = contract.obligations.len();
    let mut equations = contract.obligations.len();
    for proof in &contract.proofs {
        for range in proof.proof.owned_regions() {
            *work = work.saturating_add(1);
            let length = range
                .end
                .checked_sub(range.start)
                .filter(|_| range.start != 0 && range.end < 0x8000_0000)
                .ok_or("inline parent proof has an invalid owned range")?;
            owned = owned.saturating_add(length as usize);
        }
        for _ in proof.proof.root_equalities() {
            *work = work.saturating_add(1);
            equations = equations.saturating_add(1);
        }
    }
    check(*work, 0)?;
    Ok((owned, equations))
}
fn identities(
    contract: &Contract,
    count: usize,
    work: &mut usize,
) -> Result<(HashSet<RegionId>, RegionEquations), &'static str> {
    let mut owned =
        HashSet::with_capacity_and_hasher(count, crate::support::hash::BuildFxHasher::default());
    for obligation in &contract.obligations {
        *work = work.saturating_add(1);
        if let Kind::Variable { id, .. } = obligation.kind {
            owned.insert(id);
        }
    }
    for proof in &contract.proofs {
        for id in proof.proof.owned_regions().flatten() {
            *work = work.saturating_add(1);
            owned.insert(RegionId(id));
        }
    }
    check(*work, 0)?;
    let mut equations = RegionEquations::default();
    equations.extend_counted(
        contract
            .obligations
            .iter()
            .filter_map(|row| {
                if let Kind::Equality(pair) = row.kind {
                    Some(pair)
                } else {
                    None
                }
            })
            .chain(
                contract
                    .proofs
                    .iter()
                    .flat_map(|row| row.proof.root_equalities()),
            ),
        work,
    )?;
    Ok((owned, equations))
}
fn replacement(
    region: RegionId,
    external: &HashSet<RegionId>,
    owned: &HashSet<RegionId>,
    actual: &[RegionId],
    equations: &mut RegionEquations,
    work: &mut usize,
) -> Result<RegionId, &'static str> {
    *work = work.saturating_add(1);
    if let Some((0, slot)) = region.bound_parts() {
        return actual
            .get(usize::from(slot))
            .copied()
            .ok_or("inline request has a foreign selected lifetime slot");
    }
    if external.contains(&region) {
        if owned.contains(&region) {
            return Err("inline parent local overlaps a selected external lifetime");
        }
        return Ok(region);
    }
    if !owned.contains(&region) {
        return Err("inline request has a foreign parent lifetime");
    }
    let anchor = equations.canonical_anchor(region, work)?;
    if anchor == RegionId::STATIC {
        Ok(anchor)
    } else if let Some((0, slot)) = anchor.bound_parts() {
        actual
            .get(usize::from(slot))
            .copied()
            .ok_or("inline parent alias has no selected lifetime slot")
    } else if external.contains(&anchor) && !owned.contains(&anchor) {
        Ok(anchor)
    } else {
        Err("inline request retains an unanchored parent lifetime")
    }
}
fn check(work: usize, bytes: usize) -> Result<(), &'static str> {
    if work > WORK || bytes > BYTES {
        Err("inline parent result transport budget exceeded")
    } else {
        Ok(())
    }
}
