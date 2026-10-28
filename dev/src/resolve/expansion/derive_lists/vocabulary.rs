//! One selected interval replays vocabulary against its actual source members.
use super::{Error, Invocation, Transaction};
use crate::{expand::stream::OriginKind, resolve::expansion::Report};

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
    let source = report.package.source_core().ok_or_else(failure)?;
    source
        .macros
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
    source
        .macros
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
            OriginKind::Copied => {
                source
                    .macros
                    .charge((origin.generated.end - origin.generated.start) as usize, 0)
                    .map_err(|_| failure())?;
                if origin.introduction.0 != 0
                    || origin.context.is_some()
                    || origin.original.file != entry.owner.file
                    || unit
                        .source
                        .text()
                        .get(origin.original.span.start as usize..origin.original.span.end as usize)
                        .ok_or_else(failure)?
                        != generated
                {
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
