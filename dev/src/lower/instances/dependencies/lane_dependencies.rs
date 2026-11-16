//! An authenticated SIMD intrinsic Call needs no fabricated ordinary function body.
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
        .source_simd(Instance::new(&facts.types, ty)?, body.source)?
        .ok_or("source SIMD dependency lost its actual descriptor")?;
    facts
        .source_bindings
        .as_deref()
        .ok_or("source SIMD catalog disappeared")?
        .require_provider_report(report)?;
    let Rvalue::Call {
        signature,
        arguments,
        ..
    } = value.kind
    else {
        return Err("source SIMD dependency requires its actual Call");
    };
    let inputs = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("invalid source SIMD dependency operands")?;
    if !selected.opened_signature_matches(&facts.types, signature)
        || value.ty != selected.output
        || inputs.len() != selected.inputs().len()
        || inputs
            .iter()
            .zip(selected.inputs())
            .any(|(id, &ty)| body.values.get(id.0 as usize).is_none_or(|v| v.ty != ty))
    {
        return Err("source SIMD dependency differs from its complete checked Call");
    }
    Ok(())
}
