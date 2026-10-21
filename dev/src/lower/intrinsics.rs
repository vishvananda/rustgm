//! Typed scalar operations and reified shims use the existing volatile vocabulary.
use super::context::{Context, Representation};
use crate::{
    lowir::{
        builder::Builder,
        program::{Operand, SymbolId, Terminator, Value},
        vocabulary::Type,
    },
    resolve::model::DefId,
    support::intrinsics::{Intrinsic, Safety},
    types::{Kind, Primitive, TypeId, callable::Convention, instance::Instance},
};

impl Context<'_> {
    pub(super) fn intrinsic_type(
        &mut self,
        intrinsic: Intrinsic,
        instance: Instance,
    ) -> Result<Type, &'static str> {
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = self.types.kind(instance.ty())
        else {
            return Err("intrinsic requires a canonical function item");
        };
        if self.intrinsics.identify(DefId(item.0)) != Some(intrinsic) {
            return Err("intrinsic identity differs from its selected callable");
        }
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_,
            convention,
            ..
        }) = self.types.kind(signature)
        else {
            return Err("intrinsic has no selected callable signature");
        };
        if unsafe_ != (intrinsic.descriptor().safety == Safety::Unsafe)
            || convention != Convention::RUST
        {
            return Err("intrinsic callable permissions differ from its descriptor");
        }
        let parameters = self
            .types
            .list(parameters)
            .ok_or("missing intrinsic parameters")?;
        let Some(Kind::Pointer { mutable, pointee }) =
            parameters.first().and_then(|&ty| self.types.kind(ty))
        else {
            return Err("volatile intrinsic requires a raw pointer");
        };
        let writing = intrinsic == Intrinsic::WriteVolatile;
        if mutable != writing
            || parameters.len() != if writing { 2 } else { 1 }
            || writing && (parameters[1] != pointee || result != TypeId::UNIT)
            || !writing && result != pointee
        {
            return Err("volatile intrinsic parameter or result type mismatch");
        }
        let pointee = self.types.storage_head(pointee)?;
        match self.types.kind(pointee) {
            Some(Kind::Primitive(primitive)) if primitive != Primitive::Str => {}
            Some(Kind::Pointer { .. } | Kind::Reference { .. } | Kind::Function { .. }) => {}
            _ => return Err("aggregate and zero-sized volatile operations are not implemented"),
        }
        let Representation::Scalar(ty) = self.representation(pointee)? else {
            return Err("aggregate and wide-pointer volatile operations are not implemented");
        };
        let layout = self.layout(pointee)?;
        if ty == Type::Void
            || layout.size != u64::from(ty.bits() / 8)
            || !layout.align.is_power_of_two()
        {
            return Err("volatile scalar layout differs from its operation width or alignment");
        }
        Ok(ty)
    }
    pub(super) fn demand_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        instance: Instance,
    ) -> Result<SymbolId, &'static str> {
        if let Some(symbol) = self.intrinsic_symbols.get(&instance) {
            return Ok(*symbol);
        }
        if self.intrinsic_symbols.len() >= 100_000 {
            return Err("intrinsic shim count exceeds 100000");
        }
        let ty = self.intrinsic_type(intrinsic, instance)?;
        let Some(Kind::FunctionItem { signature, .. }) = self.types.kind(instance.ty()) else {
            return Err("intrinsic shim requires a function item");
        };
        let signature = self.signature(signature)?;
        let mut builder = Builder::default();
        let entry = builder.block()?;
        builder.enter(entry)?;
        let pointer = Value {
            ty: Type::Ptr,
            operand: Operand::Parameter(0),
        };
        let result = match intrinsic {
            Intrinsic::ReadVolatile => Some(builder.volatile_load(ty, pointer.operand)?),
            Intrinsic::WriteVolatile => {
                builder.volatile_store(
                    Value {
                        ty,
                        operand: Operand::Parameter(1),
                    },
                    pointer.operand,
                )?;
                None
            }
        };
        builder.terminate(Terminator::Return(result))?;
        let body = builder.finish()?;
        let symbol = self.output.declare(
            self.internal_name(format!(
                "rg.intrinsic.{}.t{}",
                intrinsic.name(),
                instance.ty().0
            )),
            signature,
            false,
        )?;
        self.output.define(symbol, body)?;
        self.intrinsic_symbols.insert(instance, symbol);
        Ok(symbol)
    }
}
