//! Complete closure arguments and sparse environment lifetimes share one proof scope.
use super::{Bound, Declarations, Inputs};
use crate::{
    sema::closures::{Body, Execution},
    types::{ItemId, Kind, arguments::Argument, instance::Instance},
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
                        source: contract.source,
                        kind: crate::body::regions::Kind::WellFormed(input.child),
                    }),
            );
    }
    contract.require_result_formation = true;
    contract.output = None;
}

pub(super) fn arguments(
    facts: &Declarations,
    instance: Instance,
    body: Body,
) -> Result<Inputs, &'static str> {
    let Some(Kind::Closure { arguments, .. }) = facts.types.kind(instance.ty()) else {
        return Err("closure arguments require a canonical closure instance");
    };
    let owner = body.owner.ok_or("generic closure has no enclosing owner")?;
    let parameters = match facts
        .lifetimes
        .callables
        .generic_parameters(ItemId(owner.0))
    {
        Some(parameters) => parameters,
        None if !facts.generics.has_parameters(owner) => &[],
        None => return Err("missing enclosing closure parameters"),
    };
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("invalid closure arguments")?
        .collect::<Vec<_>>();
    let inherited = body.inherited as usize;
    if arguments.len() != inherited.saturating_add(parameters.len()) {
        return Err("closure instance argument count differs from its enclosing template");
    }
    let captures = arguments[..inherited]
        .iter()
        .map(|argument| match argument {
            Argument::Lifetime(region) => Ok(*region),
            _ => Err("closure inherited argument is not a lifetime"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Inputs {
        instance: Some(instance),
        bindings: parameters
            .iter()
            .copied()
            .zip(arguments[inherited..].iter().copied())
            .collect(),
        captures,
        comparison: None,
        connections: None,
    })
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
    mut execution: Execution,
) -> Result<Execution, &'static str> {
    let Some(Kind::Closure { identity, .. }) = facts.types.kind(instance.ty()) else {
        return Err("closure execution requires a canonical instance");
    };
    let source = *facts
        .closures
        .identity(identity)
        .ok_or("missing closure source")?;
    let mut inputs = arguments(facts, instance, source)?;
    if inputs.bindings.is_empty() {
        return Ok(execution);
    }
    let mut regions = Bound::universals(execution.universals);
    for (_, argument) in &mut inputs.bindings {
        *argument = regions.apply_argument(&mut facts.types, *argument)?;
    }
    let count = u32::try_from(regions.universal_count())
        .map_err(|_| "too many closure argument lifetimes")?;
    execution.universals = execution
        .universals
        .checked_add(count)
        .ok_or("closure lifetime overflow")?;
    let mut parameters = Bound::parameters(&facts.types, &inputs.bindings)?;
    execution.self_ty = parameters.apply(&mut facts.types, execution.self_ty)?;
    execution.signature =
        parameters.signature(&mut facts.types, execution.signature, execution.universals)?;
    facts.body_work.canonical_steps += regions.steps + parameters.steps;
    Ok(execution)
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
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions,
        unsafe_,
        convention,
    }) = facts.types.kind(execution.signature)
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
    execution.signature = facts.types.intern(Kind::Function {
        parameters,
        result,
        bound_regions,
        unsafe_,
        convention,
    })?;
    Ok(Prepared {
        execution,
        proof: (!proof.is_empty()).then_some(proof),
    })
}

/// Full structural matching supplies identities, never proof of normalization.
/// Rebase private witnesses before introducing actual caller lifetimes, then
/// retain every selected debt for the ordinary body checker.
pub(crate) fn retain_proof(
    facts: &mut Declarations,
    contract: &mut crate::body::regions::Contract,
    mut proof: crate::types::lifetime_proof::Obligations,
    slots: impl Iterator<Item = (u16, crate::types::RegionId)>,
) -> Result<(), &'static str> {
    let first = super::projections::first_region(contract)?;
    let work = proof.rebase(&mut facts.types, first)?;
    let mapping = slots
        .map(|(slot, actual)| Ok((crate::types::RegionId::bound(0, slot)?, actual)))
        .collect::<Result<std::collections::HashMap<_, _>, &'static str>>()?;
    let mut fold = Bound::restore_owned_regions(mapping, proof.regions.clone());
    proof.substitute(&mut facts.types, &mut fold)?;
    facts.body_work.canonical_steps += work + fold.steps;
    contract
        .proofs
        .push(crate::body::regions::Proof::new(contract.source, proof));
    Ok(())
}
