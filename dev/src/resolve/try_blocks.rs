//! Typed try syntax consumes actual source and span authority before checking.
use super::{
    bindings::FeatureName,
    expansion::Report,
    package::{Error, Package},
    unit::FileId,
};
use crate::syntax::gate_sites::{Expression, ExpressionKind};
pub(super) fn walk(
    package: &Package,
    kind: ExpressionKind,
    mut require: impl FnMut(FileId, u32, Expression) -> Result<(), Error>,
) -> Result<(), Error> {
    for (index, unit) in package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let Some(entries) = &unit.configured.ast.gate_sites else {
            continue;
        };
        let file = FileId(
            u32::try_from(index)
                .map_err(|_| Error::unlocated("try-block file identity overflow"))?,
        );
        for (ordinal, entry) in entries.expressions().iter().copied().enumerate() {
            entries
                .charge(1, 0)
                .map_err(|message| Error::at(file, entry.keyword.start, message))?;
            if entry
                .kind(&unit.configured.ast)
                .map_err(|error| Error::at(file, error.offset, error.message))?
                != kind
            {
                continue;
            }
            let ordinal = u32::try_from(ordinal)
                .map_err(|_| Error::unlocated("try-block ordinal overflow"))?;
            super::gate_provenance::checked_expression(package, file, ordinal, entry)?;
            if kind == ExpressionKind::TryBlock
                && unit.configured.active.get(entry.node.get() as usize) != Some(&true)
            {
                continue;
            }
            require(file, ordinal, entry)?;
        }
    }
    Ok(())
}
pub(super) fn validate(report: &Report) -> Result<(), Error> {
    let requested = report
        .package
        .requested_try_blocks()
        .map_err(Error::unlocated)?;
    let feature = FeatureName::new("try_blocks").map_err(Error::unlocated)?;
    let mut session = report.macro_allowances();
    walk(
        &report.package,
        ExpressionKind::TryBlock,
        |file, ordinal, entry| {
            if requested || session.allows_try_block(file, ordinal, entry, &feature)? {
                return Ok(());
            }
            Err(Error::at(
                file,
                entry.keyword.start,
                "try block requires try_blocks or its actual expression allowance",
            ))
        },
    )
}
/// Replay one demanded expression through the same sealed site and allowance rule.
pub(crate) fn require(report: &Report, key: super::unit::NodeKey) -> Result<(), Error> {
    let unit = report
        .package
        .units
        .get(key.file.0 as usize)
        .ok_or_else(|| Error::at(key.file, 0, "Try expression lost its actual source unit"))?;
    let Some(crate::syntax::ast::Kind::TryBlock { gate, .. }) = unit
        .configured
        .ast
        .nodes
        .get(key.node.get() as usize)
        .map(|node| node.kind)
    else {
        return Err(Error::at(
            key.file,
            0,
            "Try expression lost its actual parsed wrapper",
        ));
    };
    let entry = *unit
        .configured
        .ast
        .expression_gate(gate)
        .ok_or_else(|| Error::at(key.file, 0, "Try expression lost its actual sealed site"))?;
    if entry.node != key.node || unit.configured.active.get(key.node.get() as usize) != Some(&true)
    {
        return Err(Error::at(
            key.file,
            entry.keyword.start,
            "Try expression is not its actual active sealed site",
        ));
    }
    super::gate_provenance::checked_expression(&report.package, key.file, gate, entry)?;
    if report
        .package
        .requested_try_blocks()
        .map_err(Error::unlocated)?
    {
        return Ok(());
    }
    let feature = FeatureName::new("try_blocks").map_err(Error::unlocated)?;
    if report
        .macro_allowances()
        .allows_try_block(key.file, gate, entry, &feature)?
    {
        return Ok(());
    }
    Err(Error::at(
        key.file,
        entry.keyword.start,
        "try block requires try_blocks or its actual expression allowance",
    ))
}

pub(super) fn validate_plain(package: &Package) -> Result<(), Error> {
    let requested = package.requested_try_blocks().map_err(Error::unlocated)?;
    walk(package, ExpressionKind::TryBlock, |file, _, entry| {
        if requested {
            return Ok(());
        }
        Err(Error::at(
            file,
            entry.keyword.start,
            "try block requires try_blocks",
        ))
    })
}
#[cfg(test)]
#[path = "../../tests/invariants/try_syntax.rs"]
mod tests;
