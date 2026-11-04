//! Pinned declaration signatures consume canonical types and actual owned formals once.
use super::prototype_matcher::Matcher;
use crate::{
    sema::model::{Declarations, Function},
    support::bindings::{IntrinsicDeclaration, prototype_specs::Profile},
    types::{
        Kind, TypeId,
        arguments::{Argument, ConstKind},
        callable::Convention,
        parameters::ParameterKind,
    },
};

pub(super) fn validate(
    facts: &Declarations,
    function: Function,
    identity: IntrinsicDeclaration,
) -> Result<(), &'static str> {
    let profile = identity.profile();
    // Pay even a failed check before publishing the already reserved signature.
    crate::sema::header_attributes::charge_source_contract_query(facts, usize::from(profile.work))
        .map_err(|_| "source intrinsic signature work budget exceeded")?;
    let formals = formals(facts, function, profile)?;
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("source intrinsic requires its exact Rust function signature");
    };
    if unsafe_ != profile.unsafe_ {
        return Err("source intrinsic safety differs from its pinned declaration");
    }
    let inputs = facts
        .types
        .list(parameters)
        .ok_or("missing intrinsic inputs")?;
    let mut matcher = Matcher::new(facts, formals, bound_regions, profile.work);
    matcher.list(profile.inputs, inputs)?;
    matcher.ty(profile.output, result)
}

fn formals(
    facts: &Declarations,
    function: Function,
    profile: &Profile,
) -> Result<[TypeId; 4], &'static str> {
    let parameters = facts
        .generics
        .parameters
        .get(&function.definition)
        .map_or(&[][..], Vec::as_slice);
    let scheme = facts
        .lifetimes
        .functions
        .get(&function.definition)
        .ok_or("source intrinsic requires its completed lifetime scheme")?;
    if parameters.len() != usize::from(profile.types) + usize::from(profile.consts)
        || !scheme.early.is_empty()
        || usize::from(profile.types) > 4
    {
        return Err("source intrinsic has the wrong owned generic counts");
    }
    let mut formals = [TypeId::UNIT; 4];
    for (index, parameter) in parameters.iter().enumerate() {
        let id = match parameter.argument {
            Argument::Type(ty) if index < usize::from(profile.types) => {
                let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
                    return Err("source intrinsic lost its actual type formal");
                };
                formals[index] = ty;
                id
            }
            Argument::Const(id) if index >= usize::from(profile.types) => {
                let Some(ConstKind::Parameter(id)) = facts.types.const_kind(id) else {
                    return Err("source intrinsic lost its actual const formal");
                };
                id
            }
            _ => return Err("source intrinsic changed its ordered generic kinds"),
        };
        let actual = facts
            .types
            .parameter(id)
            .ok_or("missing intrinsic formal owner")?;
        if actual.owner.0 != function.definition.0
            || usize::try_from(actual.index).ok() != Some(index)
            || matches!(actual.kind, ParameterKind::Type { .. })
                != (index < usize::from(profile.types))
        {
            return Err("source intrinsic generic identity or position changed");
        }
    }
    Ok(formals)
}

#[cfg(test)]
#[path = "../../../tests/invariants/prototype_headers.rs"]
mod tests;
