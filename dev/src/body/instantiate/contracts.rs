//! Specialize typed obligations while separating caller and callee lifetime scopes.
use super::{Bound, Declarations, Inputs};
use crate::{
    body::regions::{Contract, Kind},
    types::{arguments::Argument, outlives::Queries},
};

pub(super) fn specialize(
    facts: &mut Declarations,
    original: &Contract,
    inputs: &mut Inputs,
) -> Result<(Bound, Contract), &'static str> {
    let mut regions = Bound::universals(original.universals);
    for (_, argument) in &mut inputs.bindings {
        *argument = regions.apply_argument(&mut facts.types, *argument)?;
    }
    if let Some((expected, actual)) = &mut inputs.comparison {
        *expected = regions.apply(&mut facts.types, *expected)?;
        *actual = regions.apply(&mut facts.types, *actual)?;
    }
    let mut bound = Bound::parameters(&facts.types, &inputs.bindings)?;
    let mut contract = original.clone();
    contract.universals = original
        .universals
        .checked_add(
            u32::try_from(regions.universal_count()).map_err(|_| "too many instance lifetimes")?,
        )
        .ok_or("too many instance lifetimes")?;
    contract.parameters.clear();
    if let Some(signature) = &mut contract.signature {
        // Body signatures express their parameter roots inside the body binder.
        // Ordinary callable types visited below retain their nested binder.
        *signature = bound.signature(&mut facts.types, *signature, contract.universals)?;
    }
    for (ty, _) in &mut contract.type_assumptions {
        *ty = bound.apply(&mut facts.types, *ty)?;
    }
    for obligation in &mut contract.obligations {
        match &mut obligation.kind {
            Kind::Coercion {
                actual, expected, ..
            } => {
                *actual = bound.apply(&mut facts.types, *actual)?;
                *expected = bound.apply(&mut facts.types, *expected)?;
            }
            Kind::WellFormed(ty) | Kind::TypeOutlives { ty, .. } => {
                *ty = bound.apply(&mut facts.types, *ty)?;
            }
            _ => {}
        }
    }
    for (_, proof) in &mut contract.proofs {
        proof.substitute(&mut facts.types, &mut bound)?;
    }
    // Instantiation sites have checked argument well-formedness. Retain those
    // structural relations as assumptions in this separate proof universe,
    // including arguments absent from the signature's runtime parameters.
    for &(_, argument) in &inputs.bindings {
        if let Argument::Type(ty) = argument {
            facts
                .lifetimes
                .nominal_requirements
                .prepare(&mut facts.types, ty)?;
        }
    }
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let mut projections = Vec::new();
    for &(_, argument) in &inputs.bindings {
        if let Argument::Type(ty) = argument {
            let requirements = queries.symbolic_requirements(ty)?;
            if !requirements.scalars.parameters.is_empty() {
                return Err("instance contract retains generic argument requirements");
            }
            contract
                .assumptions
                .extend_from_slice(requirements.scalars.regions);
            projections.extend_from_slice(requirements.projections);
        }
    }
    let mut folds = std::collections::HashMap::new();
    for (projection, shorter) in projections {
        let ty = if projection.depth == 0 {
            projection.ty
        } else {
            folds
                .entry(projection.depth)
                .or_insert_with(|| Bound::lower(projection.depth))
                .apply(&mut facts.types, projection.ty)?
        };
        contract.type_assumptions.push((ty, shorter));
    }
    Ok((bound, contract))
}
