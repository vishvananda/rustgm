//! Complete closure arguments and sparse environment lifetimes share one proof scope.
#[cfg(test)]
#[path = "../../../tests/invariants/interval_cursor.rs"]
mod tests;
use super::{Bound, Declarations, Inputs};
use crate::{
    sema::closures::{Body, Execution},
    types::{Kind, instance::Instance},
};

pub(crate) struct Prepared {
    pub execution: Execution,
    pub proof: Option<crate::types::lifetime_proof::Obligations>,
}

/// Reprove the original output conditions under the selected complete Self.
/// The instance keeps every child constraint and requires field/result formation;
/// none of the source parent's conditional conclusions becomes an assumption.
pub(super) fn formation(contract: &mut crate::body::regions::Contract) {
    let Some(link) = &contract.output else {
        return;
    };
    if let Some(formation) = &link.formation {
        contract
            .obligations
            .extend(
                formation
                    .inputs
                    .iter()
                    .map(|input| crate::body::regions::Obligation {
                        source: contract.source.into(),
                        kind: crate::body::regions::Kind::WellFormed(input.child),
                    }),
            );
    }
    contract.require_result_formation = true;
    contract.output = None;
}

/// Signature connections are identities. Captured execution alone can inherit
/// conditions from the checked source seal, which must actually be present.
pub(super) fn conditions(
    facts: &Declarations,
    key: crate::body::ir::BodyKey,
) -> Result<&[crate::types::outlives::Pair], &'static str> {
    let crate::body::ir::BodyKey::Closure(identity) = key else {
        return Err("captured instance has a different source owner");
    };
    let source = facts
        .closures
        .identity(identity)
        .ok_or("instance input connections have no closure source")?;
    if source.execution.is_none() {
        return Ok(&[]);
    }
    facts
        .closures
        .conditions(identity)
        .ok_or("captured instance requires checked source execution conditions")
}

pub(super) fn arguments(
    facts: &Declarations,
    instance: Instance,
    body: Body,
) -> Result<Inputs, &'static str> {
    let actual = crate::sema::closures::actual::arguments(facts, instance, body)?;
    Ok(Inputs {
        caller: crate::sema::values::execution::CallerInstance::Callable(instance),
        instance: Some(instance),
        bindings: actual.bindings,
        captures: actual.captures,
        comparison: None,
        connections: None,
    })
}

/// Private signature ports are connected by their selected typed roots, not by
/// counting them as late call binders. The inherited prefix remains exact.
pub(super) fn private_inputs(
    facts: &mut Declarations,
    instance: Instance,
    source: Body,
    actual: crate::types::TypeId,
    inputs: &mut Inputs,
) -> Result<(), &'static str> {
    use crate::types::RegionId;
    use crate::types::lifetime_proof::{Obligations, ScopeId};
    let public = crate::sema::closures::actual::call_binders(facts, source)?;
    let late_end = source
        .inherited
        .checked_add(public)
        .ok_or("closure call binder overflow")?;
    let (mut formal, universals) =
        crate::sema::closures::actual::signature(facts, instance, source)?;
    let mut proof = Obligations::new(1);
    if facts.types.contains_projections(formal) == Some(true) {
        let mut session = crate::sema::projections::concrete(&mut proof, ScopeId::ROOT)?;
        formal = normalize_signature(facts, &mut session, formal)?;
    }
    let (bindings, steps) = crate::sema::closures::matching::signature_bindings(
        &facts.types,
        formal,
        universals,
        actual,
        public,
    )?;
    facts.body_work.canonical_steps += steps + inputs.captures.len();
    for (index, &capture) in inputs.captures.iter().enumerate() {
        let index = u16::try_from(index).map_err(|_| "too many inherited closure lifetimes")?;
        if let Some(&slot) = bindings.slots.get(&index)
            && caller_region(slot)? != capture
        {
            return Err("closure signature differs from its inherited arguments");
        }
    }
    let mut slots = Vec::with_capacity(bindings.slots.len());
    for (slot, region) in bindings.slots {
        if (source.inherited..late_end).contains(&u32::from(slot)) {
            let late = u16::try_from(u32::from(slot) - source.inherited)
                .map_err(|_| "too many closure call lifetimes")?;
            if region != RegionId::bound(0, late)? {
                return Err("closure public lifetime differs from its source binder");
            }
        } else {
            slots.push((slot, caller_region(region)?));
        }
    }
    facts.body_work.canonical_steps += slots.len();
    slots.sort_unstable_by_key(|&(slot, _)| slot);
    inputs.connections = Some(crate::body::invocation_entries::SourceBindings {
        original: source.universals,
        slots,
        equalities: bindings
            .equalities
            .into_iter()
            .map(|(a, b)| Ok((caller_region(a)?, caller_region(b)?)))
            .collect::<Result<_, &'static str>>()?,
        proof: (!proof.is_empty()).then_some(proof),
    });
    Ok(())
}

/// Actual signature roots retain their public binder. Private ports and
/// inherited arguments must come from outside it, including beneath nesting.
fn caller_region(region: crate::types::RegionId) -> Result<crate::types::RegionId, &'static str> {
    let Some((depth, slot)) = region.bound_parts() else {
        return Ok(region);
    };
    crate::types::RegionId::bound(
        depth
            .checked_sub(1)
            .ok_or("private closure port captures a public input lifetime")?,
        slot,
    )
}

/// Parameter argument lifetimes are fresh slots after the entire source binder.
/// Matching the complete Self reconnects them without capturing a source local.
pub(crate) fn execution(
    facts: &mut Declarations,
    instance: Instance,
    execution: Execution,
) -> Result<Prepared, &'static str> {
    let execution = substitute(facts, instance, execution)?;
    normalize(facts, execution)
}

/// Keep every original slot, including a receiver removed by an owned adapter.
pub(crate) fn substitute(
    facts: &mut Declarations,
    instance: Instance,
    execution: Execution,
) -> Result<Execution, &'static str> {
    crate::sema::closures::actual::execution(facts, instance, execution)
}

/// Normalize Self before shaping a requested receiver. Its separate debts must
/// survive even when the requested signature omits the source receiver slot.
pub(crate) fn self_type(
    facts: &mut Declarations,
    mut execution: Execution,
) -> Result<Prepared, &'static str> {
    use crate::types::lifetime_proof::{Obligations, ScopeId};
    if facts.types.contains_projections(execution.self_ty) != Some(true) {
        return Ok(Prepared {
            execution,
            proof: None,
        });
    }
    let mut proof = Obligations::new(1);
    let mut session = crate::sema::projections::concrete(&mut proof, ScopeId::ROOT)?;
    let (self_ty, steps) = session.normalize(facts, execution.self_ty)?;
    facts.body_work.canonical_steps += steps;
    execution.self_ty = self_ty;
    Ok(Prepared {
        execution,
        proof: (!proof.is_empty()).then_some(proof),
    })
}

/// Canonical normal forms retain their own debts in the formal execution scope.
pub(crate) fn normalize(
    facts: &mut Declarations,
    mut execution: Execution,
) -> Result<Prepared, &'static str> {
    use crate::types::lifetime_proof::{Obligations, ScopeId};
    if facts.types.contains_projections(execution.self_ty) != Some(true)
        && facts.types.contains_projections(execution.signature) != Some(true)
    {
        return Ok(Prepared {
            execution,
            proof: None,
        });
    }
    let mut proof = Obligations::new(1);
    let mut session = crate::sema::projections::concrete(&mut proof, ScopeId::ROOT)?;
    let (self_ty, steps) = session.normalize(facts, execution.self_ty)?;
    execution.self_ty = self_ty;
    facts.body_work.canonical_steps += steps;
    execution.signature = normalize_signature(facts, &mut session, execution.signature)?;
    Ok(Prepared {
        execution,
        proof: (!proof.is_empty()).then_some(proof),
    })
}

/// These roots live inside an existing proof binder, unlike a nested callable.
fn normalize_signature(
    facts: &mut Declarations,
    session: &mut crate::sema::projections::Concrete<'_>,
    signature: crate::types::TypeId,
) -> Result<crate::types::TypeId, &'static str> {
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions,
        unsafe_,
        convention,
    }) = facts.types.kind(signature)
    else {
        return Err("closure execution lacks its proof signature");
    };
    // This signature owns the existing execution universals. Normalize its
    // roots at ROOT, preserving nested callable binders inside those roots.
    let parameters = facts
        .types
        .list(parameters)
        .ok_or("invalid execution inputs")?
        .to_vec();
    let mut normalized = Vec::with_capacity(parameters.len());
    for parameter in parameters {
        let (parameter, steps) = session.normalize(facts, parameter)?;
        normalized.push(parameter);
        facts.body_work.canonical_steps += steps;
    }
    let (result, steps) = session.normalize(facts, result)?;
    facts.body_work.canonical_steps += steps;
    let parameters = facts.types.intern_list(&normalized)?;
    facts.types.intern(Kind::Function {
        parameters,
        result,
        bound_regions,
        unsafe_,
        convention,
    })
}

/// Full structural matching supplies identities, never proof of normalization.
/// Rebase private witnesses before introducing actual caller lifetimes, then
/// retain every selected debt for the ordinary body checker.
pub(crate) fn retain_proof(
    facts: &mut Declarations,
    contract: &mut crate::body::regions::Contract,
    proof: crate::types::lifetime_proof::Obligations,
    slots: impl Iterator<Item = (u16, crate::types::RegionId)>,
) -> Result<(), &'static str> {
    ProofCursor::default().retain(facts, contract, proof, slots)
}

/// One entry-selection pass appends debts and owns every new private interval.
/// Initialize lazily so projection-free calls never scan the body contract.
/// A cursor must not span another producer of contract variables or proofs.
#[derive(Default)]
pub(crate) struct ProofCursor {
    next: Option<u32>,
}
impl ProofCursor {
    pub(crate) fn first(
        &mut self,
        facts: &mut Declarations,
        contract: &crate::body::regions::Contract,
    ) -> Result<u32, &'static str> {
        if let Some(next) = self.next {
            return Ok(next);
        }
        facts.body_work.canonical_steps += contract.obligations.len() + contract.proofs.len();
        let first = super::projections::first_region(contract)?;
        self.next = Some(first);
        Ok(first)
    }

    /// Publish already allocated caller evidence without rebasing actual inputs.
    pub(crate) fn append(
        &mut self,
        facts: &mut Declarations,
        contract: &mut crate::body::regions::Contract,
        proof: crate::types::lifetime_proof::Obligations,
    ) -> Result<(), &'static str> {
        if proof.regions.start != self.first(facts, contract)?
            || proof.regions.end < proof.regions.start
        {
            return Err("closure entry proof allocation differs from its caller cursor");
        }
        self.next = Some(proof.regions.end);
        contract
            .proofs
            .push(crate::body::regions::Proof::new(contract.source, proof));
        Ok(())
    }

    pub(crate) fn retain(
        &mut self,
        facts: &mut Declarations,
        contract: &mut crate::body::regions::Contract,
        mut proof: crate::types::lifetime_proof::Obligations,
        slots: impl Iterator<Item = (u16, crate::types::RegionId)>,
    ) -> Result<(), &'static str> {
        let first = self.first(facts, contract)?;
        let work = proof.rebase(&mut facts.types, first)?;
        let mapping = slots
            .map(|(slot, actual)| Ok((crate::types::RegionId::bound(0, slot)?, actual)))
            .collect::<Result<crate::support::hash::HashMap<_, _>, &'static str>>()?;
        let mut fold = Bound::restore_owned_regions(mapping, proof.regions.clone());
        proof.substitute(&mut facts.types, &mut fold)?;
        facts.body_work.canonical_steps += work + fold.steps;
        self.append(facts, contract, proof)
    }
}
