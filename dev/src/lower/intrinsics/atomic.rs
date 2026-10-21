//! A bounded ordering switch translates Rust tags and aborts invalid dynamic orders.
mod comparison;
use crate::{
    lower::context::Context,
    lowir::{
        atomic::{LoadOrder, RmwOrder, StoreOrder},
        builder::Builder,
        program::{Function, Operand, Terminator, Value},
        vocabulary::{Binary, Predicate, Type},
    },
    support::{atomic_ordering::Ordering, intrinsics::Intrinsic},
    types::{Kind, ListId, Primitive, TypeId},
};
impl Context<'_> {
    pub(super) fn atomic_type(
        &mut self,
        intrinsic: Intrinsic,
        parameters: ListId,
        result: TypeId,
    ) -> Result<Type, &'static str> {
        let declaration = self
            .intrinsics
            .get(intrinsic)
            .ok_or("missing atomic intrinsic identity")?;
        let subject = declaration
            .owner
            .and_then(|id| self.definitions.get(id.0 as usize))
            .copied()
            .flatten()
            .ok_or("missing atomic intrinsic nominal owner")?;
        let inputs = self
            .types
            .list(parameters)
            .ok_or("missing atomic intrinsic parameters")?;
        let storing = intrinsic == Intrinsic::AtomicStore;
        let loading = intrinsic == Intrinsic::AtomicLoad;
        let scalar = TypeId::primitive(Primitive::U32);
        if inputs.len()
            != if loading {
                2
            } else if intrinsic.is_compare_exchange() {
                5
            } else {
                3
            }
            || !loading && inputs[1] != scalar
            || storing && result != TypeId::UNIT
            || !storing && !intrinsic.is_compare_exchange() && result != scalar
            || intrinsic.is_compare_exchange() && (inputs[2] != scalar || inputs[3] != inputs[4])
            || !matches!(self.types.kind(inputs[0]), Some(Kind::Reference { mutable: false, referent, .. }) if referent == subject)
        {
            return Err("atomic intrinsic parameter or result type mismatch");
        }
        if intrinsic.is_compare_exchange() {
            self.comparison_result_type(result)?;
        }
        let canonical = self.definitions[declaration.definition.0 as usize]
            .ok_or("missing atomic callable definition")?;
        let Some(Kind::FunctionItem { signature, .. }) = self.types.kind(canonical) else {
            return Err("invalid atomic callable identity");
        };
        let Some(Kind::Function {
            parameters: expected,
            ..
        }) = self.types.kind(signature)
        else {
            return Err("missing atomic callable signature");
        };
        if self.types.list(expected).and_then(|values| values.last()) != inputs.last() {
            return Err("atomic operation requires canonical Ordering");
        }
        let layout = self.layout(subject)?;
        if layout.size != 4 || layout.align != 4 {
            return Err("atomic u32 storage requires natural four-byte layout");
        }
        Ok(Type::U32)
    }
    pub(super) fn atomic_body(
        &mut self,
        intrinsic: Intrinsic,
        instance: crate::types::instance::Instance,
    ) -> Result<Function, &'static str> {
        if intrinsic.is_compare_exchange() {
            return self.comparison_body(instance);
        }
        let mut builder = Builder::default();
        let entry = builder.block()?;
        let invalid = builder.block()?;
        builder.enter(entry)?;
        let tag = builder.load(
            Type::U8,
            Operand::Parameter(if intrinsic == Intrinsic::AtomicLoad {
                1
            } else {
                2
            }),
        )?;
        for order in Ordering::ALL {
            if !intrinsic.atomic_order_valid(order) {
                continue;
            }
            let operation = builder.block()?;
            let next = builder.block()?;
            let expected = builder.integer(Type::U8, u128::from(order.tag()))?;
            let equal = builder.compare(Predicate::Eq, tag, expected)?;
            builder.terminate(Terminator::Branch {
                condition: equal.operand,
                yes: operation,
                no: next,
            })?;
            builder.enter(operation)?;
            let result = atomic_operation(&mut builder, intrinsic, order)?;
            builder.terminate(Terminator::Return(result))?;
            builder.enter(next)?;
        }
        builder.terminate(Terminator::Jump(invalid))?;
        builder.enter(invalid)?;
        let abort = self.abort()?;
        let signature = self.output.symbols[abort.0 as usize].signature;
        builder.call(signature, Type::Void, Operand::Symbol(abort), &[])?;
        builder.terminate(Terminator::Unreachable)?;
        builder.finish()
    }
}
fn atomic_operation(
    builder: &mut Builder,
    intrinsic: Intrinsic,
    order: Ordering,
) -> Result<Option<Value>, &'static str> {
    let pointer = Operand::Parameter(0);
    if intrinsic == Intrinsic::AtomicStore {
        let order = match order {
            Ordering::Relaxed => StoreOrder::Relaxed,
            Ordering::Release => StoreOrder::Release,
            Ordering::Sequential => StoreOrder::Sequential,
            _ => return Err("invalid atomic store order"),
        };
        builder.atomic_store(
            Value {
                ty: Type::U32,
                operand: Operand::Parameter(1),
            },
            pointer,
            order,
        )?;
        Ok(None)
    } else if intrinsic == Intrinsic::AtomicLoad {
        let order = match order {
            Ordering::Relaxed => LoadOrder::Relaxed,
            Ordering::Acquire => LoadOrder::Acquire,
            Ordering::Sequential => LoadOrder::Sequential,
            _ => return Err("invalid atomic load order"),
        };
        builder.atomic_load(Type::U32, pointer, order).map(Some)
    } else {
        let order = match order {
            Ordering::Relaxed => RmwOrder::Relaxed,
            Ordering::Release => RmwOrder::Release,
            Ordering::Acquire => RmwOrder::Acquire,
            Ordering::AcquireRelease => RmwOrder::AcquireRelease,
            Ordering::Sequential => RmwOrder::Sequential,
        };
        let value = Value {
            ty: Type::U32,
            operand: Operand::Parameter(1),
        };
        let result = match intrinsic {
            Intrinsic::AtomicSwap => builder.atomic_exchange(value, pointer, order)?,
            Intrinsic::AtomicFetchAdd => {
                let updated = builder.atomic_add_fetch(value, pointer, order)?;
                // Rust returns the old value. Pure wrapping subtraction uses the
                // captured atomic result; it performs no additional memory read.
                builder.binary(Binary::Sub, updated, value)?
            }
            Intrinsic::AtomicFetchSub => {
                let zero = builder.integer(Type::U32, 0)?;
                let delta = builder.binary(Binary::Sub, zero, value)?;
                let updated = builder.atomic_add_fetch(delta, pointer, order)?;
                // Both arithmetic operations wrap at u32 width. The captured
                // result supplies the old value without another atomic read.
                builder.binary(Binary::Add, updated, value)?
            }
            _ => return Err("invalid atomic RMW identity"),
        };
        Ok(Some(result))
    }
}
