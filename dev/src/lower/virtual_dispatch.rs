//! Selected virtual slots lower mechanically through the two-word receiver ABI.
mod forwarders;
mod members;
mod tables;
use super::context::{Context, Representation};
use crate::{
    lowir::{
        builder::Builder,
        program::{Operand, SymbolId, Terminator, Value},
        vocabulary::Type,
    },
    sema::vtable_plans::Virtual,
    types::{Kind, TypeId, instance::Instance},
};
pub(super) use forwarders::Callables;

pub(super) const SIZE_OFFSET: u64 = 8;
pub(super) const ALIGN_OFFSET: u64 = 16;
pub(super) const HEADER_BYTES: u64 = ALIGN_OFFSET + 8;

impl Context<'_> {
    pub(super) fn virtual_method(
        &mut self,
        instance: Instance,
        call: Virtual,
    ) -> Result<SymbolId, &'static str> {
        let key = (call.shape, call.slot);
        if let Some(&symbol) = self.virtual_symbols.get(&key) {
            return Ok(symbol);
        }
        let shape = self
            .objects
            .shape(call.shape)
            .ok_or("missing virtual receiver shape")?;
        let slot = shape
            .slots
            .get(call.slot as usize)
            .ok_or("invalid virtual method slot")?;
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = self.types.kind(instance.ty())
        else {
            return Err("virtual target requires a selected function item");
        };
        if slot.method
            != crate::sema::vtable_plans::Method::Declared(crate::resolve::model::DefId(item.0))
        {
            return Err("virtual method differs from its selected slot");
        }
        let Some(Kind::Function { parameters, .. }) = self.types.kind(signature) else {
            return Err("virtual method lacks its callable signature");
        };
        let receiver = *self
            .types
            .list(parameters)
            .and_then(|p| p.first())
            .ok_or("virtual method lacks a receiver")?;
        let Some(Kind::Reference { referent, .. }) = self.types.kind(receiver) else {
            return Err("virtual receiver requires a reference");
        };
        if !matches!((self.types.kind(referent), self.types.kind(shape.object)),
            (Some(Kind::Dynamic { shape: actual, .. }), Some(Kind::Dynamic { shape: expected, .. }))
                if actual == expected)
        {
            return Err("virtual receiver differs from its object bounds");
        }
        let name = format!("rg.virtual.{}.{}", call.shape.0, call.slot);
        let tracked = self
            .constants
            .caller_tracking(crate::resolve::model::DefId(item.0))?;
        let symbol = self.emit_virtual(signature, call, 0, name, tracked)?;
        self.virtual_symbols.insert(key, symbol);
        Ok(symbol)
    }

    pub(super) fn emit_virtual(
        &mut self,
        signature: TypeId,
        call: Virtual,
        dereferences: u32,
        name: String,
        tracked: bool,
    ) -> Result<SymbolId, &'static str> {
        let Some(Kind::Function { result, .. }) = self.types.kind(signature) else {
            return Err("virtual entry requires a function signature");
        };
        let receiver_index = usize::from(matches!(
            self.representation(result)?,
            Representation::Object(_)
        ));
        let signature = self.caller_signature(signature, tracked)?;
        let symbol = self
            .output
            .declare(self.internal_name(name), signature, false)?;
        let mut builder = Builder::default();
        let entry = builder.block()?;
        builder.enter(entry)?;
        let mut arguments = parameters(&self.output, signature)?;
        let mut receiver = *arguments
            .get(receiver_index)
            .ok_or("missing virtual ABI receiver")?;
        for _ in 0..dereferences {
            receiver = builder.load(Type::Ptr, receiver.operand)?;
        }
        let (data, method) = dispatch(&mut builder, receiver, call)?;
        arguments[receiver_index] = data;
        let native = self.output.scalar_signature(signature)?;
        let value = builder.call(signature, native.result, method.operand, &arguments)?;
        builder.terminate(if native.noreturn {
            Terminator::Unreachable
        } else {
            Terminator::Return(value)
        })?;
        self.output.define(symbol, builder.finish()?)?;
        Ok(symbol)
    }
}

pub(super) fn parameters(
    output: &crate::lowir::program::Program,
    signature: crate::lowir::program::SignatureId,
) -> Result<Vec<Value>, &'static str> {
    output
        .parameters(signature)?
        .iter()
        .enumerate()
        .map(|(index, &ty)| {
            Ok(Value {
                ty,
                operand: Operand::Parameter(
                    u32::try_from(index).map_err(|_| "too many callable arguments")?,
                ),
            })
        })
        .collect()
}

fn offset(builder: &mut Builder, base: Value, bytes: u64) -> Result<Value, &'static str> {
    let offset = builder.integer(Type::I64, u128::from(bytes))?;
    builder.index(Type::U8, base, offset)
}

fn dispatch(
    builder: &mut Builder,
    receiver: Value,
    call: Virtual,
) -> Result<(Value, Value), &'static str> {
    let data = builder.load(Type::Ptr, receiver.operand)?;
    let metadata = offset(builder, receiver, 8)?;
    let metadata = builder.load(Type::Ptr, metadata.operand)?;
    let address = offset(builder, metadata, HEADER_BYTES + u64::from(call.slot) * 8)?;
    let method = builder.load(Type::Ptr, address.operand)?;
    Ok((data, method))
}
