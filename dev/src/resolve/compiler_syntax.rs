//! Builtin syntax uses its actual emitted name, independently of enum authority.
use super::{
    bindings::FeatureName,
    expansion::Report,
    package::{Error, Package},
};
use crate::syntax::gate_sites::ExpressionKind;

pub(super) fn validate(report: &Report) -> Result<(), Error> {
    let requested = report
        .package
        .requested_builtin_syntax()
        .map_err(Error::unlocated)?;
    let feature = FeatureName::new("builtin_syntax").map_err(Error::unlocated)?;
    let mut session = report.macro_allowances();
    super::try_blocks::walk(
        &report.package,
        ExpressionKind::OffsetOf,
        |file, _, entry| {
            if requested || session.allows_at(file, entry.keyword, &feature)? {
                return Ok(());
            }
            Err(Error::at(
                file,
                entry.keyword.start,
                "builtin expression requires builtin_syntax or its actual name allowance",
            ))
        },
    )
}

pub(super) fn validate_plain(package: &Package) -> Result<(), Error> {
    let requested = package
        .requested_builtin_syntax()
        .map_err(Error::unlocated)?;
    super::try_blocks::walk(package, ExpressionKind::OffsetOf, |file, _, entry| {
        if requested {
            return Ok(());
        }
        Err(Error::at(
            file,
            entry.keyword.start,
            "builtin expression requires builtin_syntax",
        ))
    })
}

#[cfg(test)]
#[path = "../../tests/invariants/offset_contracts.rs"]
mod tests;
