//! Symbolic storage equality is independent of source call or value validity proof.
use super::{layout::Layouts, metadata::Metadata, templates::Templates};
use crate::types::{Kind, Store, TypeId, aggregate::Definitions};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StorageRelation {
    Equal,
    NeedConcreteLayout,
}
fn charge(work: &mut usize, count: usize, limit: usize) -> Result<(), &'static str> {
    *work = work.saturating_add(count);
    if *work > limit {
        return Err("storage relation work budget exceeded");
    }
    Ok(())
}
/// Actual transparent declarations and pointer metadata supply storage structure.
/// No logical type, const value, lifetime or dereference permission is inferred.
pub(crate) fn compare(
    types: &mut Store,
    definitions: &mut Definitions,
    layouts: &mut Layouts,
    pair: [TypeId; 2],
    work: &mut usize,
    limit: usize,
) -> Result<StorageRelation, &'static str> {
    charge(work, 1, limit)?;
    if types.kind(pair[0]).is_none() || types.kind(pair[1]).is_none() {
        return Err("storage relation requires actual canonical types");
    }
    if pair[0] == pair[1] {
        return Ok(StorageRelation::Equal);
    }
    let mut templates = Templates::default();
    let a = head(
        types,
        definitions,
        layouts,
        &mut templates,
        pair[0],
        work,
        limit,
    )?;
    let b = head(
        types,
        definitions,
        layouts,
        &mut templates,
        pair[1],
        work,
        limit,
    )?;
    if a == b {
        return Ok(StorageRelation::Equal);
    }
    let pointee = |ty| match types.kind(ty) {
        Some(
            Kind::Pointer { pointee, .. }
            | Kind::Reference {
                referent: pointee, ..
            },
        ) => Some(pointee),
        _ => None,
    };
    let (Some(a), Some(b)) = (pointee(a), pointee(b)) else {
        return Ok(StorageRelation::NeedConcreteLayout);
    };
    let a = metadata(types, definitions, layouts, a, work, limit)?;
    let b = metadata(types, definitions, layouts, b, work, limit)?;
    let known_width = |metadata| match metadata {
        Metadata::Thin => Some(8),
        Metadata::Length | Metadata::VTable(_) => Some(16),
        Metadata::Unknown(_) => None,
    };
    Ok(
        if a == b || known_width(a).is_some_and(|width| Some(width) == known_width(b)) {
            StorageRelation::Equal
        } else {
            StorageRelation::NeedConcreteLayout
        },
    )
}
fn head(
    types: &mut Store,
    definitions: &mut Definitions,
    layouts: &mut Layouts,
    templates: &mut Templates,
    mut ty: TypeId,
    work: &mut usize,
    limit: usize,
) -> Result<TypeId, &'static str> {
    loop {
        charge(work, 1, limit)?;
        match types.kind(ty).ok_or("missing storage relation type")? {
            Kind::Pattern { base, .. } => ty = base,
            Kind::Opaque(..) => ty = types.storage_head(ty)?,
            Kind::Adt(..) => {
                let before = (
                    templates.steps,
                    definitions.instantiation_steps(),
                    layouts.steps,
                );
                let field = templates.storage_field(types, definitions, layouts, ty);
                charge(
                    work,
                    templates
                        .steps
                        .saturating_sub(before.0)
                        .saturating_add(definitions.instantiation_steps().saturating_sub(before.1))
                        .saturating_add(layouts.steps.saturating_sub(before.2)),
                    limit,
                )?;
                if let Some(field) = field? {
                    ty = field;
                } else {
                    return Ok(ty);
                }
            }
            _ => return Ok(ty),
        }
    }
}
fn metadata(
    types: &mut Store,
    definitions: &mut Definitions,
    layouts: &mut Layouts,
    ty: TypeId,
    work: &mut usize,
    limit: usize,
) -> Result<Metadata, &'static str> {
    charge(work, 1, limit)?;
    if matches!(types.kind(ty), Some(Kind::Adt(..) | Kind::Tuple(_)))
        && types.is_sized(ty) != Some(true)
    {
        let before = definitions.instantiation_steps();
        let prepared = definitions.prepare_metadata(types, ty);
        charge(
            work,
            definitions.instantiation_steps().saturating_sub(before),
            limit,
        )?;
        prepared?;
    }
    let before = layouts.metadata_steps();
    let answer = layouts.pointer_metadata(types, definitions, ty);
    charge(work, layouts.metadata_steps().saturating_sub(before), limit)?;
    answer
}

#[cfg(test)]
#[path = "../../tests/invariants/nonnull_types.rs"]
mod tests;
