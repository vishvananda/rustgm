//! Actual source compiler declarations remain distinct from bootstrap identities.
mod calls;
mod copying;
mod explicit_cleanup;
pub(crate) use calls::check_instance as check_const_instance;
pub(super) use calls::record as record_call_effects;
pub(in crate::sema) use explicit_cleanup::explicit_drop_member;
mod intrinsic_contracts;
mod layout;
mod needs_drop;
pub(crate) use copying::copy_decisions;
mod roles;
pub(crate) use roles::destruct_predicate;
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
pub(crate) use needs_drop::NeedsDropSelection;
pub(super) use roles::validate as validate_traits;
pub(super) use sizing::validate as validate_sizing;
#[cfg(test)]
#[path = "../../tests/invariants/provider_sizing.rs"]
mod tests;

pub(crate) struct Catalog {
    provider: CrateIdentity,
    languages: [Option<Declaration>; LangItem::ALL.len()],
    sizing: Option<[DefId; 3]>,
    destruction_registration: Option<DefId>,
    traits: Option<Box<roles::Validated>>,
    call_effects: Option<Box<crate::support::hash::HashMap<DefId, calls::Effect>>>,
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
            destruction_registration: None,
            traits: None,
            call_effects: None,
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
    /// Closure formation requires the actual completed source role family.
    /// This declaration gate confers neither an implementation nor call permission.
    pub(crate) fn require_closure_bindings(&self) -> Result<(), &'static str> {
        if self.sizing.is_none() {
            return Err("source closure typing requires validated source sizing bindings");
        }
        for role in [
            LangItem::Copy,
            LangItem::Clone,
            LangItem::CloneFn,
            LangItem::Tuple,
            LangItem::FnOnce,
            LangItem::FnMut,
            LangItem::Fn,
            LangItem::FnOnceOutput,
        ] {
            if self.validated_language(role)?.is_none() {
                return Err(
                    "source closure typing requires actual source Copy and callable declarations",
                );
            }
        }
        if self
            .traits
            .as_ref()
            .is_none_or(|r| r.methods.iter().any(Option::is_none))
        {
            return Err("source closure typing requires completed callable member signatures");
        }
        Ok(())
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

/// This intrinsic trait classifies actual tuple types after its real header is sealed.
pub(crate) fn tuple_predicate(
    facts: &crate::sema::model::Declarations,
    predicate: crate::types::parameters::predicates::Predicate,
) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some((application, crate::support::constness::Constness::Ordinary)) =
        predicate.bound.trait_parts()
    else {
        return Ok(false);
    };
    let application = facts
        .types
        .trait_application(application)
        .ok_or("invalid Tuple trait application")?;
    let Some(declaration) =
        catalog.validated_language(crate::support::bindings::LangItem::Tuple)?
    else {
        return Ok(false);
    };
    Ok(application.item.0 == declaration.definition.0
        && facts
            .types
            .arguments(application.arguments)
            .is_some_and(|args| args.len() == 0)
        && matches!(
            facts.types.kind(predicate.ty),
            Some(crate::types::Kind::Tuple(_))
        ))
}
