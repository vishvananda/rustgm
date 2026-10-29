//! Body elision requires the actual source declaration and complete typed direct Call.
use crate::{
    body::ir::{Body, Rvalue, Value},
    resolve::expansion::Report,
    sema::{model::Declarations, values::Reader},
    types::{TypeId, instance::Instance},
};
pub(super) fn require(
    report: &Report,
    facts: &Declarations,
    body: &Body,
    value: &Value,
    ty: TypeId,
) -> Result<(), &'static str> {
    let selected = Reader::new(facts)
        .source_wrapping_offset(Instance::new(&facts.types, ty)?, body.source)?
        .ok_or("source arith_offset lost its dependency descriptor")?;
    facts
        .source_bindings
        .as_deref()
        .ok_or("source arith_offset catalog disappeared")?
        .require_wrapping_offset_report(report, facts, &selected)?;
    let Rvalue::Call {
        signature,
        arguments,
        ..
    } = value.kind
    else {
        return Err("source arith_offset dependency requires its actual Call");
    };
    let Some(&[pointer, count]) = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
    else {
        return Err("source arith_offset requires two direct inputs");
    };
    if signature != selected.signature()
        || value.ty != selected.pointer()
        || [pointer, count]
            .into_iter()
            .zip(selected.inputs())
            .any(|(id, ty)| body.values.get(id.0 as usize).is_none_or(|v| v.ty != ty))
    {
        return Err("source arith_offset dependency differs from its complete actual Call");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../tests/invariants/signed_dependencies.rs"]
mod tests;
