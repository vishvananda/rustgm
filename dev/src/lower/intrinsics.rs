//! Typed scalar operations and reified shims use the existing volatile vocabulary.
mod atomic;
mod counted;
mod memory;
mod overlapping;
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
    ) -> Result<Representation, &'static str> {
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
        if intrinsic.is_atomic() {
            return self
                .atomic_type(intrinsic, parameters, result)
                .map(Representation::Scalar);
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
        if intrinsic.is_counted_memory() {
            if mutable
                || parameters.len() != 3
                || result != TypeId::UNIT
                || parameters[2] != TypeId::primitive(crate::types::Primitive::Usize)
                || !matches!(self.types.kind(parameters[1]), Some(Kind::Pointer { mutable: true, pointee: target }) if target == pointee)
            {
                return Err("counted pointer memory operation signature mismatch");
            }
            return self.memory_type(pointee);
        }
        let writing = intrinsic.writes_pointer();
        if mutable != writing
            || parameters.len() != if writing { 2 } else { 1 }
            || writing && (parameters[1] != pointee || result != TypeId::UNIT)
            || !writing && result != pointee
        {
            return Err("volatile intrinsic parameter or result type mismatch");
        }
        if intrinsic.is_memory() {
            return self.memory_type(pointee);
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
        Ok(Representation::Scalar(ty))
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
        let representation = self.intrinsic_type(intrinsic, instance)?;
        let Some(Kind::FunctionItem { signature, .. }) = self.types.kind(instance.ty()) else {
            return Err("intrinsic shim requires a function item");
        };
        let signature = self.signature(signature)?;
        let body = if intrinsic.is_counted_memory() {
            Self::counted_memory_body(intrinsic, representation)?
        } else if intrinsic.is_memory() {
            Self::memory_body(intrinsic, representation)?
        } else if intrinsic.is_volatile() {
            let Representation::Scalar(ty) = representation else {
                return Err("volatile intrinsic has no scalar representation");
            };
            Self::volatile_body(intrinsic, ty)?
        } else {
            self.atomic_body(intrinsic, instance)?
        };
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
    fn volatile_body(
        intrinsic: Intrinsic,
        ty: Type,
    ) -> Result<crate::lowir::program::Function, &'static str> {
        let mut builder = Builder::default();
        let entry = builder.block()?;
        builder.enter(entry)?;
        let pointer = Operand::Parameter(0);
        let result = match intrinsic {
            Intrinsic::ReadVolatile => Some(builder.volatile_load(ty, pointer)?),
            Intrinsic::WriteVolatile => {
                builder.volatile_store(
                    Value {
                        ty,
                        operand: Operand::Parameter(1),
                    },
                    pointer,
                )?;
                None
            }
            _ => return Err("invalid volatile shim identity"),
        };
        builder.terminate(Terminator::Return(result))?;
        builder.finish()
    }
}
