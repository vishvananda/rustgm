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
        .source_subtraction(Instance::new(&facts.types, ty)?, body.source)?
        .ok_or("source subtraction dependency lost its actual descriptor")?;
    facts
        .source_bindings
        .as_deref()
        .ok_or("source subtraction catalog disappeared")?
        .require_subtraction_report(report, facts, &selected)?;
    let Rvalue::Call {
        signature,
        arguments,
        ..
    } = value.kind
    else {
        return Err("source subtraction dependency requires its actual Call");
    };
    let [left, right] = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("invalid source subtraction dependency operands")?
    else {
        return Err("source subtraction requires exactly two evaluated operands");
    };
    if !selected.opened_signature_matches(&facts.types, signature)
        || value.ty != selected.output
        || [left, right].into_iter().any(|id| {
            body.values
                .get(id.0 as usize)
                .is_none_or(|v| v.ty != selected.input)
        })
    {
        return Err("source subtraction dependency differs from its complete checked Call");
    }
    Ok(())
}
