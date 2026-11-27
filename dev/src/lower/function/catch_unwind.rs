//! The abort target invokes the checked try callback and returns normally with false.
use super::{Lower, Selected, ir};
use crate::{
    lowir::vocabulary::Type,
    types::{Kind, Primitive, Store, TypeId, callable::Convention, instance::Instance},
};

pub(in crate::lower) struct CatchUnwind {
    callee: ir::ValueId,
    callback: ir::ValueId,
    data: ir::ValueId,
    signature: TypeId,
    pointee: TypeId,
}

pub(in crate::lower) fn checked_catch_unwind(
    types: &Store,
    body: &ir::Body,
    value: &ir::Value,
    target: TypeId,
) -> Result<CatchUnwind, &'static str> {
    if !crate::resolve::package::Package::target_aborts_panics() || body.const_context {
        return Err("source catch_unwind requires its actual abort-profile runtime call");
    }
    let ir::Rvalue::Call {
        signature,
        callee,
        arguments,
    } = value.kind
    else {
        return Err("source catch_unwind requires a checked direct call");
    };
    let Some(Kind::FunctionItem {
        signature: selected,
        ..
    }) = types.kind(target)
    else {
        return Err("source catch_unwind lost its actual function item");
    };
    let [callback, data, catch] = *body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("source catch_unwind lost its three actual arguments")?
    else {
        return Err("source catch_unwind requires its three actual arguments");
    };
    let value_type = |id: ir::ValueId| {
        body.values
            .get(id.0 as usize)
            .map(|input| input.ty)
            .ok_or("source catch_unwind lost an argument type")
    };
    let (callback_ty, data_ty, catch_ty) =
        (value_type(callback)?, value_type(data)?, value_type(catch)?);
    let Some(Kind::Pointer {
        mutable: true,
        pointee,
    }) = types.kind(data_ty)
    else {
        return Err("source catch_unwind requires its mutable data pointer");
    };
    let bool_ty = TypeId::primitive(Primitive::Bool);
    if value.ty != bool_ty
        || rust_inputs(types, signature, bool_ty) != Some(&[callback_ty, data_ty, catch_ty])
    {
        return Err("source catch_unwind differs from its checked callback and data signature");
    }
    let Some([callback_data]) = rust_inputs(types, callback_ty, TypeId::UNIT) else {
        return Err("source catch_unwind requires its unsafe Rust try callback");
    };
    let Some([catch_data, payload]) = rust_inputs(types, catch_ty, TypeId::UNIT) else {
        return Err("source catch_unwind requires its unsafe Rust catch callback");
    };
    if callback_data != catch_data
        || !matches!(
            types.kind(*payload),
            Some(Kind::Pointer { mutable: true, pointee }) if pointee == TypeId::primitive(Primitive::U8)
        )
    {
        return Err(
            "source catch_unwind requires a shared callback data type and mutable byte payload",
        );
    }
    // The checked call owns its lifetime relation. Preparing its ABI only
    // compares type structure, since the caller and the specialized
    // callback may name the same payload with different region identities.
    require_same_shape(types, selected, signature)?;
    require_same_shape(types, *callback_data, data_ty)?;
    Ok(CatchUnwind {
        callee,
        callback,
        data,
        signature: callback_ty,
        pointee,
    })
}

fn rust_inputs(types: &Store, ty: TypeId, output: TypeId) -> Option<&[TypeId]> {
    match types.kind(ty)? {
        Kind::Function {
            bound_regions: 0,
            parameters,
            result,
            unsafe_: true,
            convention: Convention::RUST,
        } if result == output => types.list(parameters),
        _ => None,
    }
}

fn require_same_shape(types: &Store, a: TypeId, b: TypeId) -> Result<(), &'static str> {
    if a == b {
        return Ok(());
    }
    let mismatch = |_| "source catch_unwind data and callbacks have different runtime types";
    let mut context = crate::types::infer::Context::default();
    let actual = context.known(types, a).map_err(mismatch)?;
    let expected = context.known(types, b).map_err(mismatch)?;
    context
        .equate_shape(types, actual, expected)
        .map_err(mismatch)
}

impl Lower<'_, '_, '_> {
    pub(super) fn abort_profile_catch_unwind(
        &mut self,
        id: ir::ValueId,
        ty: TypeId,
    ) -> Result<Selected, &'static str> {
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("reached source catch_unwind call disappeared")?;
        let call = checked_catch_unwind(self.context.types, self.body, value, ty)?;
        let instance = Instance::new(self.context.types, ty)?;
        if !matches!(self.value(call.callee)?, Selected::Item(actual) if actual == self.body.callable_target(instance))
        {
            return Err("source catch_unwind callee differs from its checked instance");
        }
        if self.context.wide_referent(call.pointee)? {
            return Err("source catch_unwind requires thin data storage");
        }
        let callback = self.value(call.callback)?.runtime()?;
        let data = self.value(call.data)?.runtime()?;
        if callback.ty != Type::Ptr || data.ty != Type::Ptr {
            return Err("source catch_unwind lost its callback or data pointer");
        }
        let signature = self.context.signature(call.signature)?;
        self.builder
            .call(signature, Type::Void, callback.operand, &[data])?;
        self.builder.integer(Type::I1, 0).map(Selected::Runtime)
    }
}
