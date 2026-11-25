//! Version-fourteen non-tracked body caller sites are copied out of their
//! sealed view and interned into the consumer's constant arena once that view
//! is released.
use super::ImportedOriginals;
use crate::{
    body::ir::{Body, CallerSite, Rvalue, ValueId},
    resolve::expansion::Report,
    sema::{
        crates::SourceOriginal,
        model::Declarations,
        values::{Evaluator, Failure, Reader},
    },
};

#[derive(Default)]
pub(super) struct Pending(Vec<(u32, Box<str>, u32, u32)>);

/// Preflight byte and visit envelope for an original's caller sites.
pub(super) fn allocation(
    rows: crate::support::crates::generics::source::ValidatedView<'_>,
    body: crate::support::crates::owner_catalog::BodyRef,
) -> (usize, usize) {
    let (files, sites) = rows.caller_sites(body);
    if sites.is_empty() {
        return (0, 0);
    }
    let bytes = size_of::<crate::body::ir::CallableInputs>().saturating_add(
        sites
            .len()
            .saturating_mul(4 * size_of::<crate::body::ir::CallerSite>()),
    );
    let mut visits = sites.len();
    for row in sites {
        if let Some(file) = files.get(row.file as usize) {
            visits = visits.saturating_add(file.len());
        }
    }
    (bytes, visits)
}

/// Copy every caller site owned by this original, charging each row and path.
pub(super) fn copy(
    view: &SourceOriginal<'_>,
    work: &mut ImportedOriginals,
) -> Result<Pending, &'static str> {
    let table = view.rows();
    let (files, rows) = table.caller_sites(view.raw().body);
    if rows.is_empty() {
        return Ok(Pending::default());
    }
    work.charge(rows.len())?;
    let mut copied = Vec::with_capacity(rows.len());
    for row in rows {
        let file = files
            .get(row.file as usize)
            .ok_or("source caller site file ordinal is out of range")?;
        work.charge(file.len().saturating_add(1))?;
        copied.push((row.value, file.clone(), row.line, row.column));
    }
    Ok(Pending(copied))
}

/// Reconstruct each caller location into the consumer's own `Location` nominal.
pub(super) fn intern(
    report: &Report,
    facts: &mut Declarations,
    body: &mut Body,
    pending: Pending,
) -> Result<(), &'static str> {
    if pending.0.is_empty() {
        return Ok(());
    }
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(pending.0.len());
    for (value, file, line, column) in pending.0 {
        let Rvalue::Call { callee, .. } = body
            .values
            .get(value as usize)
            .ok_or("source caller site value is missing")?
            .kind
        else {
            return Err("source caller site requires an actual Call value");
        };
        let callable = body
            .values
            .get(callee.0 as usize)
            .ok_or("source caller site callee is missing")?
            .ty;
        let location = Evaluator::new(report, facts)
            .prepare_caller_location(&file, line, column)
            .map_err(Failure::message)?;
        body.append_caller_site(
            CallerSite {
                owner: body.source,
                value: ValueId(value),
                callable,
                location,
            },
            facts,
        )?;
    }
    body.validate_caller_sites()
}

/// A cached imported body retains exact caller-site coordinates in call order.
pub(super) fn validate(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let table = view.rows();
    let (files, rows) = table.caller_sites(view.raw().body);
    let actual = body.caller_sites();
    if actual.len() != rows.len() {
        return Err("source original changed its caller-site inventory");
    }
    let reader = Reader::new(facts);
    for (site, expected) in actual.iter().zip(rows) {
        let expected_file = files
            .get(expected.file as usize)
            .ok_or("source caller site file ordinal is out of range")?;
        *steps = steps.saturating_add(1).saturating_add(expected_file.len());
        if site.owner != body.source || site.value.0 != expected.value {
            return Err("source original changed an actual caller-site owner or call value");
        }
        let (file, line, column) = reader.caller_location_parts(site.location)?;
        if file != expected_file.as_ref() || line != expected.line || column != expected.column {
            return Err("source original changed an actual caller-site coordinate");
        }
    }
    if !actual.is_empty() {
        body.validate_caller_sites()?;
    }
    Ok(())
}
