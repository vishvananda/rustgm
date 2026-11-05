//! Actual ellipsis source identities authenticate the body-owned storage plan.
use super::application;
use crate::{
    resolve::{body_origin::BodyOrigin, model::DefId, unit::NodeKey},
    sema::model::{BodyRegionOrigin, Declarations, Operation},
    types::TypeId,
};
#[derive(Clone, Copy)]
pub(crate) struct BindingRequest {
    pub definition: DefId,
    pub signature: TypeId,
    pub body: BodyOrigin,
    pub parameter: NodeKey,
    pub ty: TypeId,
    pub binding: Option<DefId>,
}
pub(crate) fn require(facts: &Declarations, request: BindingRequest) -> Result<(), &'static str> {
    let BindingRequest {
        definition,
        signature,
        body,
        parameter: source,
        ty,
        binding,
    } = request;
    let function = facts
        .function_indices
        .get(&definition)
        .and_then(|index| facts.functions.get(*index))
        .ok_or("variadic initialization lost its completed source function")?;
    let header = facts
        .function_header(definition)?
        .ok_or("variadic function header is pending")?;
    if function.signature != signature
        || function.body != body.source()
        || header.signature != signature
        || header
            .declaration
            .source()
            .is_none_or(|source| source != function.node)
    {
        return Err("variadic initialization changed its actual function/header/body provenance");
    }
    let fact = facts
        .node(source)
        .ok_or("variadic parameter fact disappeared")?;
    if !matches!(fact.operation, Some(Operation::Variadic))
        || fact.ty != Some(ty)
        || fact.binding != binding
    {
        return Err("variadic local differs from its actual parameter/type/binding fact");
    }
    let (_, region) = application(facts, ty)?;
    let owner = body
        .source()
        .ok_or("imported variadic local requires a wider transport profile")?;
    let region = facts
        .body_regions
        .get(owner, region)
        .ok_or("variadic binding has no real body-local region")?;
    if region.body != owner
        || region.source != source
        || !matches!(region.origin, BodyRegionOrigin::Inference)
    {
        return Err("variadic binding borrowed another source parameter's lifetime");
    }
    Ok(())
}
