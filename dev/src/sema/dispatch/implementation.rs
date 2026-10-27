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

pub(super) fn select_using(
    facts: &mut Declarations,
    context: &mut Context,
    predicate: Predicate,
    caller: selection::Caller<'_>,
    availability: Availability,
    preparation: Option<&mut dyn super::Preparation>,
) -> Result<(Target, Obligations, bool), super::Failure> {
    let evidence =
        select_source_using(facts, context, predicate, caller, availability, preparation)?;
    let selection::Source::Implementation { owner, arguments } = evidence.source else {
        return Err("concrete dispatch cannot use a symbolic assumption".into());
    };
    Ok((
        Target { owner, arguments },
        evidence.proof,
        evidence.provisional_size,
    ))
}

pub(super) fn select_source_using(
    facts: &mut Declarations,
    context: &mut Context,
    predicate: Predicate,
    caller: selection::Caller<'_>,
    availability: Availability,
    preparation: Option<&mut dyn super::Preparation>,
) -> Result<selection::Evidence, super::Failure> {
    let before = context.region_equalities.len();
    let mut substitution =
        Instantiation::new(context, &facts.types, &[]).map_err(|e| e.to_string())?;
    let goal = context
        .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
        .map_err(|e| e.to_string())?;
    facts.dispatch_dependencies.work.selections += 1;
    let selected = if let Some(preparation) = preparation {
        selection::select_prepared(facts, context, &goal, &caller, preparation)?
    } else {
        selection::select_available(facts, context, &goal, caller, availability)?
    };
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

/// Compose only this selection's aliases before the caller closes source roots.
/// The original evidence remains mandatory caller debt; this grants no permission.
pub(super) fn restore_constant(
    facts: &mut Declarations,
    context: &mut Context,
    selected: crate::sema::values::Instance,
    proof: &Obligations,
    caller: &Obligations,
) -> Result<crate::sema::values::Instance, String> {
    if proof.equalities.is_empty() {
        // No aliases can change. Caller input lifetimes occurring only in
        // type-outlives debts remain untouched until ordinary body proof.
        return Ok(selected);
    }
    let mut steps = 0;
    let result = (|| {
        let nested = facts
            .types
            .arguments(selected.arguments)
            .ok_or("missing selected argument graph")?
            .any(|argument| {
                matches!(argument, crate::types::arguments::Argument::Type(ty)
                if facts.types.needs_region_context(ty) == Some(true))
            });
        let mut restore = if nested {
            crate::sema::values::execution::SourceComputation::with_argument_regions(
                facts,
                selected,
                |_, regions| {
                    proof.body_restoration_with_inputs_counted(caller, regions, &mut steps)
                },
            )?
        } else {
            proof.body_restoration_counted(&mut steps)?
        };
        let result = selected.substitute(&mut facts.types, &mut restore);
        steps += restore.steps;
        result
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(steps);
    context.account_work(steps).map_err(|e| e.to_string())?;
    result.map_err(Into::into)
}
