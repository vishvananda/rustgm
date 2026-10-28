//! Argument witnesses retain one exact typed reached direct call.
use super::{Frame, Machine, code::Code};
use crate::{
    sema::{model::Declarations, values::Reader},
    types::{Kind, Primitive, instance::Instance},
};
pub(super) fn checked_call(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    position: u32,
    target: Instance,
    contract: (crate::resolve::model::DefId, Primitive),
    charge: &mut impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    charge(16)?;
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
        charge(4)?;
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
            Some(Kind::Primitive(_)) => {
                // Reuse the actual primitive codec's kind and width validity;
                // callback inputs need not equal each other or the result type.
                Reader::new(facts)
                    .encode_primitive(parameter, value)
                    .map_err(|_| "paid call scalar differs from its declared primitive type")?;
            }
            _ => return Err("paid call value differs from its declared closed argument profile"),
        }
    }
    Ok(())
}
