//! Argument witnesses retain one exact typed reached direct call.
use super::{Frame, Machine, code::Code};
use crate::{
    sema::model::{Declarations, Scalar},
    types::{Kind, Primitive, instance::Instance},
};
pub(super) fn checked_call(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    position: u32,
    target: Instance,
    contract: (crate::resolve::model::DefId, Primitive),
) -> Result<(), &'static str> {
    let arguments = super::reached_layout::checked_call_arguments(
        facts,
        body,
        frame,
        position,
        target,
        (contract.0, crate::types::TypeId::primitive(contract.1)),
    )?;
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
    let arguments = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("paid call has invalid arguments")?;
    if parameters.len() != arguments.len() {
        return Err("paid call changed its checked arity");
    }
    for (&parameter, &value) in parameters.iter().zip(arguments) {
        if body
            .values
            .get(value.0 as usize)
            .is_none_or(|v| v.ty != parameter)
        {
            return Err("paid call argument differs from its complete signature");
        }
        let value =
            Machine::value(frame, value).map_err(|_| "paid call argument was not evaluated")?;
        match facts.types.kind(parameter) {
            Some(Kind::FunctionItem { .. })
                if value.as_function() == Some(Instance::new(&facts.types, parameter)?) => {}
            Some(Kind::Primitive(Primitive::U8)) if matches!(value.as_scalar(), Ok(Scalar::Integer(bits)) if bits <= u128::from(u8::MAX)) =>
                {}
            _ => return Err("paid call value differs from its declared closed argument profile"),
        }
    }
    Ok(())
}
