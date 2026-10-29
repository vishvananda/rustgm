//! Cold literal storage is paid before allocation and tied to each actual value type.
#[cfg(test)]
#[path = "../../../tests/invariants/literal_pools.rs"]
mod tests;
use super::ImportedOriginals;
use crate::{
    body::ir::{Body, Rvalue, ScalarId},
    sema::{crates::SourceOriginal, model::Scalar},
    support::crates::generics::source as s,
};

pub(super) fn count(
    raw: &s::Original,
    work: &mut ImportedOriginals,
) -> Result<usize, &'static str> {
    work.charge(raw.values.len())?;
    Ok(raw
        .values
        .iter()
        .filter(|v| matches!(v.kind, s::ValueKind::Scalar(_)))
        .count())
}
pub(super) fn allocate(
    raw: &s::Original,
    body: &mut Body,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    body.scalars = Vec::with_capacity(count(raw, work)?);
    Ok(())
}
fn value(view: &SourceOriginal<'_>, row: s::Value) -> Result<Scalar, &'static str> {
    if !view.rows().descriptor().profile().feature_predicates() {
        return Err("source scalar reconstruction requires its explicit v4 ordinary profile");
    }
    crate::sema::crates::import_scalar_value(row.scalar_value(&view.rows().types)?)
}
pub(super) fn append(
    view: &SourceOriginal<'_>,
    row: s::Value,
    body: &mut Body,
) -> Result<Rvalue, &'static str> {
    let value = value(view, row)?;
    let id =
        ScalarId(u32::try_from(body.scalars.len()).map_err(|_| "source scalar slots overflow")?);
    body.scalars.push(value);
    Ok(Rvalue::Scalar(id))
}
pub(super) fn same(
    view: &SourceOriginal<'_>,
    row: s::Value,
    id: ScalarId,
    body: &Body,
    next: &mut usize,
) -> Result<bool, &'static str> {
    let expected = value(view, row)?;
    let same = id.0 as usize == *next && body.scalars.get(*next) == Some(&expected);
    *next = next.saturating_add(1);
    Ok(same)
}
