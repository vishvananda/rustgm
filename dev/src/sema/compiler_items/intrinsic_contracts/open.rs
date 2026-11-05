//! Typed one-reference intrinsics share the actual source-call opening law.
use crate::{
    resolve::{expansion::Report, unit::NodeKey},
    sema::model::{BodyRegionOrigin, Declarations, Operation},
    syntax::ast::Kind as Syntax,
    types::{Kind, RegionId, TypeId},
};
#[derive(Clone, Copy)]
pub(in crate::sema::compiler_items) struct Reference {
    pub mutable: bool,
    pub referent: TypeId,
}
pub(in crate::sema::compiler_items) enum Error {
    MissingCall,
    Signature,
    MissingRegion,
    RegionOwner,
    Syntax,
    Callee,
    Arguments,
    MissingInput,
    UntypedInput,
    Input,
    Effect(&'static str),
}
pub(in crate::sema::compiler_items) fn checked(
    report: &Report,
    facts: &Declarations,
    owner: NodeKey,
    call: NodeKey,
    expected: (TypeId, TypeId),
    regions: &[RegionId],
    input: Reference,
) -> Result<(), Error> {
    let (callable, signature) = expected;
    let Some(Operation::Call {
        signature: actual,
        source_use: Some(source_use),
    }) = facts.node(call).and_then(|fact| fact.operation)
    else {
        return Err(Error::MissingCall);
    };
    if actual != signature {
        return Err(Error::Signature);
    }
    source_use.effect_for(facts, owner).map_err(Error::Effect)?;
    for &region in regions {
        let allocated = facts
            .body_regions
            .get(owner, region)
            .ok_or(Error::MissingRegion)?;
        if allocated.body != owner
            || allocated.source != call
            || !matches!(allocated.origin, BodyRegionOrigin::CallArgument)
        {
            return Err(Error::RegionOwner);
        }
    }
    let ast = &report.package.unit(call.file).configured.ast;
    let Syntax::Call { callee, arguments } = ast.node(call.node).kind else {
        return Err(Error::Syntax);
    };
    if facts
        .node(NodeKey {
            node: callee,
            ..call
        })
        .and_then(|fact| fact.ty)
        != Some(callable)
    {
        return Err(Error::Callee);
    }
    let [argument] = ast.list(arguments) else {
        return Err(Error::Arguments);
    };
    let actual = facts
        .node(NodeKey {
            node: *argument,
            ..call
        })
        .ok_or(Error::MissingInput)?;
    let ty = actual
        .adjustment
        .map(|adjustment| adjustment.target)
        .or(actual.ty)
        .ok_or(Error::UntypedInput)?;
    if !matches!(facts.types.kind(ty),Some(Kind::Reference{mutable,referent,..}) if mutable==input.mutable && referent==input.referent)
    {
        return Err(Error::Input);
    }
    Ok(())
}
