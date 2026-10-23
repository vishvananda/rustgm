//! Actual source compiler declarations remain distinct from bootstrap identities.
mod intrinsic_contracts;
mod layout;
mod sizing;
use crate::{
    resolve::{
        bindings::Declaration, expansion::Report, model::DefId, package::Error, unit::FileId,
    },
    support::{bindings::LangItem, crates::CrateIdentity},
};
pub(super) use intrinsic_contracts::validate as validate_intrinsic;
pub(crate) use layout::Selection;
pub(super) use layout::validate_member as validate_layout_member;
pub(super) use sizing::validate as validate_sizing;
#[cfg(test)]
#[path = "../../tests/invariants/provider_sizing.rs"]
mod tests;

pub(crate) struct Catalog {
    provider: CrateIdentity,
    languages: [Option<Declaration>; 5],
    sizing: Option<[DefId; 3]>,
    intrinsics: crate::support::hash::HashMap<DefId, intrinsic_contracts::Intrinsic>,
}
impl Catalog {
    pub(crate) fn reserve(report: &Report) -> Result<Option<Box<Self>>, Error> {
        let Some(reserved) = report.resolution.source_bindings.as_deref() else {
            if report.package.source_core.is_some() {
                return Err(Error::at(
                    FileId(0),
                    0,
                    "source provider has no reserved declarations",
                ));
            }
            return Ok(None);
        };
        let source = report.package.source_core.as_deref().ok_or_else(|| {
            Error::at(
                FileId(0),
                0,
                "source declarations have no provider configuration",
            )
        })?;
        if source.provider != reserved.provider || source.current != reserved.provider {
            return Err(Error::at(
                FileId(0),
                0,
                "source declaration provider identity changed",
            ));
        }
        for kind in [LangItem::MemSizeConst, LangItem::MemAlignConst] {
            if let Some(declaration) = reserved.language(kind)
                && (declaration.parent.is_none()
                    || report
                        .resolution
                        .definitions
                        .get(declaration.definition.0 as usize)
                        .is_none_or(|d| d.kind != crate::resolve::model::DefKind::Const))
            {
                return Err(crate::sema::model::Declarations::error(
                    report,
                    declaration.origin,
                    "source layout language item requires an actual associated constant",
                ));
            }
        }
        Ok(Some(Box::new(Self {
            provider: reserved.provider.clone(),
            languages: LangItem::ALL.map(|kind| reserved.language(kind)),
            sizing: None,
            intrinsics: reserved
                .intrinsics()
                .map(|entry| {
                    (
                        entry.declaration.definition,
                        intrinsic_contracts::Intrinsic::reserved(entry),
                    )
                })
                .collect(),
        })))
    }
    /// Closure typing demands bindings outside this initial layout-provider catalog.
    /// Extend this gate with validated actual Copy/Fn declarations when those
    /// source roles are implemented; bootstrap identities cannot satisfy it.
    pub(crate) fn require_closure_bindings(&self) -> Result<(), &'static str> {
        if self.sizing.is_none() {
            return Err("source closure typing requires validated source sizing bindings");
        }
        Err("source closure typing requires source Copy and callable bindings not yet implemented")
    }
    pub(crate) fn language(&self, kind: LangItem) -> Option<Declaration> {
        self.languages[kind.index()]
    }
    /// Reached source intrinsics require target-specific runtime selection even
    /// when bodyless declarations contributed no ordinary checked templates.
    pub(crate) fn has_intrinsics(&self) -> bool {
        !self.intrinsics.is_empty()
    }
    /// Reservation is enough to identify an obligation, never to answer it.
    pub(crate) fn sizing_kind(&self, definition: DefId) -> Result<Option<LangItem>, &'static str> {
        let found = [LangItem::MetaSized, LangItem::PointeeSized]
            .into_iter()
            .find(|kind| {
                self.language(*kind)
                    .is_some_and(|d| d.definition == definition)
            });
        if found.is_some() && self.sizing.is_none() {
            return Err("source sizing declarations require completed validation");
        }
        Ok(found)
    }
}

/// The caller already owns selected Builtin evidence for this exact predicate.
/// This validates its declaration family, not a new proof or a method target.
pub(in crate::sema) fn sizing_predicate(
    facts: &crate::sema::model::Declarations,
    predicate: crate::types::parameters::predicates::Predicate,
) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let crate::types::parameters::predicates::Bound::Trait(id) = predicate.bound else {
        return Ok(false);
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or("invalid source sizing application")?;
    if catalog.sizing_kind(DefId(application.item.0))?.is_none() {
        return Ok(false);
    }
    if facts
        .types
        .arguments(application.arguments)
        .is_none_or(|a| a.len() != 0)
    {
        return Err("source sizing application has unexpected arguments");
    }
    Ok(true)
}
