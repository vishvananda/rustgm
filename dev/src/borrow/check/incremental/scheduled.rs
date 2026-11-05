//! A future request borrows its actual operation from an immutable proved caller.
use super::Proved;
use crate::{
    body::ir::{Body, Rvalue, ValueId},
    types::{Kind, Store, TypeId, arena::Identity, instance::Instance},
};

/// This witness supplies caller provenance, never a generated-family permit.
pub(crate) struct FutureCaller<'a> {
    body: &'a Body,
    arena: &'a Identity,
    instance: Instance,
    site: ValueId,
    signature: TypeId,
}
impl Proved {
    pub(crate) fn future_caller<'a>(
        &'a self,
        types: &Store,
        arena: &'a Identity,
        site: ValueId,
        instance: Instance,
    ) -> Result<FutureCaller<'a>, &'static str> {
        if !types.has_identity(arena) {
            return Err("future caller belongs to another actual type store");
        }
        let signature = operation(types, &self.body, site, instance)?;
        Ok(FutureCaller {
            body: &self.body,
            arena,
            instance,
            site,
            signature,
        })
    }
}
impl FutureCaller<'_> {
    pub(crate) fn require_current(
        &self,
        types: &Store,
        instance: Instance,
    ) -> Result<TypeId, &'static str> {
        if !types.has_identity(self.arena)
            || self.instance != instance
            || operation(types, self.body, self.site, instance)? != self.signature
        {
            return Err("future caller changed its actual arena, operation or full instance");
        }
        Ok(self.signature)
    }
}

fn operation(
    types: &Store,
    body: &Body,
    site: ValueId,
    expected: Instance,
) -> Result<TypeId, &'static str> {
    let value = body
        .values
        .get(site.0 as usize)
        .ok_or("future caller lost its actual scheduled operation")?;
    let (callee, signature) = match value.kind {
        Rvalue::Call {
            callee, signature, ..
        } => (callee, signature),
        Rvalue::Reify(callee) => (callee, value.ty),
        _ => return Err("future caller requires its actual Call or Reify operation"),
    };
    let ty = body
        .values
        .get(callee.0 as usize)
        .ok_or("future caller lost its actual callee operand")?
        .ty;
    let instance = body.callable_target(Instance::new(types, ty)?);
    let Some(Kind::FunctionItem {
        signature: public, ..
    }) = types.kind(instance.ty())
    else {
        return Err("future caller resolved callee is not an actual function item");
    };
    let Some(Kind::Function { result, .. }) = types.kind(signature) else {
        return Err("future caller lost its actual scheduled signature");
    };
    if instance != expected
        || types.contains_parameters(instance.ty()) != Some(false)
        || types.contains_projections(instance.ty()) != Some(false)
        || match value.kind {
            Rvalue::Reify(_) => signature != public,
            Rvalue::Call { .. } => result != value.ty,
            _ => true,
        }
    {
        return Err("future caller changed its actual full target, signature or result");
    }
    Ok(signature)
}

const _: [(); 32] = [(); size_of::<FutureCaller<'_>>()];
