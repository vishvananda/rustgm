//! The actual defining impl's conditions are re-proved; its Self:Trait is never a premise.
use super::retention_work::{Budget, Counted};
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    support::retained::Growth,
    types::{
        GenericArgs, Kind,
        arguments::Argument,
        outlives::Pair,
        parameters::{
            ParameterKind,
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
        substitute,
    },
};
pub(in crate::sema) struct Conditions {
    pub clauses: Vec<Clause>,
    pub outlives: Vec<Pair>,
}
pub(super) struct Mapping {
    pub fold: Counted<substitute::Bound>,
    parameters: Vec<crate::types::parameters::ParameterId>,
}
pub(super) fn mapping(
    facts: &mut Declarations,
    owner: DefId,
    arguments: GenericArgs,
    budget: &Budget,
) -> Result<Mapping, &'static str> {
    let (fold, parameters) =
        facts.selected_implementation_substitution(owner, arguments, &|steps, bytes| {
            budget.charge(steps, bytes)
        })?;
    let fold = Counted::new(fold, budget)?;
    Ok(Mapping { fold, parameters })
}
pub(super) fn defining(
    facts: &mut Declarations,
    owner: DefId,
    arguments: GenericArgs,
    budget: &Budget,
) -> Result<Conditions, &'static str> {
    let Mapping {
        mut fold,
        parameters: ids,
    } = mapping(facts, owner, arguments, budget)?;
    let schema = facts
        .associated
        .schema(owner)
        .ok_or("defining implementation schema is missing")?;
    budget.charge(
        schema.bounds.len(),
        schema.bounds.len().saturating_mul(2 * size_of::<Pair>()),
    )?;
    let mut result = Conditions {
        clauses: Vec::new(),
        outlives: Vec::with_capacity(schema.bounds.len()),
    };
    // Copy only actual schema pairs before mutating the canonical store.
    let pairs = schema.bounds.clone();
    for (longer, shorter) in pairs {
        let (Argument::Lifetime(longer), Argument::Lifetime(shorter)) = (
            fold.apply_argument_charged(
                &mut facts.types,
                Argument::Lifetime(longer),
                &|steps, bytes| budget.charge(steps, bytes),
            )?,
            fold.apply_argument_charged(
                &mut facts.types,
                Argument::Lifetime(shorter),
                &|steps, bytes| budget.charge(steps, bytes),
            )?,
        ) else {
            return Err("defining region requirement changed kind");
        };
        result.outlives.push((longer, shorter));
    }
    for &id in &ids {
        if matches!(
            facts.types.parameter(id).map(|value| value.kind),
            Some(ParameterKind::Type { sized: true })
        ) {
            let ty = facts.types.intern(Kind::Parameter(id))?;
            push(
                &mut result.clauses,
                Predicate {
                    ty,
                    bound: Bound::Sized,
                }
                .into(),
                budget,
            )?;
        }
    }
    // Keep source clauses in their declaration order, including quantifiers.
    // This is the one producer consumed by Min admission, dispatch and RPIT.
    for located in
        crate::sema::declaration_inputs::declared(&facts.generics, facts.crates.as_deref(), owner)?
    {
        push(&mut result.clauses, located.value, budget)?;
    }
    for clause in &mut result.clauses {
        *clause = fold.clause_charged(&mut facts.types, *clause, &|steps, bytes| {
            budget.charge(steps, bytes)
        })?;
    }
    fold.pay()?;
    Ok(result)
}
fn push(clauses: &mut Vec<Clause>, clause: Clause, budget: &Budget) -> Result<(), &'static str> {
    let growth = Growth::vector::<Clause>(clauses.len(), clauses.capacity(), 1);
    budget.charge(1 + growth.work, growth.bytes)?;
    clauses.push(clause);
    Ok(())
}
