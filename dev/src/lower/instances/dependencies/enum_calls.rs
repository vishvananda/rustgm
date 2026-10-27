//! Only a genuine completed source discriminant Call omits a body demand.
use crate::{
    body::ir::{Body, Rvalue, Value}, resolve::expansion::Report,
    sema::{model::Declarations, values::Reader}, types::{TypeId, instance::Instance},
};

pub(super) fn require(report: &Report, facts: &Declarations, body: &Body, value: &Value, ty: TypeId)
    -> Result<(), &'static str> {
    let selected = Reader::new(facts).source_discriminant(Instance::new(&facts.types, ty)?, body.source)?
        .ok_or("source discriminant dependency lost its actual descriptor")?;
    facts.source_bindings.as_deref().ok_or("source discriminant catalog disappeared")?
        .require_discriminant_report(report, facts, &selected)?;
    let Rvalue::Call { signature, arguments, .. } = value.kind else {
        return Err("source discriminant dependency requires an actual Call");
    };
    let [input] = body.arguments.get(arguments.start as usize..arguments.end as usize)
        .ok_or("source discriminant dependency has invalid arguments")? else {
        return Err("source discriminant requires exactly one reference");
    };
    if signature != selected.signature() || value.ty != selected.output()
        || body.values.get(input.0 as usize).is_none_or(|v| !selected.input_matches(&facts.types, v.ty))
    { return Err("source discriminant dependency changed its complete Call contract"); }
    Ok(())
}
