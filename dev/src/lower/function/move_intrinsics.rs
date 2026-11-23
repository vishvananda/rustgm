//! Pinned pointer reads and moves preserve the body's existing storage and destruction proof.
use super::{Address, Lower, Representation, Selected, ir};
use crate::{
    lowir::vocabulary::Type,
    types::{Kind, TypeId, arguments::Argument, callable::Convention},
};

impl Lower<'_, '_, '_> {
    pub(super) fn source_memory_move(
        &mut self,
        name: &str,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let types = self.context.types;
        let Some(Kind::FunctionItem {
            arguments: substitutions,
            ..
        }) = types.kind(callee)
        else {
            return Err("source memory move requires its complete function item");
        };
        let mut substitutions = types
            .arguments(substitutions)
            .ok_or("source memory move lost its type substitution")?;
        let (Some(Argument::Type(element)), None) = (substitutions.next(), substitutions.next())
        else {
            return Err("source memory move requires exactly one type substitution");
        };
        let inputs = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source memory move lost its actual inputs")?;
        if name == "forget" {
            return self.forget_value(signature, inputs, element, result);
        }
        let (pointer, stored) = match (name, inputs) {
            ("read_via_copy", &[pointer]) if element == result => (pointer, None),
            ("write_via_move", &[pointer, stored])
                if result == TypeId::UNIT && self.input_type(stored)? == element =>
            {
                (pointer, Some(stored))
            }
            _ => return Err("source memory move requires its exact evaluated inputs and result"),
        };
        let pointer_type = self.input_type(pointer)?;
        if !matches!(types.kind(pointer_type), Some(Kind::Pointer { mutable, pointee })
            if mutable == stored.is_some() && pointee == element)
        {
            return Err("source memory move requires the corresponding raw pointer");
        }
        let Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result: output,
            unsafe_: true,
            convention: Convention::RUST,
        }) = types.kind(signature)
        else {
            return Err("source memory move requires its opened unsafe Rust signature");
        };
        let expected = [pointer_type, element];
        let count = 1 + usize::from(stored.is_some());
        if output != result || types.list(parameters) != Some(&expected[..count]) {
            return Err("source memory move differs from its opened signature");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let pointer = self.value(pointer)?.runtime()?;
        if pointer.ty != Type::Ptr {
            return Err("source memory move requires a native thin pointer");
        }
        let address = Address {
            operand: pointer.operand,
            align: self.context.layout(element)?.align,
        };
        if let Some(value) = stored {
            self.write(Some(address), self.value(value)?, element)?;
            Ok(Selected::Unit)
        } else {
            self.loaded(address, element)
        }
    }

    /// Consumption of the evaluated argument is already proved by the normal Call.
    fn forget_value(
        &self,
        signature: TypeId,
        inputs: &[ir::ValueId],
        element: TypeId,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let types = self.context.types;
        let &[input] = inputs else {
            return Err("source forget requires one evaluated value");
        };
        if result != TypeId::UNIT
            || self.input_type(input)? != element
            || !matches!(
                types.kind(signature),
                Some(Kind::Function {
                    bound_regions: 0,
                    parameters,
                    result: TypeId::UNIT,
                    unsafe_: false,
                    convention: Convention::RUST,
                }) if types.list(parameters) == Some(&[element])
            )
        {
            return Err("source forget differs from its opened signature and exact value");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        self.value(input)?;
        Ok(Selected::Unit)
    }

    /// A fresh snapshot of the value of type `ty` stored at `source`.
    pub(super) fn loaded(&mut self, source: Address, ty: TypeId) -> Result<Selected, &'static str> {
        match self.context.representation(ty)? {
            Representation::Scalar(Type::Void) => Ok(Selected::Unit),
            Representation::Scalar(scalar) => self
                .builder
                .load(scalar, source.operand)
                .map(Selected::Runtime),
            Representation::Object(_) | Representation::ScalarObject(_) => {
                let destination = self
                    .allocate(ty)?
                    .ok_or("loaded object value has no snapshot storage")?;
                self.write(Some(destination), Selected::Object(source), ty)?;
                Ok(Selected::Object(destination))
            }
        }
    }
}
