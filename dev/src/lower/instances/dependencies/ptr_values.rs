//! Only the actual source intrinsic Call omits an ordinary callable-body demand.
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
        .source_ptr_metadata(Instance::new(&facts.types, ty)?, body.source)?
        .ok_or("source ptr_metadata lost its dependency descriptor")?;
    facts
        .source_bindings
        .as_deref()
        .ok_or("source ptr_metadata catalog disappeared")?
        .require_ptr_metadata_report(report, facts, &selected)?;
    let Rvalue::Call {
        signature,
        arguments,
        ..
    } = value.kind
    else {
        return Err("source ptr_metadata dependency requires its actual Call");
    };
    let [input] = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("source ptr_metadata argument range disappeared")?
    else {
        return Err("source ptr_metadata requires one direct pointer argument");
    };
    if signature != selected.signature()
        || value.ty != selected.output()
        || body
            .values
            .get(input.0 as usize)
            .is_none_or(|v| Some(v.ty) != selected.input(facts).ok())
    {
        return Err("source ptr_metadata dependency differs from its complete direct Call");
    }
    Ok(())
}

pub(super) fn require_parts(
    report: &Report,
    facts: &Declarations,
    body: &Body,
    value: &Value,
    ty: TypeId,
) -> Result<(), &'static str> {
    let selected = Reader::new(facts)
        .source_raw_parts(Instance::new(&facts.types, ty)?, body.source)?
        .ok_or("source aggregate_raw_ptr lost its dependency descriptor")?;
    facts
        .source_bindings
        .as_deref()
        .ok_or("source aggregate_raw_ptr catalog disappeared")?
        .require_raw_parts_report(report, facts, &selected)?;
    let Rvalue::Call {
        signature,
        arguments,
        ..
    } = value.kind
    else {
        return Err("source aggregate_raw_ptr dependency requires its actual Call");
    };
    let inputs = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("source aggregate_raw_ptr lost its argument range")?;
    let expected = selected.inputs(facts)?;
    if signature != selected.signature()
        || value.ty != selected.output()
        || inputs.len() != 2
        || inputs
            .iter()
            .zip(expected)
            .any(|(id, ty)| body.values.get(id.0 as usize).is_none_or(|v| v.ty != ty))
    {
        return Err("source aggregate_raw_ptr dependency differs from its complete direct Call");
    }
    Ok(())
}

pub(super) fn require_operation(
    report: &Report,
    facts: &Declarations,
    body: &Body,
    value: &Value,
    ty: TypeId,
    kind: crate::support::bindings::SourceIntrinsic,
) -> Result<(), &'static str> {
    use crate::support::bindings::SourceIntrinsic;
    match kind {
        SourceIntrinsic::PtrMetadata => require(report, facts, body, value, ty),
        SourceIntrinsic::AggregateRawPtr => require_parts(report, facts, body, value, ty),
        _ => Err("pointer dependency operation has an unrelated source intrinsic kind"),
    }
}
