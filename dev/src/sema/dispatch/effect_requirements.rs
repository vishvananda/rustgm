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
    let count = facts
        .generics
        .declared_predicates_in_scope(definition)
        .count();
    facts.charge_caller_inputs(
        count.saturating_add(1),
        count.saturating_mul(size_of::<Clause>()),
    )?;
    let clauses = facts
        .generics
        .declared_predicates_in_scope(definition)
        .map(|located| located.value)
        .collect::<Vec<_>>();
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
        let proof = dispatch.implementation(facts, predicate);
        facts.charge_caller_inputs(dispatch.context.steps.saturating_sub(before), 0)?;
        let (_, _, proof) =
            proof.map_err(|_| "const default method requirement is not satisfied")?;
        if !proof.is_empty() || dispatch.provisional_size {
            return Err(
                "const default method requirement has unresolved lifetime or size obligations",
            );
        }
    }
    Ok(())
}
