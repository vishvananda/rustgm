//! Closed selections discharge their entire lifetime proof without caller assumptions.
use super::{Declarations, Obligations, selection};
use crate::{
    resolve::model::DefId,
    sema::traits::implementations::Availability,
    types::{
        GenericArgs,
        infer::{Context, Instantiation},
        outlives::Queries,
        parameters::predicates::Predicate,
        relations::Constraints,
    },
};

#[derive(Clone, Copy)]
pub(super) struct Target {
    pub owner: DefId,
    pub arguments: GenericArgs,
}

pub(super) fn select(
    facts: &mut Declarations,
    context: &mut Context,
    predicate: Predicate,
    caller: selection::Caller<'_>,
    availability: Availability,
) -> Result<(Target, Obligations, bool), String> {
    let evidence = select_source(facts, context, predicate, caller, availability)?;
    let selection::Source::Implementation { owner, arguments } = evidence.source else {
        return Err("concrete dispatch cannot use a symbolic assumption".into());
    };
    Ok((
        Target { owner, arguments },
        evidence.proof,
        evidence.provisional_size,
    ))
}

pub(super) fn select_source(
    facts: &mut Declarations,
    context: &mut Context,
    predicate: Predicate,
    caller: selection::Caller<'_>,
    availability: Availability,
) -> Result<selection::Evidence, String> {
    let before = context.region_equalities.len();
    let mut substitution =
        Instantiation::new(context, &facts.types, &[]).map_err(|e| e.to_string())?;
    let goal = context
        .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
        .map_err(|e| e.to_string())?;
    facts.dispatch_dependencies.work.selections += 1;
    let selected = selection::select_available(facts, context, &goal, caller, availability)?;
    let selection::Selection::Proven(mut evidence) = selected else {
        return Err("trait callable requires an unambiguous concrete implementation".into());
    };
    evidence
        .proof
        .equalities
        .extend(context.region_equalities.drain(before..));
    Ok(evidence)
}

/// Return a reusable target only when its complete arguments can escape the
/// private proof and every obligation holds in an empty lifetime environment.
/// An unanchored private argument retains the original caller-owned proof.
pub(super) fn certify(
    facts: &mut Declarations,
    context: &mut Context,
    target: Target,
    proof: &Obligations,
) -> Result<Option<Target>, String> {
    let mut arguments = target.arguments;
    if !proof.is_empty() {
        let mut restore = proof.body_restoration()?;
        let mut values = facts
            .types
            .arguments(arguments)
            .ok_or("invalid closed implementation arguments")?
            .collect::<Vec<_>>();
        for argument in &mut values {
            *argument = restore.apply_argument(&mut facts.types, *argument)?;
        }
        arguments = facts.types.intern_arguments(&values)?;
        context
            .account_work(restore.steps + values.len())
            .map_err(|e| e.to_string())?;
    }
    if !facts.types.arguments_are_concrete(arguments)? {
        return Ok(None);
    }
    if !proof.is_empty() {
        let mut constraints =
            Constraints::new(&facts.types, &facts.lifetimes.variances, 0, 1_000_000)
                .map_err(|e| e.to_string())?;
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        // Requiring the retained type-outlives clauses decomposes their actual
        // components; it does not assume unfinished nominal formation contracts.
        proof.require(&mut constraints, &mut queries)?;
        constraints
            .solve()
            .map_err(|e| format!("closed dispatch lifetime requirement is not satisfied: {e}"))?;
    }
    facts.dispatch_dependencies.work.closed_proofs += 1;
    Ok(Some(Target {
        arguments,
        ..target
    }))
}
