//! Method-specific conditional bounds are distinct from the selected Self impl.
use super::{Declarations, Dispatcher};
use crate::{
    sema::compiler_items::const_payments::{closed_clause, open_closed_clause},
    support::{call_traits::CallTrait, constness::Constness},
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
    let callable = facts
        .imported_iterator_fold(definition)?
        .then(|| crate::sema::compiler_items::callable_role(facts, CallTrait::FnMut))
        .transpose()?;
    let iterator_reduction = callable.is_some() || facts.imported_iterator_last(definition)?;
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
        let callback = callable.is_some_and(|role| {
            matches!(predicate.bound, Bound::ConstTrait(application)
                if facts.types.trait_application(application).is_some_and(|app| app.item.0 == role.0))
        });
        let closed = predicate.is_concrete(&facts.types)?
            && !predicate.contains_projections(&facts.types)?;
        let destruct = (closed
            && crate::sema::compiler_items::destruct_predicate(facts, predicate)?)
            || (iterator_reduction
                && facts.types.contains_parameters(predicate.ty) == Some(false)
                && facts.types.needs_region_context(predicate.ty) == Some(false)
                && crate::sema::compiler_items::destruct_role(facts, predicate)?);
        if !(closed || callback || iterator_reduction && destruct) {
            return Err("const default method requirements need closed normalized predicates");
        }
        let before = dispatch.context.steps;
        let proof = if callback || destruct {
            // The ordinary source selection normalizes the closed Iterator
            // projection before checking the actual callback. Builtin Destruct
            // keeps its own Drop and field debts in the same proof.
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
