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
pub(super) fn defining(
    facts: &mut Declarations,
    owner: DefId,
    arguments: GenericArgs,
    budget: &Budget,
) -> Result<Conditions, &'static str> {
    let schema = facts
        .associated
        .schema(owner)
        .ok_or("defining implementation schema is missing")?;
    let count = facts
        .types
        .arguments(arguments)
        .ok_or("defining implementation arguments are missing")?
        .len();
    let captures = schema.captures as usize;
    let parameters = facts.generics.parameters.get(&owner).map_or(0, Vec::len);
    budget.charge(
        1 + count + parameters,
        count
            .saturating_mul(4 * size_of::<Argument>())
            .saturating_add(
                parameters.saturating_mul(4 * size_of::<crate::types::parameters::ParameterId>()),
            )
            .saturating_add(schema.bounds.len().saturating_mul(2 * size_of::<Pair>())),
    )?;
    let values: Vec<_> = facts
        .types
        .arguments(arguments)
        .ok_or("defining arguments disappeared")?
        .collect();
    let ids = facts.generics.parameter_ids(&facts.types, owner)?;
    if count != captures.saturating_add(ids.len()) {
        return Err("defining implementation argument arity changed");
    }
    let regions = values[..captures]
        .iter()
        .map(|argument| match argument {
            Argument::Lifetime(region) => Ok(*region),
            _ => Err("defining capture is not a lifetime"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let bindings: Vec<_> = ids
        .iter()
        .copied()
        .zip(values[captures..].iter().copied())
        .collect();
    let mut fold = Counted::new(
        substitute::Bound::parameters_captured_charged(
            &facts.types,
            &bindings,
            regions,
            &|steps, bytes| budget.charge(steps, bytes),
        )?,
        budget,
    )?;
    let mut result = Conditions {
        clauses: Vec::new(),
        outlives: Vec::with_capacity(schema.bounds.len()),
    };
    // Copy only actual schema pairs before mutating the canonical store.
    let pairs = schema.bounds.clone();
    for (longer, shorter) in pairs {
        let (Argument::Lifetime(longer), Argument::Lifetime(shorter)) = (
            fold.apply_argument(&mut facts.types, Argument::Lifetime(longer))?,
            fold.apply_argument(&mut facts.types, Argument::Lifetime(shorter))?,
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
