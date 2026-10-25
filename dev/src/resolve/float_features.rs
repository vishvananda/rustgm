//! Wide grammar facts require actual requests or the shared literal-origin authority.
use super::{
    bindings::FeatureName,
    expansion::{MacroAllowanceSession, Report},
    model::{DefId, DefKind, Target},
    package::Error,
    unit::{FileId, NodeKey},
};
use crate::{
    literal::{Value, value::FloatType},
    syntax::{ast::Kind, token_tree::Span},
};

fn require(
    report: &Report,
    session: &mut MacroAllowanceSession<'_>,
    file: FileId,
    span: Span,
    half: bool,
) -> Result<(), Error> {
    if report.package.requested_wide_float(half) {
        return Ok(());
    }
    let name = FeatureName::new(if half { "f16" } else { "f128" }).map_err(Error::unlocated)?;
    if session.allows_at(file, span, &name)? {
        return Ok(());
    }
    Err(Error::at(
        file,
        span.start,
        "wide floating type requires its feature or actual literal macro allowance",
    ))
}

pub(super) fn validate_literals(report: &Report) -> Result<(), Error> {
    if report.package.requested_wide_float(true) && report.package.requested_wide_float(false) {
        return Ok(());
    }
    let mut session = report.macro_allowances();
    literals(&report.package, |file, span, half| {
        require(report, &mut session, file, span, half)
    })
}

fn literals(
    package: &super::package::Package,
    mut check: impl FnMut(FileId, Span, bool) -> Result<(), Error>,
) -> Result<(), Error> {
    for (file, unit) in package.units.iter().enumerate() {
        let file = FileId(
            u32::try_from(file).map_err(|_| Error::unlocated("wide literal file overflow"))?,
        );
        if !unit.configured.ast.literals.iter().any(|value| {
            matches!(
                value,
                crate::literal::storage::StoredValue::Float {
                    suffix: Some(FloatType::F16 | FloatType::F128),
                    ..
                }
            )
        }) {
            continue;
        }
        for (index, node) in unit.configured.ast.nodes.iter().enumerate() {
            if !unit.configured.active[index] {
                continue;
            }
            let Kind::Literal(index) = node.kind else {
                continue;
            };
            let Value::Float(value) = unit
                .literal(index)
                .map_err(|e| Error::at(file, e.offset, e.message))?
            else {
                continue;
            };
            match value.suffix {
                Some(FloatType::F16) => check(file, node.span, true)?,
                Some(FloatType::F128) => check(file, node.span, false)?,
                _ => {}
            }
        }
    }
    Ok(())
}

fn primitive(
    package: &super::package::Package,
    resolution: &super::model::Resolution,
    definition: DefId,
) -> Option<bool> {
    let definition = resolution.definitions.get(definition.0 as usize)?;
    if definition.kind != DefKind::Primitive {
        return None;
    }
    match package.names.get(definition.name) {
        "f16" => Some(true),
        "f128" => Some(false),
        _ => None,
    }
}
fn path(
    report: &Report,
    session: &mut MacroAllowanceSession<'_>,
    key: NodeKey,
    definition: DefId,
) -> Result<(), Error> {
    if let Some(half) = primitive(&report.package, &report.resolution, definition) {
        let ast = &report.package.unit(key.file).configured.ast;
        let node = ast
            .path_segments(key.node)
            .and_then(|(_, nodes)| nodes.last().copied())
            .unwrap_or(key.node);
        require(report, session, key.file, ast.node(node).span, half)?;
    }
    Ok(())
}
pub(super) fn validate_paths(report: &Report) -> Result<(), Error> {
    if report.package.requested_wide_float(true) && report.package.requested_wide_float(false) {
        return Ok(());
    }
    let mut session = report.macro_allowances();
    for reference in &report.resolution.references {
        if let Target::Definition(definition) = reference.target {
            path(report, &mut session, reference.node, definition)?;
        }
    }
    for (&key, selected) in &report.resolution.associated_paths {
        path(
            report,
            &mut session,
            NodeKey {
                file: key.file,
                node: selected.prefix,
            },
            selected.owner,
        )?;
    }
    Ok(())
}

/// The unexpanded API has no executed frames; only checked global requests apply.
pub(super) fn validate_plain(
    package: &super::package::Package,
    resolution: &super::model::Resolution,
) -> Result<(), Error> {
    let check = |file: FileId, span: Span, half| {
        if package.requested_wide_float(half) {
            Ok(())
        } else {
            Err(Error::at(
                file,
                span.start,
                "wide floating type requires its feature or expanded literal provenance",
            ))
        }
    };
    literals(package, check)?;
    for reference in &resolution.references {
        if let Target::Definition(definition) = reference.target
            && let Some(half) = primitive(package, resolution, definition)
        {
            check(
                reference.node.file,
                package
                    .unit(reference.node.file)
                    .configured
                    .ast
                    .node(reference.node.node)
                    .span,
                half,
            )?;
        }
    }
    for (&key, selected) in &resolution.associated_paths {
        if let Some(half) = primitive(package, resolution, selected.owner) {
            check(
                key.file,
                package.unit(key.file).configured.ast.node(key.node).span,
                half,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/invariants/float_features.rs"]
mod tests;
