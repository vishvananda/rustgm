//! Argument witnesses retain one exact typed reached direct or invoked call.
use super::{Frame, Machine, code::Code};
use crate::{
    body::ir::{Action, Rvalue, ValueId},
    sema::{compiler_items::const_payments::ResultProfile, model::Declarations, values::Reader},
    support::call_traits::CallTrait,
    types::{Kind, TypeId, instance::Instance},
};

pub(super) fn is_invoke(body: &Code, position: u32) -> bool {
    let Some(Action::Evaluate(id)) = body.statements.get(position as usize).map(|s| s.action)
    else {
        return false;
    };
    body.values
        .get(id.0 as usize)
        .is_some_and(|value| matches!(value.kind, Rvalue::Invoke { .. }))
}

pub(super) fn checked_call(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    position: u32,
    target: Instance,
    contract: (crate::resolve::model::DefId, ResultProfile),
    charge: &mut impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    charge(16)?;
    let arguments = super::reached_layout::checked_call_arguments(
        facts,
        body,
        frame,
        position,
        target,
        (contract.0, contract.1.ty()),
    )?;
    let arguments = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("paid call has invalid arguments")?;
    check_operands(facts, body, frame, target, arguments, charge)
}

pub(super) fn checked_invoke(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    position: u32,
    target: Instance,
    contract: (crate::resolve::model::DefId, ResultProfile),
    charge: &mut impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    charge(16)?;
    let block = body
        .blocks
        .get(frame.block.0 as usize)
        .ok_or("paid invoke has no block")?;
    if !body.const_context || position < block.start || position >= block.end {
        return Err("paid invoke is outside its checked const block");
    }
    let Some(Action::Evaluate(id)) = body.statements.get(position as usize).map(|s| s.action)
    else {
        return Err("paid invoke requires its evaluated statement");
    };
    let value = body
        .values
        .get(id.0 as usize)
        .ok_or("paid invoke lost its evaluated value")?;
    let Rvalue::Invoke {
        kind,
        signature,
        arguments,
    } = value.kind
    else {
        return Err("paid invoke changed its actual Invoke operation");
    };
    let Some(Kind::FunctionItem {
        item,
        signature: item_signature,
        ..
    }) = facts.types.kind(target.ty())
    else {
        return Err("paid invoke target is not a function item");
    };
    if item.0 != contract.0.0 || item_signature != signature || value.ty != contract.1.ty() {
        return Err("paid invoke changed its target, signature, or result type");
    }
    let Some([receiver, operands @ ..]) = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
    else {
        return Err("paid invoke lost its receiver operand");
    };
    let receiver_ty = body
        .values
        .get(receiver.0 as usize)
        .ok_or("paid invoke receiver is absent")?
        .ty;
    let subject = if kind == CallTrait::FnOnce {
        receiver_ty
    } else {
        let Some(Kind::Reference {
            mutable, referent, ..
        }) = facts.types.kind(receiver_ty)
        else {
            return Err("paid invoke requires its actual borrowed receiver");
        };
        if mutable != (kind == CallTrait::FnMut) {
            return Err("paid invoke changed its receiver borrow capability");
        }
        referent
    };
    if subject != target.ty() || Machine::value(frame, *receiver).is_err() {
        return Err("paid invoke receiver differs from its checked function item");
    }
    check_operands(facts, body, frame, target, operands, charge)
}

fn check_operands(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    target: Instance,
    arguments: &[ValueId],
    charge: &mut impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(target.ty()) else {
        return Err("paid call target is not an item");
    };
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
        return Err("paid call has no signature");
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or("paid call has invalid parameters")?;
    if parameters.len() != arguments.len() {
        return Err("paid call changed its checked arity");
    }
    for (&parameter, &value) in parameters.iter().zip(arguments) {
        charge(4)?;
        check_operand(facts, body, frame, parameter, value)?;
    }
    Ok(())
}

fn check_operand(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    parameter: TypeId,
    value: ValueId,
) -> Result<(), &'static str> {
    if body
        .values
        .get(value.0 as usize)
        .is_none_or(|v| v.ty != parameter)
    {
        return Err("paid call argument differs from its complete signature");
    }
    let value = Machine::value(frame, value).map_err(|_| "paid call argument was not evaluated")?;
    let physical = Reader::new(facts).storage_type(parameter)?;
    match facts.types.kind(physical) {
        Some(Kind::FunctionItem { .. })
            if value.as_function() == Some(Instance::new(&facts.types, physical)?) => {}
        Some(Kind::Primitive(_)) => {
            // Reuse the actual primitive codec's kind and width validity;
            // callback inputs need not equal each other or the result type.
            Reader::new(facts)
                .encode_primitive(physical, value)
                .map_err(|_| "paid call scalar differs from its declared primitive type")?;
        }
        Some(Kind::Adt(..) | Kind::Closure { .. }) => {
            Reader::new(facts)
                .view(value)
                .map_err(|_| "paid call aggregate value is malformed")?;
        }
        _ => return Err("paid call value differs from its declared closed argument profile"),
    }
    Ok(())
}
