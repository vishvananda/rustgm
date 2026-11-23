//! An owning box is returned after its existing allocation receives the moved value.
use super::{Address, Lower, Selected, ir};
use crate::{
    lowir::vocabulary::Type,
    sema::{compiler_items::boxed_write, model::Declarations},
    types::{TypeId, instance::Instance},
};

pub(in crate::lower) struct Call {
    callee: ir::ValueId,
    inputs: [ir::ValueId; 2],
    selected: boxed_write::Selection,
}

pub(in crate::lower) fn checked(
    facts: &Declarations,
    body: &ir::Body,
    value: &ir::Value,
    target: TypeId,
) -> Result<Call, &'static str> {
    let ir::Rvalue::Call {
        signature,
        callee,
        arguments,
    } = value.kind
    else {
        return Err("source box write requires its checked direct Call");
    };
    let selected = boxed_write::checked_type(facts, target, signature)?;
    let Some(&[boxed, stored]) = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
    else {
        return Err("source box write requires its two evaluated inputs");
    };
    let inputs = [boxed, stored];
    if body.const_context
        || body.source.source().is_none()
        || value.ty != selected.boxed
        || inputs
            .iter()
            .zip([selected.boxed, selected.value])
            .any(|(id, ty)| {
                body.values
                    .get(id.0 as usize)
                    .is_none_or(|input| input.ty != ty)
            })
    {
        return Err("source box write differs from its actual evaluated Call");
    }
    Ok(Call {
        callee,
        inputs,
        selected,
    })
}

impl Lower<'_, '_, '_> {
    pub(super) fn source_boxed_write(
        &mut self,
        id: ir::ValueId,
        target: TypeId,
    ) -> Result<Selected, &'static str> {
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("reached source box write call disappeared")?;
        let call = checked(self.context.ledger, self.body, value, target)?;
        let instance = self
            .body
            .callable_target(Instance::new(self.context.types, target)?);
        if !matches!(self.value(call.callee)?, Selected::Item(actual) if actual == instance) {
            return Err("source box write callee differs from its checked instance");
        }
        let selected = call.selected;
        let content = self.context.layout(selected.storage)?;
        let stored = self.context.layout(selected.value)?;
        if (content.size, content.align) != (stored.size, stored.align) {
            return Err("source box write storage differs from its value layout");
        }
        let offset = self.context.box_pointer_offset(selected.boxed)?;
        let [boxed, value] = call.inputs;
        let boxed = self.value(boxed)?;
        let pointer = match boxed {
            Selected::Object(address) => {
                let field = self.offset(address, offset)?;
                self.builder.load(Type::Ptr, field.operand)?
            }
            Selected::Runtime(pointer) if pointer.ty == Type::Ptr && offset == 0 => pointer,
            _ => return Err("source box write lost its actual owning pointer"),
        };
        let destination = Address {
            operand: pointer.operand,
            align: content.align,
        };
        self.write(Some(destination), self.value(value)?, selected.value)?;
        match boxed {
            Selected::Object(source) => self.loaded(source, selected.boxed),
            boxed => Ok(boxed),
        }
    }
}
