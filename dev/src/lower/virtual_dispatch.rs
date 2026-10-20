//! Selected virtual slots lower mechanically through the two-word receiver ABI.
mod tables;
use super::context::{Context, Representation};
use crate::{
    lowir::{
        builder::Builder,
        program::{Operand, SymbolId, Terminator, Value},
        vocabulary::Type,
    },
    sema::vtable_plans::Virtual,
    types::{Kind, instance::Instance},
};

pub(super) const HEADER_BYTES: u64 = 24;

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
        if item.0 != slot.method.0 {
            return Err("virtual method differs from its selected slot");
        }
        let Some(Kind::Function {
            parameters, result, ..
        }) = self.types.kind(signature)
        else {
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
        let receiver_index = usize::from(matches!(
            self.representation(result)?,
            Representation::Object(_)
        ));
        let signature = self.signature(signature)?;
        let symbol = self.output.declare(
            format!("rg.virtual.{}.{}", call.shape.0, call.slot),
            signature,
            false,
        )?;
        let mut builder = Builder::default();
        let entry = builder.block()?;
        builder.enter(entry)?;
        let mut arguments = self
            .output
            .parameters(signature)
            .iter()
            .enumerate()
            .map(|(index, &ty)| {
                Ok(Value {
                    ty,
                    operand: Operand::Parameter(
                        u32::try_from(index).map_err(|_| "too many virtual arguments")?,
                    ),
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?;
        let receiver = *arguments
            .get(receiver_index)
            .ok_or("missing virtual ABI receiver")?;
        let data = builder.load(Type::Ptr, receiver.operand)?;
        let metadata = offset(&mut builder, receiver, 8)?;
        let metadata = builder.load(Type::Ptr, metadata.operand)?;
        let address = offset(
            &mut builder,
            metadata,
            HEADER_BYTES + u64::from(call.slot) * 8,
        )?;
        let method = builder.load(Type::Ptr, address.operand)?;
        arguments[receiver_index] = data;
        let native = &self.output.signatures[signature.0 as usize];
        let value = builder.call(signature, native.result, method.operand, &arguments)?;
        builder.terminate(if native.noreturn {
            Terminator::Unreachable
        } else {
            Terminator::Return(value)
        })?;
        self.output.symbols[symbol.0 as usize].body = Some(builder.finish()?);
        self.virtual_symbols.insert(key, symbol);
        Ok(symbol)
    }
}

fn offset(builder: &mut Builder, base: Value, bytes: u64) -> Result<Value, &'static str> {
    let offset = builder.integer(Type::I64, u128::from(bytes))?;
    builder.index(Type::U8, base, offset)
}
