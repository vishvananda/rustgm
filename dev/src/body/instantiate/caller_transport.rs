//! Alpha-rename source-owned facts before simultaneous external substitutions.
use super::Declarations;
use crate::{
    body::{
        ir::{Body, ProjectionKind, Rvalue},
        regions::{Contract, Kind},
    },
    sema::values::execution::{
        CallerInstance, caller_namespace_available, caller_published, with_namespace,
        with_registered_namespace,
    },
    types::{lifetime_proof::OwnedTransport, substitute::Bound},
};
use std::ops::Range;

pub(super) fn prepare_body(
    facts: &mut Declarations,
    source: &Body,
    body: &mut Body,
    caller: CallerInstance,
) -> Result<(), &'static str> {
    for value in &mut body.values {
        if let Rvalue::SourceAssociated(prepared) = value.kind {
            prepared.validate_use(
                facts,
                source.source,
                source
                    .owner
                    .source_caller(facts, source.source, source.instance)?,
                value.ty,
            )?;
            value.kind = Rvalue::AssociatedConstant(prepared.input(facts)?);
        }
    }
    self::body(facts, body, caller)
}

pub(super) fn contract(
    facts: &mut Declarations,
    contract: &mut Contract,
    caller: CallerInstance,
) -> Result<(), &'static str> {
    if matches!(caller, CallerInstance::Original) || !caller_published(facts, contract.source) {
        return Ok(());
    }
    if contract.output.is_some() {
        return Err("source caller namespace requires its nested output transport");
    }
    let (owned, visits) = segments(contract, caller_namespace_available(facts))?;
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(visits);
    with_namespace(
        facts,
        contract.source,
        caller,
        contract.universals,
        &owned,
        crate::support::retained::vector(&owned),
        |facts, transport| map_contract(facts, contract, transport),
    )
}
fn segments(contract: &Contract, bytes: usize) -> Result<(Vec<Range<u32>>, usize), &'static str> {
    let mut owned = Vec::<Range<u32>>::new();
    let mut visits = contract
        .obligations
        .len()
        .saturating_add(contract.proofs.len());
    let mut add = |range: Range<u32>| -> Result<(), &'static str> {
        visits = visits.saturating_add(1);
        if range.is_empty() {
            return Ok(());
        }
        if owned.len() >= 1_000_000 {
            return Err("source caller segment budget exceeded");
        }
        if owned.len() == owned.capacity() {
            let capacity = owned.capacity().saturating_mul(2).max(4);
            if capacity.saturating_mul(size_of::<Range<u32>>()) > bytes {
                return Err("source caller segment storage budget exceeded");
            }
            owned.reserve_exact(capacity - owned.len());
        }
        owned.push(range);
        Ok(())
    };
    for obligation in &contract.obligations {
        if let Kind::Variable { id, .. } = obligation.kind {
            add(id.0..id
                .0
                .checked_add(1)
                .ok_or("source caller region identity overflow")?)?;
        }
    }
    for evidence in &contract.proofs {
        for range in evidence.proof.owned_regions() {
            add(range)?;
        }
    }
    owned.sort_unstable_by(|a, b| {
        visits = visits.saturating_add(1);
        (a.start, a.end).cmp(&(b.start, b.end))
    });
    let mut count = 0_usize;
    for n in 0..owned.len() {
        visits = visits.saturating_add(1);
        if count > 0 && owned[n].start <= owned[count - 1].end {
            owned[count - 1].end = owned[count - 1].end.max(owned[n].end);
        } else {
            owned[count] = owned[n].clone();
            count += 1;
        }
    }
    owned.truncate(count);
    Ok((owned, visits))
}
fn map_contract(
    facts: &mut Declarations,
    contract: &mut Contract,
    transport: &mut OwnedTransport,
) -> Result<(), &'static str> {
    transport.steps = transport
        .steps
        .saturating_add(contract.obligations.len())
        .saturating_add(contract.proofs.len())
        .saturating_add(contract.assumptions.len())
        .saturating_add(contract.type_assumptions.len());
    let fold = &mut transport.fold;
    if let Some(signature) = &mut contract.signature {
        *signature = fold.signature(&mut facts.types, *signature, contract.universals)?;
    }
    for pair in &mut contract.assumptions {
        pair.0 = fold.region(pair.0, 0)?;
        pair.1 = fold.region(pair.1, 0)?;
    }
    for clause in &mut contract.type_assumptions {
        *clause = fold.clause(&mut facts.types, *clause)?;
    }
    for obligation in &mut contract.obligations {
        obligation_kind(facts, &mut obligation.kind, fold)?;
    }
    if let Some(pending) = &mut contract.builtins {
        let pending = std::sync::Arc::make_mut(pending);
        pending.environment = None;
        for (_, predicate) in &mut pending.predicates {
            *predicate = fold.predicate(&mut facts.types, *predicate)?;
        }
    }
    for evidence in &mut contract.proofs {
        transport.proof(&mut facts.types, &mut evidence.proof)?;
    }
    Ok(())
}
fn obligation_kind(
    facts: &mut Declarations,
    kind: &mut Kind,
    fold: &mut Bound,
) -> Result<(), &'static str> {
    match kind {
        Kind::Variable { id, .. } | Kind::LocalBorrow(id) => *id = fold.region(*id, 0)?,
        Kind::Equality(pair) | Kind::Outlives(pair) => {
            pair.0 = fold.region(pair.0, 0)?;
            pair.1 = fold.region(pair.1, 0)?;
        }
        Kind::Membership { region, captures } => {
            *region = fold.region(*region, 0)?;
            *captures = fold.apply_arguments(&mut facts.types, *captures)?;
        }
        Kind::Coercion {
            actual, expected, ..
        } => {
            *actual = fold.apply(&mut facts.types, *actual)?;
            *expected = fold.apply(&mut facts.types, *expected)?;
        }
        Kind::ScopedWellFormed { ty, .. } => *ty = fold.apply_inside(&mut facts.types, *ty, 1)?,
        Kind::TypeOutlives { ty, shorter } => {
            *ty = fold.apply(&mut facts.types, *ty)?;
            *shorter = fold.region(*shorter, 0)?;
        }
        Kind::WellFormed(ty) => *ty = fold.apply(&mut facts.types, *ty)?,
    }
    Ok(())
}
pub(super) fn body(
    facts: &mut Declarations,
    body: &mut Body,
    caller: CallerInstance,
) -> Result<(), &'static str> {
    if matches!(caller, CallerInstance::Original) || !caller_published(facts, body.source) {
        return Ok(());
    }
    with_registered_namespace(facts, body.source, caller, |facts, transport| {
        transport.steps = transport
            .steps
            .saturating_add(body.locals.len())
            .saturating_add(body.projections.len())
            .saturating_add(body.values.len())
            .saturating_add(1);
        let fold = &mut transport.fold;
        body.result = fold.apply(&mut facts.types, body.result)?;
        for local in &mut body.locals {
            local.ty = fold.apply(&mut facts.types, local.ty)?;
        }
        for projection in &mut body.projections {
            projection.ty = fold.apply(&mut facts.types, projection.ty)?;
            if let ProjectionKind::Dereference { region, .. } = &mut projection.kind {
                *region = fold.region(*region, 0)?;
            }
        }
        for value in &mut body.values {
            value.ty = fold.apply(&mut facts.types, value.ty)?;
            operand(facts, &mut value.kind, fold)?;
        }
        Ok(())
    })
}
fn operand(
    facts: &mut Declarations,
    kind: &mut Rvalue,
    fold: &mut Bound,
) -> Result<(), &'static str> {
    match kind {
        Rvalue::AssociatedConstant(instance) => {
            *instance = instance.substitute(&mut facts.types, fold)?;
        }
        Rvalue::Intrinsic { instance, .. } => {
            let ty = fold.apply(&mut facts.types, instance.ty())?;
            *instance = crate::types::instance::Instance::new(&facts.types, ty)?;
        }
        Rvalue::Call { signature, .. } | Rvalue::Invoke { signature, .. } => {
            *signature = fold.signature(&mut facts.types, *signature, 0)?;
        }
        Rvalue::Unsize { conversion, .. } => {
            *conversion = conversion.try_map(|ty| fold.apply(&mut facts.types, ty))?;
        }
        Rvalue::SourceAssociated(_) => {
            return Err("source operand was not recovered before namespace transport");
        }
        _ => {}
    }
    Ok(())
}
