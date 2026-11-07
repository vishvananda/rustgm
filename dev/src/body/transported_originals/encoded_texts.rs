//! Version-thirteen text literals are copied out of their sealed view and
//! interned into the consumer's arena once that view is released.
use super::ImportedOriginals;
use crate::{
    body::ir::{Body, Rvalue},
    sema::{
        crates::SourceOriginal,
        model::Declarations,
        text::{Id, Texts},
    },
    support::crates::generics::source as s,
};

/// A rebuilt text value before its literal is interned. No arena issues this
/// identity, so a body that escapes before `intern` fails at its first read.
const PENDING: Id = Id(u32::MAX);

/// An original's literals in value order, owned past the end of its view.
#[derive(Default)]
pub(super) struct Pending(Vec<Box<str>>);

/// Copy every literal, paying one step and one byte per literal byte.
pub(super) fn copy(
    view: &SourceOriginal<'_>,
    work: &mut ImportedOriginals,
) -> Result<Pending, &'static str> {
    let Some(texts) = view.raw().texts.as_deref() else {
        return Ok(Pending::default());
    };
    work.charge(texts.rows.len())?;
    let mut copied = Vec::with_capacity(texts.rows.len());
    for text in &texts.rows {
        work.charge(text.len())?;
        copied.push(text.clone());
    }
    Ok(Pending(copied))
}
pub(super) fn append(view: &SourceOriginal<'_>) -> Result<Rvalue, &'static str> {
    if !view.rows().descriptor().profile().body_nodes() {
        return Err("source text reconstruction requires the version-thirteen body IR profile");
    }
    Ok(Rvalue::Text(PENDING))
}
/// Replace each pending text value, in value order, by its interned literal.
pub(super) fn intern(
    facts: &mut Declarations,
    body: &mut Body,
    pending: Pending,
) -> Result<(), &'static str> {
    let mut texts = pending.0.into_iter();
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(body.values.len());
    for value in &mut body.values {
        if matches!(value.kind, Rvalue::Text(id) if id == PENDING) {
            let text = texts
                .next()
                .ok_or("source text value has no copied literal")?;
            value.kind = Rvalue::Text(facts.texts.intern(text.as_bytes())?);
        }
    }
    if texts.next().is_some() {
        return Err("source text column outlives its text values");
    }
    Ok(())
}
/// A cached body keeps each literal at its ordinal, in value order.
pub(super) fn validate(
    texts: &Texts,
    view: &SourceOriginal<'_>,
    body: &Body,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let record = view.raw();
    let rows = record.texts.as_deref().map_or(&[][..], |texts| &texts.rows);
    let mut next = 0_usize;
    for (actual, record) in body.values.iter().zip(&record.values) {
        let (Rvalue::Text(id), s::ValueKind::Text(ordinal)) = (actual.kind, record.kind) else {
            continue;
        };
        let expected = rows
            .get(next)
            .filter(|_| ordinal as usize == next)
            .ok_or("source original changed an actual text ordinal")?;
        *steps = steps.saturating_add(1).saturating_add(expected.len());
        if texts.get(id)? != expected.as_bytes() {
            return Err("source original changed an actual text literal");
        }
        next = next.saturating_add(1);
    }
    if next != rows.len() {
        return Err("source original changed its complete text domain");
    }
    Ok(())
}
