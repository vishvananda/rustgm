//! Method-specific conditional bounds are distinct from the selected Self impl.
use super::{Declarations, Dispatcher};
use crate::{
    sema::compiler_items::const_payments::{closed_clause, open_closed_clause},
    support::constness::Constness,
    types::{
        Kind,
        instance::Instance,
        parameters::{clauses::Clause, predicates::Bound},
    },
};

pub(super) fn check(
    dispatch: &mut Dispatcher,
    facts: &mut Declarations,
    instance: Instance,
) -> Result<(), &'static str> {
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
        return Err("const default requirements lost their function item");
    };
    let definition = crate::resolve::model::DefId(item.0);
    let clauses = declared_inputs(facts, definition)?;
    let mut fold = super::callable_substitution(facts, instance)?;
    for clause in clauses {
        facts.charge_caller_inputs(4, 0)?;
        if matches!(clause, Clause::UnstableFeature(_)) {
            continue;
        }
        let predicate = closed_clause(&facts.types, clause)?;
        if !matches!(predicate.bound, Bound::MaybeConstTrait(_)) {
            continue;
        }
        let clause =
            crate::sema::traits::effects::in_context(&mut facts.types, clause, Constness::Const)?;
        let before = fold.steps;
        let clause = fold.clause(&mut facts.types, clause);
        facts.charge_caller_inputs(fold.steps.saturating_sub(before), 0)?;
        let predicate = open_closed_clause(facts, clause?)?;
        if !predicate.is_concrete(&facts.types)? || predicate.contains_projections(&facts.types)? {
            return Err("const default method requirements need closed normalized predicates");
        }
        let before = dispatch.context.steps;
        let proof = if crate::sema::compiler_items::destruct_predicate(facts, predicate)? {
            // This exact source role has builtin bound evidence, rather than
            // an implementation owner. Actual Drop/field debts stay in its proof.
            dispatch
                .source_evidence(facts, predicate, None)
                .map(|evidence| evidence.proof)
                .map_err(super::Failure::message)
        } else {
            // Preserve ordinary closed-implementation certification and reuse.
            dispatch
                .implementation(facts, predicate)
                .map(|(_, _, proof)| proof)
        };
        facts.charge_caller_inputs(dispatch.context.steps.saturating_sub(before), 0)?;
        let proof = proof.map_err(|_| "const default method requirement is not satisfied")?;
        if !proof.is_empty() || dispatch.provisional_size {
            return Err(
                "const default method requirement has unresolved lifetime or size obligations",
            );
        }
    }
    Ok(())
}

fn declared_inputs(
    facts: &Declarations,
    definition: crate::resolve::model::DefId,
) -> Result<Vec<Clause>, &'static str> {
    if facts.imported_const_default(definition)? {
        // Imported provided methods carry their own predicates in their
        // authenticated header, including Clone's conditional Destruct premise.
        let inputs = crate::sema::declaration_inputs::declared_own(
            &facts.generics,
            facts.crates.as_deref(),
            definition,
        )?;
        reserve(facts, inputs.len())?;
        return Ok(inputs.iter().map(|input| input.value).collect());
    }
    let inputs = facts.generics.declared_predicates_in_scope(definition);
    reserve(facts, inputs.count())?;
    Ok(facts
        .generics
        .declared_predicates_in_scope(definition)
        .map(|input| input.value)
        .collect())
}

fn reserve(facts: &Declarations, count: usize) -> Result<(), &'static str> {
    facts.charge_caller_inputs(
        count.saturating_add(1),
        count.saturating_mul(size_of::<Clause>()),
    )
}
