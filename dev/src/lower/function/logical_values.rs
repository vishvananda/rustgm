//! Declared discriminants are independent of physical tag width and nullable niches.
use super::{Address, Lower, Selected, ir};
use crate::{
    abi::layout::Shape,
    lowir::{
        program::Value,
        vocabulary::{Binary, Predicate, Type},
    },
    types::{Kind, TypeId, aggregate, instance::Instance},
};

impl Lower<'_, '_, '_> {
    pub(super) fn source_discriminant(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("source discriminant Call disappeared")?;
        let ir::Rvalue::Call {
            signature,
            callee,
            arguments,
        } = value.kind
        else {
            return Ok(None);
        };
        let ty = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("source discriminant callee disappeared")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(None);
        }
        let instance = Instance::new(self.context.types, ty)?;
        let Some(selected) = self
            .context
            .constants
            .source_discriminant(instance, self.body.source)?
        else {
            return Ok(None);
        };
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let [input] = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source discriminant argument range disappeared")?
        else {
            return Err("source discriminant requires one evaluated shared reference");
        };
        let input = *input;
        if !selected.opened_signature_matches(self.context.types, signature)
            || value.ty != selected.output()
            || self
                .body
                .values
                .get(input.0 as usize)
                .is_none_or(|v| !selected.input_matches(self.context.types, v.ty))
            || !matches!(self.value(callee)?, Selected::Item(actual) if actual == self.body.callable_target(instance))
        {
            return Err("source discriminant differs from its actual complete Call");
        }
        let Selected::Runtime(pointer) = self.value(input)? else {
            return Err("source discriminant shared reference has no actual pointer value");
        };
        let output = super::super::context::representation(self.context.types, selected.output())?;
        let is_enum = match self.context.types.kind(selected.referent()) {
            Some(Kind::Adt(item, _)) => self
                .context
                .aggregates
                .get(item)
                .is_some_and(|d| d.kind == aggregate::Kind::Enum),
            _ => false,
        };
        let result = if is_enum {
            self.logical_discriminant(pointer, selected.referent(), selected.output(), output)?
        } else {
            self.builder.integer(output, 0)?
        };
        Ok(Some(Selected::Runtime(result)))
    }

    fn logical_discriminant(
        &mut self,
        pointer: Value,
        ty: TypeId,
        output: TypeId,
        target: Type,
    ) -> Result<Value, &'static str> {
        let layout = self.context.layout(ty)?;
        let address = Address {
            operand: pointer.operand,
            align: layout.align,
        };
        if let Some(niche) = self.context.layouts.niche(&layout)?.copied() {
            let variants = self.context.layouts.variants(&layout);
            let empty = variants
                .get(niche.empty as usize)
                .ok_or("missing empty logical variant")?
                .discriminant;
            let payload = variants
                .get(niche.payload as usize)
                .ok_or("missing payload logical variant")?
                .discriminant;
            let empty_test = self.niche_test(address, niche, Predicate::Eq)?;
            let selected =
                super::super::casts::integer(&mut self.builder, empty_test, target, false)?;
            let Some(Kind::Primitive(primitive)) = self.context.types.kind(output) else {
                return Err("logical enum result is not an integer");
            };
            let (_, bits) = primitive
                .integer()
                .ok_or("logical enum result is not an integer")?;
            let mask = if bits == 128 {
                u128::MAX
            } else {
                (1_u128 << bits) - 1
            };
            let difference = self
                .builder
                .integer(target, empty.wrapping_sub(payload) & mask)?;
            let difference = self.builder.binary(Binary::Mul, selected, difference)?;
            let payload = self.builder.integer(target, payload)?;
            return self.builder.binary(Binary::Add, payload, difference);
        }
        if let Shape::Aggregate { tag: Some(tag), .. } = layout.shape {
            let tag_address = self.offset(address, tag.offset)?;
            let tag_type = super::super::context::representation(
                self.context.types,
                TypeId::primitive(tag.ty),
            )?;
            let physical = self.builder.load(tag_type, tag_address.operand)?;
            return super::super::casts::integer(
                &mut self.builder,
                physical,
                target,
                tag.ty.integer().is_some_and(|(signed, _)| signed),
            );
        }
        let variants = self.context.layouts.variants(&layout);
        let [variant] = variants else {
            return Err("enum without a physical tag must have one genuine variant");
        };
        self.builder.integer(target, variant.discriminant)
    }
}
