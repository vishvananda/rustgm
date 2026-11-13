//! One selected interval replays vocabulary against its actual source members.
use super::{Error, Invocation, Transaction};
use crate::{
    expand::stream::{Origin, OriginKind},
    resolve::{expansion::Report, unit::Unit},
};

pub(super) fn require(
    report: &Report,
    entry: &Transaction,
    invocation: &Invocation,
) -> Result<(), Error> {
    let failure = || {
        Error::at(
            entry.owner.file,
            invocation.original.start,
            "derived vocabulary source introduction or edition changed",
        )
    };
    let unit = report.package.unit(entry.owner.file);
    let ledger = super::derive_ledger(&report.package)
        .ok()
        .flatten()
        .ok_or_else(failure)?;
    ledger
        .charge(
            unit.origins.len().checked_ilog2().unwrap_or(0) as usize * 2 + 1,
            0,
        )
        .map_err(|_| failure())?;
    let start = unit
        .origins
        .partition_point(|origin| origin.generated.end <= invocation.generated.start);
    let end = unit
        .origins
        .partition_point(|origin| origin.generated.start < invocation.generated.end);
    let origins = unit.origins.get(start..end).ok_or_else(failure)?;
    ledger
        .charge(origins.len().saturating_mul(4), 0)
        .map_err(|_| failure())?;
    if entry.contexts.len() > crate::expand::builtin::derive_output::MEMBERS.len()
        || entry.contexts.is_empty()
    {
        return Err(failure());
    }
    let mut observed = 0_u16;
    for origin in origins {
        if origin.generated.start < invocation.generated.start
            || origin.generated.end > invocation.generated.end
        {
            return Err(failure());
        }
        let generated = unit
            .source
            .text()
            .get(origin.generated.start as usize..origin.generated.end as usize)
            .ok_or_else(failure)?;
        match origin.kind {
            OriginKind::Captured(_) => return Err(failure()),
            OriginKind::Copied | OriginKind::Anchored => {
                ledger
                    .charge((origin.generated.end - origin.generated.start) as usize, 0)
                    .map_err(|_| failure())?;
                if !owner_text(unit, entry, origin, generated) {
                    return Err(failure());
                }
            }
            OriginKind::Synthetic(edition) => {
                let index = origin.introduction.0.checked_sub(1).ok_or_else(failure)? as usize;
                if !entry.contexts.contains(&index) {
                    return Err(failure());
                }
                let member = report.invocations.get(index).ok_or_else(failure)?;
                let definition = report
                    .definitions
                    .get(member.definition)
                    .ok_or_else(failure)?;
                if origin.original.file != entry.owner.file
                    || origin.original.span != invocation.original
                    || definition.edition != edition
                    || origin.context.is_none_or(|context| {
                        !report
                            .package
                            .marks
                            .is_opaque(context, definition.context, member.context)
                    })
                {
                    return Err(failure());
                }
                observed |= 1 << (index - entry.contexts.start);
            }
        }
    }
    if observed != (1 << entry.contexts.len()) - 1 {
        return Err(failure());
    }
    Ok(())
}

/// A copied range repeats the owner's bytes; an anchored tuple index names a
/// field inside the owner. Neither carries a context or an introduction.
fn owner_text(unit: &Unit, entry: &Transaction, origin: &Origin, generated: &str) -> bool {
    let span = origin.original.span;
    if origin.introduction.0 != 0
        || origin.context.is_some()
        || origin.original.file != entry.owner.file
    {
        return false;
    }
    if origin.kind == OriginKind::Anchored {
        let owner = unit.configured.ast.node(entry.owner.node).span;
        return span.start >= owner.start
            && span.end <= owner.end
            && crate::expand::stream::is_index(generated);
    }
    unit.source
        .text()
        .get(span.start as usize..span.end as usize)
        == Some(generated)
}
