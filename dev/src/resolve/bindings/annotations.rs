//! Diagnostic identities and lint hints are never semantic proof or body factories.
use super::{Declaration, Reservations, SourceCore, policy::Feature, reservations::Claim};
use crate::{
    resolve::package::{Error, Package},
    support::bindings::LangItem,
    syntax::{ast::Kind, item::ItemKind},
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
    Alignment,
    Send,
    Sync,
    AssertMacro,
    Formatter,
    Display,
    FmtWrite,
    PartialEq,
    PartialEqEq,
    PartialEqNe,
    Eq,
    EqTotal,
    Ord,
    OrdCmp,
    OrdMax,
    OrdMin,
    PartialOrd,
    PartialOrdCmp,
    PartialOrdLt,
    PartialOrdLe,
    PartialOrdGt,
    PartialOrdGe,
    Default,
    DefaultFn,
    Hash,
    Result,
    Option,
    NegMethod,
}
impl DiagnosticItem {
    pub(super) const fn index(self) -> usize {
        self as usize
    }
}
pub(super) fn known(bytes: &[u8]) -> Option<DiagnosticItem> {
    Some(match bytes {
        b"Clone" => DiagnosticItem::Clone,
        b"Copy" => DiagnosticItem::Copy,
        b"convert_identity" => DiagnosticItem::Identity,
        b"cfg_select" => DiagnosticItem::CfgSelect,
        b"format_args_macro" => DiagnosticItem::FormatArgsMacro,
        b"NonNull" => DiagnosticItem::NonNull,
        b"Debug" => DiagnosticItem::Debug,
        b"Alignment" => DiagnosticItem::Alignment,
        b"Send" => DiagnosticItem::Send,
        b"Sync" => DiagnosticItem::Sync,
        b"assert_macro" => DiagnosticItem::AssertMacro,
        b"Formatter" => DiagnosticItem::Formatter,
        b"Display" => DiagnosticItem::Display,
        b"FmtWrite" => DiagnosticItem::FmtWrite,
        b"PartialEq" => DiagnosticItem::PartialEq,
        b"cmp_partialeq_eq" => DiagnosticItem::PartialEqEq,
        b"cmp_partialeq_ne" => DiagnosticItem::PartialEqNe,
        b"Eq" => DiagnosticItem::Eq,
        b"assert_receiver_is_total_eq" => DiagnosticItem::EqTotal,
        b"Ord" => DiagnosticItem::Ord,
        b"ord_cmp_method" => DiagnosticItem::OrdCmp,
        b"cmp_ord_max" => DiagnosticItem::OrdMax,
        b"cmp_ord_min" => DiagnosticItem::OrdMin,
        b"PartialOrd" => DiagnosticItem::PartialOrd,
        b"cmp_partialord_cmp" => DiagnosticItem::PartialOrdCmp,
        b"cmp_partialord_lt" => DiagnosticItem::PartialOrdLt,
        b"cmp_partialord_le" => DiagnosticItem::PartialOrdLe,
        b"cmp_partialord_gt" => DiagnosticItem::PartialOrdGt,
        b"cmp_partialord_ge" => DiagnosticItem::PartialOrdGe,
        b"Default" => DiagnosticItem::Default,
        b"default_fn" => DiagnosticItem::DefaultFn,
        b"Hash" => DiagnosticItem::Hash,
        b"Result" => DiagnosticItem::Result,
        b"Option" => DiagnosticItem::Option,
        b"neg" => DiagnosticItem::NegMethod,

        _ => return None,
    })
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
        let expected = expected_language(*item);
        if let Some(expected) = expected {
            if !claims
                .iter()
                .any(|claim| matches!(claim, Claim::Language(role) if *role == expected))
            {
                return Err(error(
                    "diagnostic identity has a different actual language owner",
                ));
            }
            if matches!(item, DiagnosticItem::PartialEq | DiagnosticItem::PartialOrd) {
                validate_module_item(package, declaration, *item)?;
            }
        } else if matches!(
            item,
            DiagnosticItem::CfgSelect
                | DiagnosticItem::FormatArgsMacro
                | DiagnosticItem::AssertMacro
        ) {
            let role = match item {
                DiagnosticItem::CfgSelect => crate::expand::builtin::Builtin::CfgSelect,
                DiagnosticItem::AssertMacro => crate::expand::builtin::Builtin::Assert,
                _ => crate::expand::builtin::Builtin::FormatArgs,
            };
            if !claims
                .iter()
                .any(|claim| matches!(claim,Claim::BuiltinMacro(actual) if *actual==role))
            {
                return Err(error(if matches!(item, DiagnosticItem::CfgSelect) {
                    "cfg_select diagnostic identity requires its actual builtin role"
                } else if matches!(item, DiagnosticItem::AssertMacro) {
                    "assert diagnostic identity requires its actual builtin role"
                } else {
                    "format_args diagnostic identity requires its actual builtin role"
                }));
            }
        } else if let Some(expected_owner) = member_owner(*item) {
            validate_member(package, reserved, declaration, expected_owner)?;
        } else if matches!(
            item,
            DiagnosticItem::Debug
                | DiagnosticItem::Alignment
                | DiagnosticItem::Send
                | DiagnosticItem::Formatter
                | DiagnosticItem::Display
                | DiagnosticItem::FmtWrite
                | DiagnosticItem::Eq
                | DiagnosticItem::Ord
                | DiagnosticItem::Default
                | DiagnosticItem::Hash
                | DiagnosticItem::Result
                | DiagnosticItem::Option
        ) {
            validate_module_item(package, declaration, *item)?;
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

fn validate_module_item(
    package: &Package,
    declaration: Declaration,
    item: DiagnosticItem,
) -> Result<(), Error> {
    let unit = package.unit(declaration.origin.file);
    let right_kind = matches!(unit.configured.ast.node(declaration.origin.node).kind,
        Kind::Item(index) if matches!((item,unit.configured.ast.items[index as usize].kind),
            (DiagnosticItem::Debug | DiagnosticItem::Display | DiagnosticItem::FmtWrite | DiagnosticItem::PartialEq | DiagnosticItem::PartialOrd
                    | DiagnosticItem::Eq | DiagnosticItem::Ord | DiagnosticItem::Default | DiagnosticItem::Hash,ItemKind::Trait{..})
                | (DiagnosticItem::Formatter,ItemKind::Struct{..})
                | (DiagnosticItem::Alignment | DiagnosticItem::Result | DiagnosticItem::Option,ItemKind::Enum{..})
                | (DiagnosticItem::Send,ItemKind::Trait{auto_:true,unsafe_:true,constant:false,..})));
    if declaration.parent.is_some() || !right_kind {
        return Err(super::reservations::error(
            package,
            declaration.origin,
            if matches!(item, DiagnosticItem::Send) {
                "Send diagnostic identity requires its actual unsafe module auto trait"
            } else if matches!(item, DiagnosticItem::Debug | DiagnosticItem::Alignment) {
                "diagnostic identity requires its actual Debug trait or Alignment enum"
            } else {
                "diagnostic identity requires its actual module declaration kind"
            },
        ));
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum MemberOwner {
    Diagnostic(DiagnosticItem),
    Language(LangItem),
}
fn member_owner(item: DiagnosticItem) -> Option<MemberOwner> {
    let expected = match item {
        DiagnosticItem::NegMethod => return Some(MemberOwner::Language(LangItem::Neg)),
        DiagnosticItem::PartialEqEq | DiagnosticItem::PartialEqNe => DiagnosticItem::PartialEq,
        DiagnosticItem::EqTotal => DiagnosticItem::Eq,
        DiagnosticItem::OrdCmp | DiagnosticItem::OrdMax | DiagnosticItem::OrdMin => {
            DiagnosticItem::Ord
        }
        DiagnosticItem::PartialOrdCmp
        | DiagnosticItem::PartialOrdLt
        | DiagnosticItem::PartialOrdLe
        | DiagnosticItem::PartialOrdGt
        | DiagnosticItem::PartialOrdGe => DiagnosticItem::PartialOrd,
        DiagnosticItem::DefaultFn => DiagnosticItem::Default,
        _ => return None,
    };
    Some(MemberOwner::Diagnostic(expected))
}
fn validate_member(
    package: &Package,
    reserved: &Reservations,
    declaration: Declaration,
    expected: MemberOwner,
) -> Result<(), Error> {
    let owner = match expected {
        MemberOwner::Diagnostic(item) => reserved.diagnostics[item.index()],
        MemberOwner::Language(item) => reserved.language(item),
    }
    .ok_or_else(|| {
        super::reservations::error(
            package,
            declaration.origin,
            "diagnostic member requires its actual trait owner",
        )
    })?;
    validate_member_definition(package, declaration, owner.definition)
}
fn validate_member_definition(
    package: &Package,
    declaration: Declaration,
    owner: crate::resolve::model::DefId,
) -> Result<(), Error> {
    let ast = &package.unit(declaration.origin.file).configured.ast;
    if declaration.parent != Some(owner)
        || !matches!(ast.node(declaration.origin.node).kind,
        Kind::Item(index) if matches!(ast.items[index as usize].kind, ItemKind::Function {..}))
    {
        return Err(super::reservations::error(
            package,
            declaration.origin,
            "diagnostic member has another declaration kind or trait owner",
        ));
    }
    Ok(())
}

fn expected_language(item: DiagnosticItem) -> Option<LangItem> {
    match item {
        DiagnosticItem::Clone => Some(LangItem::Clone),
        DiagnosticItem::Copy => Some(LangItem::Copy),
        DiagnosticItem::Sync => Some(LangItem::Sync),
        DiagnosticItem::PartialEq => Some(LangItem::PartialEq),
        DiagnosticItem::PartialOrd => Some(LangItem::PartialOrd),
        DiagnosticItem::Identity
        | DiagnosticItem::CfgSelect
        | DiagnosticItem::FormatArgsMacro
        | DiagnosticItem::NonNull
        | DiagnosticItem::Debug
        | DiagnosticItem::Alignment
        | DiagnosticItem::Send
        | DiagnosticItem::AssertMacro
        | DiagnosticItem::Formatter
        | DiagnosticItem::Display
        | DiagnosticItem::FmtWrite
        | DiagnosticItem::PartialEqEq
        | DiagnosticItem::PartialEqNe
        | DiagnosticItem::Eq
        | DiagnosticItem::EqTotal
        | DiagnosticItem::Ord
        | DiagnosticItem::OrdCmp
        | DiagnosticItem::OrdMax
        | DiagnosticItem::OrdMin
        | DiagnosticItem::PartialOrdCmp
        | DiagnosticItem::PartialOrdLt
        | DiagnosticItem::PartialOrdLe
        | DiagnosticItem::PartialOrdGt
        | DiagnosticItem::PartialOrdGe
        | DiagnosticItem::Default
        | DiagnosticItem::DefaultFn
        | DiagnosticItem::Hash
        | DiagnosticItem::Result
        | DiagnosticItem::Option
        | DiagnosticItem::NegMethod => None,
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/enum_claims.rs"]
mod tests;
