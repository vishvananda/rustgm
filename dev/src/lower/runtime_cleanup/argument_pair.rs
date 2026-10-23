//! A checked source reference is thin or the exact slice length-bearing pair.
use super::{
    Context, DropGlueSelection, Kind, Operand, Representation, Type, TypeId, Value, parameter,
};
use crate::{
    abi::{layout::Shape, metadata::Metadata},
    body::drops::glue,
    lowir::builder::Builder,
};

#[derive(Clone, Copy)]
pub(super) enum Input {
    Sized,
    Slice,
    Dynamic,
}
impl Input {
    pub(super) fn prepare(
        context: &mut Context<'_>,
        selection: DropGlueSelection,
    ) -> Result<Self, &'static str> {
        let reference = parameter(context.types, selection.signature)?;
        let representation = context.representation(reference)?;
        if context.types.is_sized(selection.element) == Some(true) {
            context.layout(selection.element)?;
            return if matches!(representation, Representation::Scalar(Type::Ptr)) {
                Ok(Self::Sized)
            } else {
                Err("source drop glue requires its selected thin-reference ABI")
            };
        }
        let (input, expected) = match context.types.kind(selection.element) {
            Some(Kind::Slice(element)) => {
                if context.types.is_sized(element) != Some(true) {
                    return Err("slice cleanup requires a completed sized element");
                }
                context.layout(element)?;
                (Self::Slice, Metadata::Length)
            }
            Some(Kind::Dynamic { .. }) => (Self::Dynamic, Metadata::VTable(selection.element)),
            _ => return Err("unsized source drop glue requires metadata-aware cleanup"),
        };
        let metadata = context.layouts.pointer_metadata(
            context.types,
            context.aggregates,
            selection.element,
        )?;
        let reference_layout = context.layout(reference)?;
        wide_reference(expected, metadata, reference_layout, representation)?;
        Ok(input)
    }
    pub(super) fn call(
        self,
        context: &mut Context<'_>,
        builder: &mut Builder,
        selected: TypeId,
        id: glue::Id,
    ) -> Result<(), &'static str> {
        let recipe = context
            .destruction
            .and_then(|p| p.recipes().get(id))
            .ok_or("source cleanup lacks its exposed recipe")?;
        if !matches!(
            (self, recipe.kind),
            (
                Self::Sized,
                glue::Shape::Fields(_) | glue::Shape::Variants(_) | glue::Shape::Array { .. }
            ) | (Self::Slice, glue::Shape::Slice { .. })
                | (Self::Dynamic, glue::Shape::Dynamic)
        ) || (!matches!(self, Self::Sized) && recipe.ty != selected)
        {
            return Err("source cleanup recipe differs from its selected metadata");
        }
        let symbol = context.demand_drop(id)?;
        let signature = context.output.symbols[symbol.0 as usize].signature;
        let input = Value {
            ty: Type::Ptr,
            operand: Operand::Parameter(0),
        };
        match self {
            Self::Sized => {
                builder.call(signature, Type::Void, Operand::Symbol(symbol), &[input])?;
            }
            Self::Slice | Self::Dynamic => {
                let data = builder.load(Type::Ptr, input.operand)?;
                let offset = builder.integer(Type::I64, 8)?;
                let metadata = builder.index(Type::U8, input, offset)?;
                let kind = if matches!(self, Self::Slice) {
                    Type::I64
                } else {
                    Type::Ptr
                };
                let metadata = builder.load(kind, metadata.operand)?;
                builder.call(
                    signature,
                    Type::Void,
                    Operand::Symbol(symbol),
                    &[data, metadata],
                )?;
            }
        }
        Ok(())
    }
}

fn wide_reference(
    expected: Metadata,
    metadata: Metadata,
    layout: crate::abi::layout::Layout,
    representation: Representation,
) -> Result<(), &'static str> {
    if !matches!(expected, Metadata::Length | Metadata::VTable(_))
        || metadata != expected
        || !matches!(layout.shape, Shape::WidePointer)
        || layout.size != 16
        || layout.align != 8
        || !matches!(representation, Representation::Object(span) if span.bytes == 16 && span.align == 8)
    {
        return Err("source slice cleanup requires its exact wide-reference ABI");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/abi_receivers.rs"]
mod tests;
