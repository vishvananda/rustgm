//! Specialize typed obligations while separating caller and callee lifetime scopes.
use super::{Binding, Bound, Declarations};
use crate::{
    body::regions::{Contract, Kind},
    types::{arguments::Argument, outlives::Queries},
};

pub(super) fn specialize(
    facts: &mut Declarations,
    original: &Contract,
    bindings: &mut [Binding],
) -> Result<(Bound, Contract), &'static str> {
    let mut regions = Bound::universals(original.universals);
    for (_, argument) in bindings.iter_mut() {
        *argument = regions.apply_argument(&mut facts.types, *argument)?;
    }
    let mut bound = Bound::parameters(&facts.types, bindings)?;
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
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    for &(_, argument) in bindings.iter() {
        if let Argument::Type(ty) = argument {
            let requirements = queries.symbolic_requirements(ty)?;
            if !requirements.parameters.is_empty() {
                return Err("instance contract retains generic argument requirements");
            }
            contract.assumptions.extend_from_slice(requirements.regions);
        }
    }
    Ok((bound, contract))
}
