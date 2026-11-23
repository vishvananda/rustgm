//! Source and bring-up atomic loads and stores share the target word operations.
use super::{Lower, Representation, Selected, ir};
use crate::{
    lowir::{
        atomic::{LoadOrder, StoreOrder},
        program::{Operand, Value},
        vocabulary::Type,
    },
    types::TypeId,
};

#[derive(Clone, Copy)]
pub(super) enum Access {
    Load(LoadOrder),
    Store(StoreOrder, ir::ValueId),
}

/// The accessed word: its type, its address and the integer that carries it.
#[derive(Clone, Copy)]
pub(super) struct Word {
    pub ty: TypeId,
    pub address: Operand,
    pub carrier: Type,
    pointer: bool,
}

impl Lower<'_, '_, '_> {
    pub(super) fn source_atomic_access(
        &mut self,
        name: &str,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Option<Selected>, &'static str> {
        let Some(selection) = self
            .context
            .constants
            .source_atomic_access(name, callee, signature)?
        else {
            return Ok(None);
        };
        let inputs = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("missing source atomic access arguments")?;
        let (pointer, access) = match (selection.stored, inputs) {
            (None, &[pointer]) => (
                pointer,
                Access::Load(
                    LoadOrder::from_rust(selection.ordering)
                        .ok_or("invalid source atomic load order")?,
                ),
            ),
            (Some(ty), &[pointer, value]) if self.input_type(value)? == ty => (
                pointer,
                Access::Store(
                    StoreOrder::from_rust(selection.ordering)
                        .ok_or("invalid source atomic store order")?,
                    value,
                ),
            ),
            _ => return Err("source atomic access differs from its evaluated inputs"),
        };
        if self.input_type(pointer)? != selection.pointer || result != selection.result {
            return Err("source atomic access differs from its evaluated Call");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let word = self
            .atomic_word(pointer)?
            .ok_or("source atomic access has no target word representation")?;
        self.atomic_access(word, access)?
            .map(Some)
            .ok_or("source atomic access has no target word representation")
    }

    pub(super) fn atomic_access(
        &mut self,
        word: Word,
        access: Access,
    ) -> Result<Option<Selected>, &'static str> {
        match access {
            Access::Load(order) => {
                let value = self
                    .builder
                    .atomic_load(word.carrier, word.address, order)?;
                self.atomic_value(word, value)
                    .map(|value| Some(Selected::Runtime(value)))
            }
            Access::Store(order, operand) => {
                let Some(value) = self.atomic_operand(word, operand)? else {
                    return Ok(None);
                };
                self.builder.atomic_store(value, word.address, order)?;
                Ok(Some(Selected::Unit))
            }
        }
    }

    /// The word behind a thin pointer to an integer or pointer the backend's
    /// atomic operations cover.
    pub(super) fn atomic_word(
        &mut self,
        pointer: ir::ValueId,
    ) -> Result<Option<Word>, &'static str> {
        let Some((address, ty)) = self.thin_input(pointer)? else {
            return Ok(None);
        };
        let (carrier, pointer) = match self.context.representation(ty)? {
            Representation::Scalar(Type::Ptr) => (Type::I64, true),
            Representation::Scalar(
                scalar @ (Type::I8
                | Type::U8
                | Type::I16
                | Type::U16
                | Type::I32
                | Type::U32
                | Type::I64),
            ) => (scalar, false),
            _ => return Ok(None),
        };
        Ok(Some(Word {
            ty,
            address: address.operand,
            carrier,
            pointer,
        }))
    }

    /// An operand in the word's carrier: the word's own type, or the machine
    /// word that updates a pointer.
    pub(super) fn atomic_operand(
        &mut self,
        word: Word,
        operand: ir::ValueId,
    ) -> Result<Option<Value>, &'static str> {
        let Selected::Runtime(mut value) = self.value(operand)? else {
            return Ok(None);
        };
        if value.ty == Type::Ptr {
            value = self.builder.copy(Type::I64, value)?;
        }
        Ok((value.ty == word.carrier).then_some(value))
    }

    /// A carried word as a value of the word's own type.
    pub(super) fn atomic_value(&mut self, word: Word, value: Value) -> Result<Value, &'static str> {
        if word.pointer {
            self.builder.copy(Type::Ptr, value)
        } else {
            Ok(value)
        }
    }
}
