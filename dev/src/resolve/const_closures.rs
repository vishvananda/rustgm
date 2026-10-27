//! The pinned parser gates the const keyword, even in cfg-removed closures.
use super::{
    bindings::FeatureName,
    expansion::Report,
    package::{Error, Package},
};
use crate::syntax::gate_sites::ExpressionKind;

pub(super) fn validate(report: &Report) -> Result<(), Error> {
    let requested = report
        .package
        .requested_const_closures()
        .map_err(Error::unlocated)?;
    let feature = FeatureName::new("const_closures").map_err(Error::unlocated)?;
    let mut session = report.macro_allowances();
    super::try_blocks::walk(
        &report.package,
        ExpressionKind::ConstClosure,
        |file, _, entry| {
            if requested || session.allows_at(file, entry.keyword, &feature)? {
                return Ok(());
            }
            Err(Error::at(
                file,
                entry.keyword.start,
                "const closure requires const_closures or its actual keyword allowance",
            ))
        },
    )
}

pub(super) fn validate_plain(package: &Package) -> Result<(), Error> {
    let requested = package
        .requested_const_closures()
        .map_err(Error::unlocated)?;
    super::try_blocks::walk(package, ExpressionKind::ConstClosure, |file, _, entry| {
        if requested {
            return Ok(());
        }
        Err(Error::at(
            file,
            entry.keyword.start,
            "const closure requires const_closures",
        ))
    })
}

#[cfg(test)]
#[path = "../../tests/invariants/const_closure_syntax.rs"]
mod tests;
