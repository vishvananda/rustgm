//! Reuse prepared public field instances and exact layouts after selected schema proof.
use crate::{
    sema::model::Declarations,
    support::retained::Growth,
    types::{
        ItemId, Kind, TypeId, aggregate,
        unsizing::{CoercionPlan, CoercionSpan, CoercionStep},
    },
};

pub(super) fn build(
    facts: &mut Declarations,
    owner: ItemId,
    source: TypeId,
    target: TypeId,
) -> Result<CoercionPlan, &'static str> {
    facts.charge_caller_inputs(
        1,
        size_of::<CoercionPlan>().saturating_add(size_of::<Vec<CoercionSpan>>()),
    )?;
    for ty in [source, target] {
        if facts.types.contains_parameters(ty) != Some(false) {
            return Err("nominal physical conversion requires a closed actual application");
        }
        facts.aggregates.prepare(&mut facts.types, ty)?;
        facts
            .layouts
            .demand_bounded(&facts.types, &facts.aggregates, ty, 1_000_000)
            .map_err(|error| error.message)?;
    }
    let schema = facts
        .unsizing_rules
        .as_deref()
        .ok_or("nominal physical conversion lacks declaration recipes")?
        .nominal_schema(&facts.types, owner)?
        .ok_or("nominal physical conversion lacks its selected field schema")?;
    let from = facts.aggregates.shape(&facts.types, source)?;
    let to = facts.aggregates.shape(&facts.types, target)?;
    if from.definition.item != to.definition.item
        || from.definition.kind != aggregate::Kind::Struct
        || from.variants().len() != 1
        || to.variants().len() != 1
    {
        return Err("nominal physical conversion changed its actual struct owner");
    }
    let from = from
        .variant(0)
        .ok_or("nominal source field group is missing")?;
    let to = to
        .variant(0)
        .ok_or("nominal target field group is missing")?;
    let from = facts
        .types
        .list(from.fields)
        .ok_or("nominal source fields are missing")?;
    let to = facts
        .types
        .list(to.fields)
        .ok_or("nominal target fields are missing")?;
    if from.len() != schema.fields.len() || to.len() != schema.fields.len() {
        return Err("nominal public field coverage differs from the selected schema");
    }
    let growth = Growth::vector::<CoercionSpan>(0, 0, from.len());
    facts.charge_caller_inputs(growth.work.saturating_add(from.len()), growth.bytes)?;
    let mut fields = Vec::new();
    fields
        .try_reserve(from.len())
        .map_err(|_| "nominal physical field allocation failed")?;
    for (index, (&a, &b)) in from.iter().zip(to).enumerate() {
        fields.push(span(facts, (source, target), schema, index, a, b)?);
    }
    let changed = *fields
        .get(schema.changed as usize)
        .ok_or("nominal changed field is missing")?;
    let step = step(&facts.types, changed.source, changed.target)?;
    Ok(CoercionPlan {
        owner,
        source,
        target,
        changed: schema.changed,
        fields,
        step,
    })
}

fn span(
    facts: &Declarations,
    endpoints: (TypeId, TypeId),
    schema: &crate::types::unsizing::CoercionSchema,
    index: usize,
    source: TypeId,
    target: TypeId,
) -> Result<CoercionSpan, &'static str> {
    let layout = |ty| {
        facts
            .layouts
            .get(ty)
            .ok_or("nominal prepared layout is missing")
    };
    let offsets = |root| {
        facts
            .layouts
            .variants(layout(root)?)
            .first()
            .map(|variant| facts.layouts.offsets(variant))
            .ok_or("nominal public layout field group is missing")
    };
    let a = layout(source)?;
    let b = layout(target)?;
    if schema.fields[index].phantom {
        if facts
            .aggregates
            .phantom_argument(&facts.types, source)?
            .is_none()
            || facts
                .aggregates
                .phantom_argument(&facts.types, target)?
                .is_none()
            || a.size != 0
            || b.size != 0
        {
            return Err("nominal marker conversion requires actual zero-sized PhantomData");
        }
    } else if index != schema.changed as usize && (a.size != b.size || a.align != b.align) {
        return Err("nominal unchanged field has a different physical representation");
    }
    Ok(CoercionSpan {
        source,
        target,
        from: *offsets(endpoints.0)?
            .get(index)
            .ok_or("nominal source field offset is missing")?,
        to: *offsets(endpoints.1)?
            .get(index)
            .ok_or("nominal target field offset is missing")?,
        bytes: a.size,
        align: a.align.min(b.align),
    })
}

fn step(
    types: &crate::types::Store,
    source: TypeId,
    target: TypeId,
) -> Result<CoercionStep, &'static str> {
    match (types.kind(source), types.kind(target)) {
        (Some(Kind::Adt(a, _)), Some(Kind::Adt(b, _))) if a == b => {
            Ok(CoercionStep::Nested { source, target })
        }
        (Some(Kind::Reference { referent: a, .. }), Some(Kind::Reference { referent: b, .. }))
        | (Some(Kind::Pointer { pointee: a, .. }), Some(Kind::Pointer { pointee: b, .. })) => {
            match (types.kind(a), types.kind(b)) {
                (Some(Kind::Array { .. }), Some(Kind::Slice(_))) => Ok(CoercionStep::Slice(a)),
                _ => Err("nominal pointer terminal requires its wider checked conversion profile"),
            }
        }
        _ => Err("nominal changed field lacks its checked physical conversion profile"),
    }
}
