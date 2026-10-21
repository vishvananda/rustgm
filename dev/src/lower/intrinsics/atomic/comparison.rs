//! CAS returns canonical Result storage using ordinary variant layout and return ABI.
use crate::{
    abi::layout::Shape,
    lower::{address::Address, context::Context},
    lowir::{
        atomic::{CompareOrder, LoadOrder, RmwOrder},
        builder::Builder,
        program::{Function, Operand, Terminator, Value},
        vocabulary::{Binary, Predicate, Type},
    },
    resolve::model::DefId,
    sema::members::{ConstructorKind, FieldId, Selection},
    support::atomic_ordering::Ordering,
    types::{Kind, Primitive, TypeId, arguments::Argument, instance::Instance},
};
impl Context<'_> {
    pub(super) fn comparison_result_type(&self, ty: TypeId) -> Result<(), &'static str> {
        let identity = self
            .intrinsics
            .comparison_result()
            .ok_or("missing canonical comparison Result")?;
        let Some(Kind::Adt(owner, arguments)) = self.types.kind(ty) else {
            return Err("comparison result requires canonical Result");
        };
        let scalar = TypeId::primitive(Primitive::U32);
        let arguments = self
            .types
            .arguments(arguments)
            .ok_or("missing comparison Result arguments")?;
        if owner.0 != identity.definition.0 || !arguments.eq([Argument::Type(scalar); 2]) {
            return Err("comparison result differs from Result<u32,u32>");
        }
        Ok(())
    }
    pub(super) fn comparison_body(&mut self, instance: Instance) -> Result<Function, &'static str> {
        let Some(Kind::FunctionItem { signature, .. }) = self.types.kind(instance.ty()) else {
            return Err("comparison requires a callable item");
        };
        let Some(Kind::Function { result, .. }) = self.types.kind(signature) else {
            return Err("comparison requires a callable signature");
        };
        self.comparison_result_type(result)?;
        let mut builder = Builder::default();
        let entry = builder.block()?;
        let invalid = builder.block()?;
        builder.enter(entry)?;
        let expected = builder.slot(Type::U32)?;
        builder.store(
            Value {
                ty: Type::U32,
                operand: Operand::Parameter(2),
            },
            Operand::Slot(expected),
        )?;
        let pointer = builder.address(Operand::Slot(expected))?;
        let success = builder.load(Type::U8, Operand::Parameter(4))?;
        let failure = builder.load(Type::U8, Operand::Parameter(5))?;
        for success_order in Ordering::ALL {
            for failure_order in Ordering::ALL {
                let Some(ordering) = orders(success_order, failure_order) else {
                    continue;
                };
                let selected = builder.block()?;
                let next = builder.block()?;
                let a = builder.integer(Type::U8, u128::from(success_order.tag()))?;
                let b = builder.integer(Type::U8, u128::from(failure_order.tag()))?;
                let a = builder.compare(Predicate::Eq, success, a)?;
                let b = builder.compare(Predicate::Eq, failure, b)?;
                let both = builder.binary(Binary::And, a, b)?;
                builder.terminate(Terminator::Branch {
                    condition: both.operand,
                    yes: selected,
                    no: next,
                })?;
                builder.enter(selected)?;
                let flag = builder.atomic_compare_exchange(
                    Value {
                        ty: Type::U32,
                        operand: Operand::Parameter(3),
                    },
                    Operand::Parameter(1),
                    pointer.operand,
                    ordering,
                )?;
                // The backend preserves expected on success and updates it with
                // the observed value on failure. This reads only local scratch.
                let observed = builder.load(Type::U32, Operand::Slot(expected))?;
                self.comparison_return(&mut builder, result, flag, observed)?;
                builder.enter(next)?;
            }
        }
        builder.terminate(Terminator::Jump(invalid))?;
        builder.enter(invalid)?;
        let abort = self.abort()?;
        let signature = self.output.symbols[abort.0 as usize].signature;
        builder.call(signature, Type::Void, Operand::Symbol(abort), &[])?;
        builder.terminate(Terminator::Unreachable)?;
        builder.finish()
    }
    fn comparison_return(
        &mut self,
        builder: &mut Builder,
        ty: TypeId,
        flag: Value,
        observed: Value,
    ) -> Result<(), &'static str> {
        let identity = self
            .intrinsics
            .comparison_result()
            .ok_or("missing comparison Result identity")?;
        let yes = builder.block()?;
        let no = builder.block()?;
        builder.terminate(Terminator::Branch {
            condition: flag.operand,
            yes,
            no,
        })?;
        for (block, constructor) in [(yes, identity.ok), (no, identity.err)] {
            builder.enter(block)?;
            self.comparison_variant(builder, ty, constructor, observed)?;
            builder.terminate(Terminator::Return(None))?;
        }
        Ok(())
    }
    fn comparison_variant(
        &mut self,
        builder: &mut Builder,
        ty: TypeId,
        constructor: DefId,
        value: Value,
    ) -> Result<(), &'static str> {
        let identity = self
            .intrinsics
            .comparison_result()
            .ok_or("missing comparison Result identity")?;
        let selected = self
            .members
            .constructors
            .get(&constructor)
            .filter(|c| c.kind == ConstructorKind::Variant)
            .ok_or("comparison Result requires a canonical variant constructor")?;
        let member = self
            .members
            .variants
            .get(selected.variant.0 as usize)
            .ok_or("missing comparison Result variant")?;
        if member.owner.0 != identity.definition.0
            || member.end.checked_sub(member.start) != Some(1)
        {
            return Err("comparison Result variant differs from its canonical owner");
        }
        let layout = self.layout(ty)?;
        let Shape::Aggregate { tag: Some(tag), .. } = layout.shape else {
            return Err("comparison Result has no completed variant tag");
        };
        let variant = self
            .layouts
            .variants(&layout)
            .get(member.ordinal as usize)
            .ok_or("missing comparison Result variant layout")?;
        let tag_type =
            super::super::super::context::representation(self.types, TypeId::primitive(tag.ty))?;
        let tag_value = builder.integer(tag_type, variant.discriminant)?;
        let base = Address {
            operand: Operand::Parameter(0),
            align: layout.align,
        };
        let destination = base.offset(builder, tag.offset)?;
        builder.store(tag_value, destination.operand)?;
        let (field, offset) =
            self.selected_field(&layout, Selection::Nominal(FieldId(member.start)))?;
        if field != TypeId::primitive(Primitive::U32) || value.ty != Type::U32 {
            return Err("comparison Result payload differs from u32");
        }
        let destination = base.offset(builder, offset)?;
        builder.store(value, destination.operand)
    }
}
fn orders(success: Ordering, failure: Ordering) -> Option<CompareOrder> {
    let success = match success {
        Ordering::Relaxed => RmwOrder::Relaxed,
        Ordering::Release => RmwOrder::Release,
        Ordering::Acquire => RmwOrder::Acquire,
        Ordering::AcquireRelease => RmwOrder::AcquireRelease,
        Ordering::Sequential => RmwOrder::Sequential,
    };
    let failure = match failure {
        Ordering::Relaxed => LoadOrder::Relaxed,
        Ordering::Acquire => LoadOrder::Acquire,
        Ordering::Sequential => LoadOrder::Sequential,
        Ordering::Release | Ordering::AcquireRelease => return None,
    };
    Some(CompareOrder::new(success, failure))
}
