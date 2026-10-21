//! Physical reference traversal identifies a leaf without delegating call permission.
use crate::types::{Kind, Store, TypeId};

pub(crate) fn leaf(
    types: &Store,
    mut subject: TypeId,
) -> Result<(Option<TypeId>, usize), &'static str> {
    let mut visits = 1usize;
    subject = types.storage_head(subject)?;
    while let Some(Kind::Reference { referent, .. }) = types.kind(subject) {
        if visits > 1_000_000 {
            return Err("callable reference path exceeds its work budget");
        }
        visits += 1;
        subject = types.storage_head(referent)?;
    }
    let kind = types
        .kind(subject)
        .ok_or("invalid callable reference type")?;
    Ok((
        matches!(kind, Kind::Closure { .. }).then_some(subject),
        visits,
    ))
}
