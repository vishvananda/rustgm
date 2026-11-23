//! Typed loads, stores and value moves through the ordinary snapshot paths.
use super::{
    super::{Address, Lower, Representation, Selected},
    Call,
};
use crate::{
    body::ir,
    lowir::{program::Value, vocabulary::Type},
    types::TypeId,
};

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_memory(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        match (call.name, call.inputs) {
            ("read_via_copy", &[pointer]) => {
                let Some((pointer, target)) = self.thin_input(pointer)? else {
                    return Ok(None);
                };
                if target != call.result {
                    return Ok(None);
                }
                let source = Address {
                    operand: pointer.operand,
                    align: self.context.layout(target)?.align,
                };
                self.loaded(source, target).map(Some)
            }
            ("write_via_move", &[pointer, value]) => {
                let Some((pointer, target)) = self.thin_input(pointer)? else {
                    return Ok(None);
                };
                if target != self.input_type(value)? || call.result != TypeId::UNIT {
                    return Ok(None);
                }
                let selected = self.value(value)?;
                if matches!(selected, Selected::Runtime(_) | Selected::Object(_)) {
                    let destination = Address {
                        operand: pointer.operand,
                        align: self.context.layout(target)?.align,
                    };
                    self.write(Some(destination), selected, target)?;
                }
                Ok(Some(Selected::Unit))
            }
            ("write_box_via_move", &[boxed, value]) => self.write_box(call, boxed, value),
            // The value was moved into the call; nothing drops it.
            ("forget", &[_]) => Ok((call.result == TypeId::UNIT).then_some(Selected::Unit)),
            ("volatile_load", &[pointer]) => {
                let Some((pointer, scalar)) = self.volatile_input(pointer, call.result)? else {
                    return Ok(None);
                };
                self.builder
                    .volatile_load(scalar, pointer.operand)
                    .map(|value| Some(Selected::Runtime(value)))
            }
            ("volatile_store", &[pointer, value]) => {
                let stored = self.input_type(value)?;
                let Some((pointer, scalar)) = self.volatile_input(pointer, stored)? else {
                    return Ok(None);
                };
                let value = self.value(value)?.runtime()?;
                if value.ty != scalar || call.result != TypeId::UNIT {
                    return Ok(None);
                }
                self.builder.volatile_store(value, pointer.operand)?;
                Ok(Some(Selected::Unit))
            }
            _ => Ok(None),
        }
    }
    /// The thin pointer to `target` and the scalar one volatile access moves.
    /// An aggregate or zero-sized target keeps the trap.
    fn volatile_input(
        &mut self,
        pointer: ir::ValueId,
        target: TypeId,
    ) -> Result<Option<(Value, Type)>, &'static str> {
        let Some((pointer, referent)) = self.thin_input(pointer)? else {
            return Ok(None);
        };
        let Representation::Scalar(scalar) = self.context.representation(target)? else {
            return Ok(None);
        };
        let whole = self.context.layout(target)?.size == u64::from(scalar.bits() / 8);
        Ok((referent == target && scalar != Type::Void && whole).then_some((pointer, scalar)))
    }
    /// `write_box_via_move(b, x)`: move `x` into the content of the box `b`
    /// of `MaybeUninit<X>`, through the pointer its leading fields hold, and
    /// return `b`.
    fn write_box(
        &mut self,
        call: Call<'_>,
        boxed: ir::ValueId,
        value: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let ty = self.input_type(boxed)?;
        let stored = self.input_type(value)?;
        let content = crate::sema::compiler_items::boxes::content(
            self.context.types,
            self.context.aggregates,
            ty,
        )?;
        let uninit = content.and_then(|content| match self.context.types.kind(content) {
            Some(crate::types::Kind::Adt(_, arguments)) => self
                .context
                .types
                .arguments(arguments)
                .and_then(|mut arguments| arguments.next()),
            _ => None,
        });
        if ty != call.result || uninit != Some(crate::types::arguments::Argument::Type(stored)) {
            return Ok(None);
        }
        let offset = self.context.box_pointer_offset(ty)?;
        let selected = self.value(boxed)?;
        let pointer = match selected {
            Selected::Object(address) => {
                let field = self.offset(address, offset)?;
                self.builder.load(Type::Ptr, field.operand)?
            }
            Selected::Runtime(pointer) if pointer.ty == Type::Ptr && offset == 0 => pointer,
            _ => return Ok(None),
        };
        let moved = self.value(value)?;
        let destination = Address {
            operand: pointer.operand,
            align: self.context.layout(stored)?.align,
        };
        self.write(Some(destination), moved, stored)?;
        match selected {
            Selected::Object(source) => self.loaded(source, ty).map(Some),
            selected => Ok(Some(selected)),
        }
    }
    /// A fresh snapshot of the value of type `ty` stored at `source`.
    fn loaded(&mut self, source: Address, ty: TypeId) -> Result<Selected, &'static str> {
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
