//! Only a checked direct Call can use the non-overlapping byte exchange.
use super::{Lower, Selected, ir};
use crate::{
    lowir::program::Operand,
    sema::{compiler_items::swap_contract, model::Declarations},
    types::{TypeId, instance::Instance},
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
        return Err("source typed swap requires its checked direct Call");
    };
    let (element, pointer) = swap_contract::checked_type(facts, target, signature)?;
    let inputs = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("source typed swap lost its evaluated inputs")?;
    if body.const_context
        || body.source.source().is_none()
        || value.ty != TypeId::UNIT
        || inputs.len() != 2
        || inputs.iter().any(|id| {
            body.values
                .get(id.0 as usize)
                .is_none_or(|input| input.ty != pointer)
        })
    {
        return Err("source typed swap differs from its evaluated runtime Call");
    }
    Ok(Call {
        callee,
        element,
        signature,
        arguments,
    })
}

impl Lower<'_, '_, '_> {
    pub(super) fn source_typed_swap(
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
            .ok_or("reached source typed swap call disappeared")?;
        let call = checked(self.context.ledger, self.body, value, target)?;
        let instance = self
            .body
            .callable_target(Instance::new(self.context.types, target)?);
        if !matches!(self.value(call.callee)?, Selected::Item(actual) if actual == instance) {
            return Err("source typed swap callee differs from its checked instance");
        }
        let symbol = self
            .context
            .demand_typed_swap(instance, call.signature, call.element)?;
        self.call_operand(
            call.signature,
            Operand::Symbol(symbol),
            call.arguments,
            TypeId::UNIT,
        )
    }
}
