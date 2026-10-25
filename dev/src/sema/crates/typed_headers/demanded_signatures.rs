//! Genuine signature shapes share validation/interning without inventing headers.
use super::{Error, error};
use crate::{
    support::crates::{
        generics::{self as g, source as s},
        owner_catalog::DeclarationRef,
    },
    types::{Kind, Store, TypeId, callable::Convention},
};

pub(super) fn check(
    rows: s::ValidatedView<'_>,
    expected: Option<(DeclarationRef, s::BinderKind)>,
    raw: g::Signature,
) -> Result<u32, Error> {
    if expected.is_none() && (raw.convention != g::Convention::Rust || raw.variadic) {
        return Err(error(
            "source callable convention/variadic form needs a wider profile",
        ));
    }
    let frame = rows.binders.get(raw.binder.0 as usize).ok_or_else(|| {
        error(if expected.is_some() {
            "source declaration signature lost its actual binder"
        } else {
            "source callable has no exact actual binder"
        })
    })?;
    if expected.is_some_and(|(owner, kind)| frame.owner != owner || frame.kind != kind) {
        return Err(error(
            "source signature changed its exact declaration/binder kind",
        ));
    }
    if raw.convention != g::Convention::Rust || raw.variadic {
        return Err(error(
            "source signature convention requires its wider explicit profile",
        ));
    }
    Ok(frame.count)
}
pub(super) fn intern(
    store: &mut Store,
    count: u32,
    raw: g::Signature,
    parameters: &[TypeId],
    result: TypeId,
) -> Result<TypeId, Error> {
    let parameters = store.intern_list(parameters).map_err(error)?;
    store
        .intern(Kind::Function {
            bound_regions: count,
            parameters,
            result,
            unsafe_: raw.unsafe_,
            convention: Convention::RUST,
        })
        .map_err(error)
}
