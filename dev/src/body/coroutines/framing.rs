//! Actual physical frame coordinates supply data, never a generated-family permit.
use crate::{
    body::ir::{Body, LocalId, LocalKind, Owner, Projection, ProjectionId, ProjectionKind},
    types::{Kind, Store},
};

pub(crate) fn field(
    types: &Store,
    body: &Body,
    projection: &Projection,
) -> Result<u32, &'static str> {
    let Owner::FutureEntry { key, .. } = body.owner else {
        return Err("future field requires its actual generated family owner");
    };
    let ProjectionKind::FutureField(index) = projection.kind else {
        return Err("future field query requires its typed coordinate");
    };
    let Some(Kind::Future {
        identity,
        arguments,
    }) = types.kind(key.future)
    else {
        return Err("future field owner lost its genuine complete application");
    };
    if body.place_type(projection.base) != Some(key.future)
        || types.contains_parameters(key.future) != Some(false)
        || types.contains_projections(key.future) != Some(false)
    {
        return Err("future field changed its complete closed frame identity");
    }
    let storage = types.future_application(identity, arguments)?.storage;
    if types
        .list(storage)
        .and_then(|fields| fields.get(index as usize))
        .copied()
        != Some(projection.ty)
    {
        return Err("future field differs from its actual storage index/type");
    }
    Ok(index)
}

pub(crate) fn transfer(
    types: &Store,
    body: &Body,
    source: ProjectionId,
    target: LocalId,
) -> Result<Projection, &'static str> {
    let projection = *body
        .projections
        .get(source.0 as usize)
        .ok_or("future transfer lost its actual source projection")?;
    if field(types, body, &projection)? == 0 {
        return Err("future transfer cannot consume the state discriminant");
    }
    let local = body
        .locals
        .get(target.0 as usize)
        .ok_or("future transfer lost its actual owned target")?;
    if local.kind != LocalKind::Binding || local.ty != projection.ty {
        return Err("future transfer changes its actual owned target kind/type");
    }
    Ok(projection)
}
