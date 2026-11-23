//! A reached raw equality call compares exactly the checked referent's bytes.
use super::{Lower, Selected, ir};
use crate::{
    lowir::program::Operand,
    sema::{compiler_items::bytewise, model::Declarations},
    types::{Primitive, TypeId, instance::Instance},
};

pub(in crate::lower) struct Call {
    callee: ir::ValueId,
    element: TypeId,
    signature: TypeId,
    arguments: ir::ValueList,
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
        return Err("source raw equality requires its checked direct Call");
    };
    if body.const_context
        || body.source.source().is_none()
        || value.ty != TypeId::primitive(Primitive::Bool)
    {
        return Err("source raw equality requires its actual runtime Call");
    }
    let (element, expected) = bytewise::checked_type(facts, target, signature)?;
    let inputs = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("source raw equality lost its evaluated inputs")?;
    if inputs.len() != expected.len()
        || inputs.iter().zip(expected).any(|(id, ty)| {
            body.values
                .get(id.0 as usize)
                .is_none_or(|input| input.ty != ty)
        })
    {
        return Err("source raw equality differs from its evaluated inputs");
    }
    Ok(Call {
        callee,
        element,
        signature,
        arguments,
    })
}

impl Lower<'_, '_, '_> {
    pub(super) fn source_raw_equality(
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
            .ok_or("reached source raw equality call disappeared")?;
        let call = checked(self.context.ledger, self.body, value, target)?;
        let instance = self
            .body
            .callable_target(Instance::new(self.context.types, target)?);
        if !matches!(self.value(call.callee)?, Selected::Item(actual) if actual == instance) {
            return Err("source raw equality callee differs from its checked instance");
        }
        let symbol = self
            .context
            .demand_source_raw_eq(instance, call.signature, call.element)?;
        self.call_operand(
            call.signature,
            Operand::Symbol(symbol),
            call.arguments,
            TypeId::primitive(Primitive::Bool),
        )
    }
}
