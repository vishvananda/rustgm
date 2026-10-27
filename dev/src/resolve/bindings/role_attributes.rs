//! Exact source role claims are data; arbitrary attributes grant no trait permission.
use super::{Declaration, Reservations, SourceCore, policy::Feature, reservations::Claim};
use crate::{
    resolve::package::{Error, Package},
    support::bindings::LangItem,
    syntax::attribute::Input,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HeaderAttribute {
    Fundamental,
    Specialization,
    Coinductive,
    DynIncompatible,
    Marker,
}
impl HeaderAttribute {
    pub(super) fn named(name: &str) -> Option<Self> {
        Some(match name {
            "fundamental" => Self::Fundamental,
            "rustc_specialization_trait" => Self::Specialization,
            "rustc_coinductive" => Self::Coinductive,
            "rustc_dyn_incompatible_trait" => Self::DynIncompatible,
            _ => return None,
        })
    }
    const fn bit(self) -> u8 {
        1 << self as u8
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HeaderAttributes(u8);
impl HeaderAttributes {
    pub(crate) const EMPTY: Self = Self(0);
    /// Retained claims only. Consumers still validate their exact role owner,
    /// profile and deny-implementation contract before assigning any semantics.
    pub(crate) fn retained(attributes: impl IntoIterator<Item = HeaderAttribute>) -> Self {
        Self(
            attributes
                .into_iter()
                .fold(0, |bits, attribute| bits | attribute.bit()),
        )
    }
    pub(crate) const fn contains(self, attribute: HeaderAttribute) -> bool {
        self.0 & attribute.bit() != 0
    }
    pub(crate) fn with_marker(self, marker: bool) -> Self {
        Self(
            self.0
                | if marker {
                    HeaderAttribute::Marker.bit()
                } else {
                    0
                },
        )
    }
    pub(crate) fn requires_sizing_profile(self) -> bool {
        [
            HeaderAttribute::Fundamental,
            HeaderAttribute::Specialization,
            HeaderAttribute::Coinductive,
        ]
        .into_iter()
        .any(|attribute| self.contains(attribute))
    }
}

pub(super) fn claim(name: &str, input: Input) -> Result<Option<Claim>, &'static str> {
    if let Some(attribute) = HeaderAttribute::named(name) {
        if input != Input::Empty {
            return Err("source role header attribute requires no arguments");
        }
        return Ok(Some(Claim::Header(attribute)));
    }
    if matches!(
        name,
        "rustc_nonnull_optimization_guaranteed" | "rustc_never_returns_null_ptr"
    ) {
        if input != Input::Empty {
            return Err("non-null source attribute requires no arguments");
        }
        return Ok(Some(if name == "rustc_nonnull_optimization_guaranteed" {
            Claim::NonNullDocumentation
        } else {
            Claim::NeverNullLint
        }));
    }
    if name == "rustc_trivial_field_reads" {
        if input != Input::Empty {
            return Err("rustc_trivial_field_reads requires no arguments");
        }
        return Ok(Some(Claim::TrivialFieldReads));
    }
    if matches!(
        name,
        "stable" | "unstable" | "rustc_const_stable" | "rustc_const_unstable"
    ) {
        return Err("source API and const-stability contracts are not implemented");
    }
    Ok(None)
}

pub(super) fn validate(
    package: &Package,
    source: &SourceCore,
    reserved: &mut Reservations,
    declaration: Declaration,
    claims: &[Claim],
) -> Result<(), Error> {
    let error = |message| super::reservations::error(package, declaration.origin, message);
    validate_nonnull_claims(package, source, reserved, declaration, claims)?;
    let mut flags = HeaderAttributes::EMPTY;
    let mut roles = claims.iter().filter_map(|claim| {
        if let Claim::Language(role) = claim {
            Some(*role)
        } else {
            None
        }
    });
    let role = roles.next().filter(|_| roles.next().is_none());
    for claim in claims {
        let Claim::Header(attribute) = claim else {
            continue;
        };
        if !source.features.has(Feature::RustcAttributes) {
            return Err(error("source role header attribute requires rustc_attrs"));
        }
        if *attribute == HeaderAttribute::Fundamental && !source.features.has(Feature::Fundamental)
        {
            return Err(error("source fundamental attribute requires fundamental"));
        }
        let Some(role) = role else {
            return Err(error(
                "source header attribute requires one actual language role",
            ));
        };
        let allowed = if *attribute == HeaderAttribute::DynIncompatible {
            matches!(
                role,
                LangItem::Sized
                    | LangItem::Destruct
                    | LangItem::Unsize
                    | LangItem::PointeeTrait
                    | LangItem::DiscriminantKind
            )
        } else {
            matches!(
                role,
                LangItem::Sized | LangItem::MetaSized | LangItem::PointeeSized
            ) && claims
                .iter()
                .any(|claim| matches!(claim, Claim::DenyImplementations))
        };
        if !allowed {
            return Err(error(
                "source header attribute is outside the supported actual-role profile",
            ));
        }
        if flags.contains(*attribute) {
            return Err(error("duplicate source role header attribute"));
        }
        flags.0 |= attribute.bit();
        reserved.steps = reserved.steps.saturating_add(1);
    }
    if let Some(role) = role {
        reserved.header_attributes[role.index()] = flags;
    }
    let trivial_reads = claims
        .iter()
        .filter(|claim| matches!(claim, Claim::TrivialFieldReads))
        .count();
    if trivial_reads != 0 {
        let unit = package.unit(declaration.origin.file);
        if !source.features.has(Feature::RustcAttributes)
            || !matches!(unit.configured.ast.node(declaration.origin.node).kind,
                crate::syntax::ast::Kind::Item(index) if matches!(unit.configured.ast.items[index as usize].kind, crate::syntax::item::ItemKind::Trait { .. }))
        {
            return Err(error(
                "trivial-field-read diagnostics require an actual source trait",
            ));
        }
        if trivial_reads != 1 {
            return Err(error(
                "duplicate trivial-field-read diagnostic on one trait",
            ));
        }
        // The existing header owns the original attribute and its source image.
        // A lint hint neither reserves a unique global trait nor grants a proof.
        reserved.steps = reserved.steps.saturating_add(1);
    }
    if role == Some(LangItem::Unsize) && !flags.contains(HeaderAttribute::DynIncompatible) {
        return Err(error(
            "actual source Unsize requires its pinned dyn restriction",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/capability_markers.rs"]
mod tests;

/// Only actual indexed declaration kinds own source claims; a scoped builtin
/// requires the role already checked on that same node, independent of spelling.
pub(super) fn primary(
    package: &Package,
    key: crate::resolve::unit::NodeKey,
    kind: crate::resolve::model::DefKind,
    namespace: crate::resolve::model::Namespace,
    claims: Option<&[Claim]>,
) -> bool {
    use crate::{
        resolve::model::DefKind,
        syntax::{ast::Kind, item::ItemKind},
    };
    let unit = package.unit(key.file);
    if matches!(unit.configured.ast.node(key.node).kind, Kind::Variant(_)) {
        return kind == DefKind::Variant
            && namespace == crate::resolve::model::Namespace::Value
            && claims.is_some_and(|claims| {
                claims.iter().any(|claim| {
                    matches!(
                        claim,
                        Claim::Language(
                            LangItem::ControlFlowContinue
                                | LangItem::ControlFlowBreak
                                | LangItem::ResultOk
                                | LangItem::ResultErr
                        )
                    )
                })
            });
    }
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return false;
    };
    match (unit.configured.ast.items[index as usize].kind, kind) {
        (ItemKind::DeclarativeMacro { .. }, DefKind::Macro) => claims.is_some_and(|claims| {
            claims
                .iter()
                .any(|claim| matches!(claim, Claim::BuiltinMacro(_)))
        }),
        (ItemKind::Trait { .. }, DefKind::Trait)
        | (ItemKind::Function { .. }, DefKind::Function)
        | (ItemKind::Const { .. }, DefKind::Const)
        | (ItemKind::Static { .. }, DefKind::Static)
        | (ItemKind::Struct { .. }, DefKind::Struct)
        | (ItemKind::Enum { .. }, DefKind::Enum)
        | (ItemKind::TypeAlias { .. }, DefKind::TypeAlias)
        | (ItemKind::Impl { .. }, DefKind::SelfType)
        | (ItemKind::MacroRules { .. }, DefKind::Macro) => true,
        _ => false,
    }
}

fn validate_nonnull_claims(
    package: &Package,
    source: &SourceCore,
    reserved: &mut Reservations,
    declaration: Declaration,
    claims: &[Claim],
) -> Result<(), Error> {
    let error = |message| super::reservations::error(package, declaration.origin, message);
    for claim in claims {
        if !matches!(claim, Claim::NonNullDocumentation | Claim::NeverNullLint) {
            continue;
        }
        if !source.features.has(Feature::RustcAttributes) {
            return Err(error("non-null source attribute requires rustc_attrs"));
        }
        let unit = package.unit(declaration.origin.file);
        let crate::syntax::ast::Kind::Item(index) =
            unit.configured.ast.node(declaration.origin.node).kind
        else {
            return Err(error("non-null source attribute requires an item"));
        };
        let kind = unit.configured.ast.items[index as usize].kind;
        let valid = match claim {
            Claim::NonNullDocumentation => {
                matches!(kind, crate::syntax::item::ItemKind::Struct { .. })
            }
            Claim::NeverNullLint => matches!(kind, crate::syntax::item::ItemKind::Function { .. }),
            _ => false,
        };
        if !valid {
            return Err(error("non-null source attribute has the wrong item target"));
        }
        // Rust88d9: the former is documentary; the latter supplies ptr_nulls lint metadata.
        // Neither changes scalar validity, layout, call effects or const eligibility.
        reserved.steps = reserved.steps.saturating_add(1);
    }
    Ok(())
}
