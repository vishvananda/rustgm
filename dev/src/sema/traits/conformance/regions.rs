//! Trait late lifetimes are universal; an impl may choose compatible call-time instantiations.
use super::{Contract, Declarations};
use crate::types::{
    Kind,
    outlives::{Pair, Queries},
    parameters::predicates::Bound,
    relations::Constraints,
};

pub(super) fn prove(
    facts: &Declarations,
    contract: &Contract,
    equalities: &[Pair],
) -> Result<(), String> {
    let mut constraints = Constraints::new(
        &facts.types,
        &facts.lifetimes.variances,
        contract.universals,
        1_000_000,
    )
    .map_err(|e| e.to_string())?;
    for &id in &contract.parameters {
        constraints.parameter(id).map_err(|e| e.to_string())?;
    }
    for &id in &contract.variables {
        constraints.variable(id).map_err(|e| e.to_string())?;
    }
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    for &(a, b) in &contract.assumed_regions {
        constraints.assume(a, b).map_err(|e| e.to_string())?;
    }
    for predicate in &contract.assumptions {
        if let Bound::Outlives(shorter) = predicate.bound {
            constraints.assume_type_outlives(&mut queries, predicate.ty, shorter)?;
        }
    }
    constraints.assume_signature(&mut queries, contract.expected)?;
    for &ty in &contract.inputs {
        constraints.assume_well_formed(&mut queries, ty)?;
    }
    for &(a, b) in &contract.required_regions {
        constraints.outlives(a, b, 0).map_err(|e| e.to_string())?;
    }
    for &(a, b) in equalities {
        constraints.outlives(a, b, 0).map_err(|e| e.to_string())?;
        constraints.outlives(b, a, 0).map_err(|e| e.to_string())?;
    }
    for predicate in &contract.obligations {
        if let Bound::Outlives(shorter) = predicate.bound {
            let parts = queries.components(predicate.ty)?;
            for &longer in parts.regions {
                constraints
                    .outlives(longer, shorter, 0)
                    .map_err(|e| e.to_string())?;
            }
            for &id in parts.parameters {
                constraints
                    .parameter_outlives(id, shorter, 0)
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    let (
        Some(Kind::Function {
            parameters: a,
            result: ar,
            unsafe_: au,
            ..
        }),
        Some(Kind::Function {
            parameters: b,
            result: br,
            unsafe_: bu,
            ..
        }),
    ) = (
        facts.types.kind(contract.actual),
        facts.types.kind(contract.expected),
    )
    else {
        return Err("missing method signatures".into());
    };
    let a = facts
        .types
        .list(a)
        .ok_or("missing impl method parameters")?;
    let b = facts
        .types
        .list(b)
        .ok_or("missing trait method parameters")?;
    if au != bu || a.len() != b.len() {
        return Err("implementation method safety or parameter arity disagrees with trait".into());
    }
    for (&a, &b) in a.iter().zip(b) {
        constraints
            .coerce(b, a, false, 0)
            .map_err(|e| e.to_string())?;
    }
    constraints
        .coerce(ar, br, false, 0)
        .map_err(|e| e.to_string())?;
    constraints
        .solve()
        .map(|_| ())
        .map_err(|e| format!("implementation method lifetime contract does not satisfy trait: {e}"))
}
