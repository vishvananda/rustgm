//! Diagnostic identities and lint hints are never semantic proof or body factories.
use super::{Declaration, Reservations, SourceCore, policy::Feature, reservations::Claim};
use crate::{
    resolve::{
        package::{Error, Package},
        unit::{FileId, Unit},
    },
    support::bindings::LangItem,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::Forest,
    },
};

#[derive(Clone, Copy)]
pub(super) enum DiagnosticItem {
    Clone,
    Copy,
    Identity,
    CfgSelect,
    FormatArgsMacro,
    NonNull,
    Debug,
}
impl DiagnosticItem {
    pub(super) const fn index(self) -> usize {
        self as usize
    }
}
pub(super) fn parse(
    unit: &Unit,
    file: FileId,
    attribute: Attribute,
    forest: &Forest,
) -> Result<DiagnosticItem, Error> {
    let error = |message| Error::at(file, attribute.span.start, message);
    let Input::Equals(range) = attribute.input else {
        return Err(error("diagnostic item requires one string literal"));
    };
    if range.end != range.start + 1 {
        return Err(error("diagnostic item requires one string literal"));
    }
    let token = forest
        .token(range.start)
        .ok_or_else(|| error("missing diagnostic item literal"))?;
    let Some(crate::literal::Value::String(bytes)) =
        crate::literal::interpret(unit.source.text(), token)
            .map_err(|error| Error::at(file, error.offset, error.message))?
    else {
        return Err(error("diagnostic item requires one string literal"));
    };
    match bytes.as_ref() {
        b"Clone" => Ok(DiagnosticItem::Clone),
        b"Copy" => Ok(DiagnosticItem::Copy),
        b"convert_identity" => Ok(DiagnosticItem::Identity),
        b"cfg_select" => Ok(DiagnosticItem::CfgSelect),
        b"format_args_macro" => Ok(DiagnosticItem::FormatArgsMacro),
        b"NonNull" => Ok(DiagnosticItem::NonNull),
        b"Debug" => Ok(DiagnosticItem::Debug),
        _ => Err(error("unsupported source diagnostic identity")),
    }
}
pub(super) fn validate(
    package: &Package,
    source: &SourceCore,
    reserved: &mut Reservations,
    declaration: Declaration,
    claims: &[Claim],
) -> Result<(), Error> {
    let error = |message| super::reservations::error(package, declaration.origin, message);
    for claim in claims {
        let Claim::DiagnosticItem(item) = claim else {
            continue;
        };
        if !source.features.has(Feature::RustcAttributes) {
            return Err(error("diagnostic identity requires rustc_attrs"));
        }
        let expected = match item {
            DiagnosticItem::Clone => Some(LangItem::Clone),
            DiagnosticItem::Copy => Some(LangItem::Copy),
            DiagnosticItem::Identity
            | DiagnosticItem::CfgSelect
            | DiagnosticItem::FormatArgsMacro
            | DiagnosticItem::NonNull
            | DiagnosticItem::Debug => None,
        };
        if let Some(expected) = expected {
            if !claims
                .iter()
                .any(|claim| matches!(claim, Claim::Language(role) if *role == expected))
            {
                return Err(error(
                    "diagnostic identity has a different actual language owner",
                ));
            }
        } else if matches!(
            item,
            DiagnosticItem::CfgSelect | DiagnosticItem::FormatArgsMacro
        ) {
            let role = if matches!(item, DiagnosticItem::CfgSelect) {
                crate::expand::builtin::Builtin::CfgSelect
            } else {
                crate::expand::builtin::Builtin::FormatArgs
            };
            if !claims
                .iter()
                .any(|claim| matches!(claim,Claim::BuiltinMacro(actual) if *actual==role))
            {
                return Err(error(if matches!(item, DiagnosticItem::CfgSelect) {
                    "cfg_select diagnostic identity requires its actual builtin role"
                } else {
                    "format_args diagnostic identity requires its actual builtin role"
                }));
            }
        } else if matches!(item, DiagnosticItem::Debug) {
            let unit = package.unit(declaration.origin.file);
            if declaration.parent.is_some()
                || !matches!(unit.configured.ast.node(declaration.origin.node).kind,
                    Kind::Item(index) if matches!(unit.configured.ast.items[index as usize].kind, ItemKind::Trait { .. }))
            {
                return Err(error(
                    "Debug diagnostic identity requires its actual module trait",
                ));
            }
        } else if matches!(item, DiagnosticItem::NonNull) {
            // A diagnostic identity is not a language role, layout marker or naming requirement.
            let unit = package.unit(declaration.origin.file);
            if !matches!(unit.configured.ast.node(declaration.origin.node).kind, Kind::Item(index) if matches!(unit.configured.ast.items[index as usize].kind, ItemKind::Struct { .. }))
            {
                return Err(error(
                    "NonNull diagnostic item requires its nominal declaration profile",
                ));
            }
        } else {
            let unit = package.unit(declaration.origin.file);
            let Kind::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
                return Err(error(
                    "diagnostic function identity has no actual declaration",
                ));
            };
            if declaration.parent.is_some()
                || !matches!(
                    unit.configured.ast.items[index as usize].kind,
                    ItemKind::Function { body: Some(_), .. }
                )
            {
                return Err(error(
                    "diagnostic function identity requires a real module-level body",
                ));
            }
        }
        let slot = &mut reserved.diagnostics[item.index()];
        if slot.is_some() {
            return Err(error("duplicate source diagnostic identity"));
        }
        *slot = Some(declaration);
        reserved.steps = reserved.steps.saturating_add(1);
    }
    Ok(())
}
