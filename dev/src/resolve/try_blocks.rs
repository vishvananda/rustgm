//! Typed try syntax consumes actual source and span authority before checking.
use super::{
    bindings::FeatureName,
    expansion::Report,
    package::{Error, Package},
    unit::FileId,
};
use crate::syntax::gate_sites::Expression;
fn walk(
    package: &Package,
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
            let ordinal = u32::try_from(ordinal)
                .map_err(|_| Error::unlocated("try-block ordinal overflow"))?;
            super::gate_provenance::checked_expression(package, file, ordinal, entry)?;
            if unit.configured.active.get(entry.node.get() as usize) != Some(&true) {
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
    walk(&report.package, |file, ordinal, entry| {
        if requested || session.allows_try_block(file, ordinal, entry, &feature)? {
            return Ok(());
        }
        Err(Error::at(
            file,
            entry.keyword.start,
            "try block requires try_blocks or its actual expression allowance",
        ))
    })
}
pub(super) fn validate_plain(package: &Package) -> Result<(), Error> {
    let requested = package.requested_try_blocks().map_err(Error::unlocated)?;
    walk(package, |file, _, entry| {
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
