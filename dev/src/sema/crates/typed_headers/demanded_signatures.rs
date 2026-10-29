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
    if expected.is_none() && !admitted(rows, raw) {
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
    if !admitted(rows, raw) {
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
            convention: convention(raw).map_err(error)?,
        })
        .map_err(error)
}

#[derive(Clone, Copy)]
pub(super) struct SignaturePort {
    pub owner: DeclarationRef,
    pub kind: s::BinderKind,
    pub raw: g::Signature,
}

/// The same canonical shape check serves eager Graphs and the private memo.
/// Callers authenticate their actual owner/arena and pay their own lookup work.
pub(super) fn require(
    rows: s::ValidatedView<'_>,
    store: &Store,
    expected: SignaturePort,
    actual: TypeId,
    steps: &mut usize,
    mut mapped: impl FnMut(g::TypeRef) -> Result<TypeId, &'static str>,
) -> Result<(), &'static str> {
    let SignaturePort { owner, kind, raw } = expected;
    let frame = rows
        .binders
        .get(raw.binder.0 as usize)
        .ok_or("imported callable signature lacks its actual binder frame")?;
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_,
        convention,
    }) = store.kind(actual)
    else {
        return Err("imported callable signature is not an actual function type");
    };
    if frame.owner != owner
        || frame.kind != kind
        || bound_regions != frame.count
        || unsafe_ != raw.unsafe_
        || raw.variadic
        || !admitted(rows, raw)
        || convention != self::convention(raw)?
        || mapped(raw.result).map_err(|_| "missing actual source result type")? != result
    {
        return Err("imported callable changed its full actual binder/signature contract");
    }
    let expected = raw.parameters.slice(&rows.roots)?;
    let actual = store
        .list(parameters)
        .ok_or("imported callable omitted its actual inputs")?;
    if actual.len() != expected.len() {
        return Err("imported callable signature omitted actual inputs");
    }
    for (&actual, &expected) in actual.iter().zip(expected) {
        *steps = steps.saturating_add(1);
        if actual != mapped(expected).map_err(|_| "missing actual source input type")? {
            return Err("imported callable changed an actual full parameter type");
        }
    }
    Ok(())
}

fn admitted(rows: s::ValidatedView<'_>, raw: g::Signature) -> bool {
    !raw.variadic
        && (raw.convention == g::Convention::Rust
            || (raw.convention == g::Convention::C && rows.descriptor().profile().definitions()))
}
/// Shape conversion only. Callers authenticate profile/owner before interning.
pub(super) fn convention(raw: g::Signature) -> Result<Convention, &'static str> {
    if raw.variadic {
        return Err("source variadic callable requires its wider explicit profile");
    }
    match raw.convention {
        g::Convention::Rust => Ok(Convention::RUST),
        g::Convention::C => Ok(Convention::C),
        g::Convention::RustCall => {
            Err("source rust-call convention requires its wider explicit profile")
        }
    }
}
