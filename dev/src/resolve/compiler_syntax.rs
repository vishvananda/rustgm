//! Builtin syntax uses its actual emitted name, independently of enum authority.
use super::{
    bindings::FeatureName,
    expansion::Report,
    package::{Error, Package},
};
use crate::syntax::gate_sites::ExpressionKind;

pub(crate) fn require_offset(
    report: &Report,
    key: super::unit::NodeKey,
) -> Result<crate::syntax::token_tree::Span, Error> {
    let unit = report.package.unit(key.file);
    let crate::syntax::ast::Kind::OffsetOf { gate, fields, .. } =
        unit.configured.ast.node(key.node).kind
    else {
        return Err(Error::at(
            key.file,
            0,
            "offset expression lost its actual parsed site",
        ));
    };
    let entry = *unit
        .configured
        .ast
        .expression_gate(gate)
        .ok_or_else(|| Error::at(key.file, 0, "offset expression lost its sealed site"))?;
    if entry.node != key.node || unit.configured.active.get(key.node.get() as usize) != Some(&true)
    {
        return Err(Error::at(
            key.file,
            entry.keyword.start,
            "offset expression changed its active source site",
        ));
    }
    let sites = unit.configured.ast.gate_sites.as_ref().ok_or_else(|| {
        Error::at(
            key.file,
            entry.keyword.start,
            "offset gate owner disappeared",
        )
    })?;
    crate::syntax::offset_fields::visit(&unit.configured.ast, fields, |_, span| {
        sites
            .charge(2, 0)
            .map_err(|message| crate::syntax::token_tree::error(span.start, message))
    })
    .map_err(|error| Error::at(key.file, error.offset, error.message))?;
    super::gate_provenance::checked_expression(&report.package, key.file, gate, entry)?;
    let requested = report
        .package
        .requested_builtin_syntax()
        .map_err(Error::unlocated)?;
    let feature = FeatureName::new("builtin_syntax").map_err(Error::unlocated)?;
    if !requested
        && !report
            .macro_allowances()
            .allows_at(key.file, entry.keyword, &feature)?
    {
        return Err(Error::at(
            key.file,
            entry.keyword.start,
            "builtin expression requires builtin_syntax or its actual name allowance",
        ));
    }
    Ok(entry.keyword)
}

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
