//! Nested inputs and actual returns retain ordinary formation/coercion obligations.
use super::{Error, constraints::Constraints};
use crate::{
    body::{ir::Body, regions::Contract},
    types::{Store, outlives::Queries},
};
pub(super) fn assume_inputs(
    types: &Store,
    queries: &mut Queries<'_>,
    input: &Contract,
    constraints: &mut Constraints<'_>,
) -> Result<(), Error> {
    let error = |message| Error {
        source: Some(input.source),
        message,
    };
    if input
        .output
        .as_ref()
        .is_some_and(|link| link.source != input.source || link.universals > input.universals)
    {
        return Err(error(
            "nested interface belongs to another body or revision".into(),
        ));
    }
    if let Some(formation) = input
        .output
        .as_ref()
        .and_then(|link| link.formation.as_ref())
    {
        if input.key != crate::body::ir::BodyKey::Closure(formation.identity) {
            return Err(error("formation input belongs to another closure".into()));
        }
        for formation in &formation.inputs {
            constraints
                .assume_well_formed(queries, formation.child)
                .map_err(error)?;
        }
    }
    if let Some(signature) = input.signature {
        if input.output.is_some() {
            let Some(crate::types::Kind::Function {
                parameters, result, ..
            }) = types.kind(signature)
            else {
                return Err(error("nested output has no callable signature".into()));
            };
            for &ty in types
                .list(parameters)
                .ok_or_else(|| error("missing nested inputs".into()))?
            {
                constraints.assume_well_formed(queries, ty).map_err(error)?;
            }
            constraints
                .require_requirements(
                    queries
                        .symbolic_requirements(result)
                        .map_err(|e| error(e.into()))?,
                    crate::types::relations::ProofScope::ROOT,
                    0,
                )
                .map_err(error)?;
        } else {
            constraints
                .assume_signature(queries, signature)
                .map_err(error)?;
        }
    }
    Ok(())
}
pub(super) fn require_return(
    types: &Store,
    input: &Contract,
    cfg: Option<&Body>,
    constraints: &mut Constraints<'_>,
    first_proof: usize,
) -> Result<(), Error> {
    let error = |message| Error {
        source: Some(input.source),
        message,
    };
    if let Some(cfg) = cfg
        && matches!(cfg.owner, crate::body::ir::Owner::Closure { .. })
        && let Some(crate::types::Kind::Function { result, .. }) =
            input.signature.and_then(|signature| types.kind(signature))
        && cfg.result != result
    {
        constraints
            .coerce(
                cfg.result,
                result,
                false,
                u32::try_from(first_proof)
                    .map_err(|_| error("too many return proof sources".into()))?,
            )
            .map_err(|e| error(e.to_string()))?;
    }
    Ok(())
}

pub(super) fn validate_body(input: &Contract, cfg: Option<&Body>) -> Result<(), Error> {
    let error = |message| Error {
        source: Some(input.source),
        message,
    };
    if cfg.is_some_and(|body| body.source != input.source || body.key() != input.key) {
        return Err(error(
            "nested region contract does not match its CFG".into(),
        ));
    }
    if let Some(Body {
        owner: crate::body::ir::Owner::ClosureAdapter { signature, .. },
        ..
    }) = cfg
        && input.signature != Some(*signature)
    {
        return Err(error(
            "receiver adapter contract differs from its CFG signature".into(),
        ));
    }
    if input.output.is_some() && cfg.is_some_and(|body| body.instance.is_some()) {
        return Err(error(
            "specialized nested output proof is not implemented".into(),
        ));
    }
    if let Some(crate::body::ir::Body {
        owner:
            crate::body::ir::Owner::Closure {
                execution: Some(signature),
                ..
            },
        ..
    }) = cfg
        && input.signature != Some(*signature)
    {
        return Err(error(
            "closure proof does not select its checked execution signature".into(),
        ));
    }
    Ok(())
}
