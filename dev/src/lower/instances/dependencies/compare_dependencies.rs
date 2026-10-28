//! An authenticated scalar comparison Call needs no fabricated ordinary function body.
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
        .source_three_way(Instance::new(&facts.types, ty)?, body.source)?
        .ok_or("three_way_compare dependency lost its actual descriptor")?;
    facts
        .source_bindings
        .as_deref()
        .ok_or("three_way_compare catalog disappeared")?
        .require_three_way_report(report, facts, &selected)?;
    if selected.pending_tag(facts)?.is_some() {
        return Err("three_way_compare actual Ordering tags are pending");
    }
    let Rvalue::Call {
        signature,
        arguments,
        ..
    } = value.kind
    else {
        return Err("three_way_compare dependency requires its actual Call");
    };
    let [left, right] = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("invalid three_way_compare dependency operands")?
    else {
        return Err("three_way_compare requires exactly two evaluated operands");
    };
    if signature != selected.signature
        || value.ty != selected.output
        || [left, right].into_iter().any(|id| {
            body.values
                .get(id.0 as usize)
                .is_none_or(|v| v.ty != selected.input)
        })
    {
        return Err("three_way_compare dependency differs from its complete checked Call");
    }
    Ok(())
}
