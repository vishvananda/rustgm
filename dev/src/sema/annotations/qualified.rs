//! Qualified paths preserve inferred inputs separately from the associated result.
use super::{Host, applications, error, term};
use crate::{
    resolve::{package::Error, unit::NodeKey},
    sema::generics::{AnnotationProjection, parameter_id},
    types::{
        ItemId,
        arguments::Argument as Canonical,
        infer::{TermId, arguments::Argument},
        lifetime_proof::ScopeId,
    },
};

pub(super) fn infer(
    host: &mut impl Host,
    key: NodeKey,
    scope: ScopeId,
    path: AnnotationProjection,
) -> Result<TermId, Error> {
    let subject = term(host, path.subject)?;
    let owner = host
        .facts()
        .associated
        .trait_owner(path.declaration)
        .ok_or_else(|| error(host, key, "missing projection trait owner"))?;
    let self_type = host
        .facts()
        .associated
        .schema(owner)
        .ok_or_else(|| error(host, key, "missing projection trait schema"))?
        .self_type;
    let self_parameter = parameter_id(&host.facts().types, Canonical::Type(self_type))
        .map_err(|e| error(host, key, e))?;
    let (parent, _) = applications::arguments(
        host,
        path.parent,
        scope,
        vec![(self_parameter, Argument::Type(subject))],
    )?;
    let (member, _) = applications::arguments(host, path.member, scope, Vec::new())?;
    let (facts, infer) = host.inference();
    let result = infer
        .projected(
            &facts.types,
            ItemId(path.declaration.0),
            subject,
            &parent,
            &member,
        )
        .map_err(|e| error(host, key, e))?;
    // Keep the original operation until all inputs and their formation proofs
    // complete, even when the selected definition erases one of those inputs.
    host.normalization(key, scope, result)?;
    Ok(result)
}
