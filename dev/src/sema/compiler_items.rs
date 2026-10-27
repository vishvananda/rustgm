//! Actual source compiler declarations remain distinct from bootstrap identities.
mod binding_views;
mod calls;
pub(in crate::sema) mod formatting;
pub(in crate::sema) mod phantom;
pub(in crate::sema) use binding_views::Role;
pub(in crate::sema) use calls::Effect as CallEffect;
pub(crate) mod const_payments;
mod copying;
mod explicit_cleanup;
mod glue_calls;
pub(in crate::sema) mod headers;
pub(crate) use calls::check_instance as check_const_instance;
pub(super) use calls::record as record_call_effects;
pub(in crate::sema) use explicit_cleanup::explicit_drop_member;
pub(crate) use glue_calls::{DropGlueSelection, check_const_drop_glue, source_drop_glue};
mod abort_declaration;
mod runtime_panics;
pub(in crate::sema) use runtime_panics::runtime_body as runtime_panic_body;
#[path = "compiler_items/location_intrinsic.rs"]
mod caller_location;
pub(crate) use caller_location::CallerLocationSelection;
mod transmute_contract;
mod transmute_sizes;
pub(crate) use abort_declaration::AbortSelection;
pub(crate) use transmute_contract::TransmuteSelection;
pub(in crate::sema) use transmute_sizes::require_equal as require_transmute_size;
mod compose_pointer;
mod intrinsic_contracts;
mod layout;
mod needs_drop;
pub(in crate::sema) mod pointee;
mod ptr_metadata;
mod tag_intrinsic;
pub(crate) use tag_intrinsic::DiscriminantSelection;
#[path = "compiler_items/logical_tags.rs"]
pub(in crate::sema) mod discriminant;
pub(crate) use discriminant::authenticate as authenticate_discriminant;
mod runtime_checks;
pub(crate) use compose_pointer::RawPartsSelection;
pub(crate) use ptr_metadata::PtrMetadataSelection;
pub(crate) use runtime_checks::RuntimeCheckSelection;
mod select_calls;
mod select_contract;
pub(crate) use copying::{
    copy_decisions, copy_decisions_prepared, copy_declaration, prove_body_requirements,
    reserved_copy_declaration,
};
pub(crate) use select_calls::{Select, check_const_select, prepare_select, source_select};
mod roles;
pub(in crate::sema) mod site_records;
pub(in crate::sema) use roles::admitted_headers::selected_copying;
pub(crate) use roles::callable_role;
pub(crate) use roles::{destruct_predicate, drop_glue_member, freeze_predicate};
pub(crate) use roles::{unsizing_predicate, unsizing_role};
mod binary_contract;
mod operator_contract;
mod sizing;
pub(in crate::sema) mod try_protocol;
pub(in crate::sema) use binary_contract::source_binary;
mod unary_contract;
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
pub(in crate::sema) use unary_contract::{source_neg, source_not};
#[cfg(test)]
#[path = "../../tests/invariants/panic_declarations.rs"]
mod panic_declarations;
#[cfg(test)]
#[path = "../../tests/invariants/slice_headers.rs"]
mod slice_headers;
#[cfg(test)]
#[path = "../../tests/invariants/provider_sizing.rs"]
mod tests;

pub(crate) struct Catalog {
    provider: CrateIdentity,
    bindings: binding_views::Bindings,
    sizing: Option<[DefId; 3]>,
    destruction_registration: Option<DefId>,
    traits: Option<Box<roles::Validated>>,
    call_effects: Option<Box<crate::support::hash::HashMap<DefId, calls::Effect>>>,
    pub(in crate::sema) call_contexts: Option<Box<super::bodies::CallContexts>>,
    intrinsics: crate::support::hash::HashMap<DefId, intrinsic_contracts::Intrinsic>,
    attributes: [crate::resolve::bindings::HeaderAttributes; LangItem::ALL.len()],
    runtime_checks: runtime_checks::RuntimeCheckSession,
}
impl Catalog {
    /// A provider catalog cannot authenticate a different source report or mode.
    pub(crate) fn require_provider_report(&self, report: &Report) -> Result<(), &'static str> {
        self.runtime_checks.require_report(&report.package)?;
        if let binding_views::Bindings::Imported(imported) = &self.bindings {
            return imported.require_report(report, &self.provider);
        }
        let source = report.package.source_core()
            .ok_or("source catalog requires its actual provider report; SourceClient mode is not implemented")?;
        let reserved = report
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("source provider report has no reserved declaration identities")?;
        if source.provider != self.provider
            || source.current != self.provider
            || reserved.provider != self.provider
        {
            return Err("source catalog belongs to another provider report");
        }
        Ok(())
    }
    pub(crate) fn reserve(report: &Report) -> Result<Option<Box<Self>>, Error> {
        if report.package.staged_consumer().is_some() {
            return Err(Error::unlocated(
                "private consumer facts require their owned staged constructor",
            ));
        }
        if report.package.source_client().is_some() {
            return binding_views::client_catalog(report)
                .map(Some)
                .map_err(Error::unlocated);
        }
        let Some(reserved) = report.resolution.source_bindings.as_deref() else {
            if report.package.source_core().is_some() {
                return Err(Error::at(
                    FileId(0),
                    0,
                    "source provider has no reserved declarations",
                ));
            }
            return Ok(None);
        };
        let source = report.package.source_core().ok_or_else(|| {
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
        for kind in [LangItem::FormatArguments, LangItem::FormatArgument] {
            if let Some(declaration) = reserved.language(kind) {
                formatting::require_shape(report, declaration).map_err(|message| {
                    crate::sema::model::Declarations::error(report, declaration.origin, message)
                })?;
            }
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
            bindings: binding_views::Bindings::Provider(
                LangItem::ALL.map(|kind| reserved.language(kind)),
            ),
            sizing: None,
            destruction_registration: None,
            traits: None,
            call_effects: None,
            call_contexts: None,
            intrinsics: reserved
                .intrinsics()
                .map(|entry| {
                    (
                        entry.declaration.definition,
                        intrinsic_contracts::Intrinsic::reserved(entry),
                    )
                })
                .collect(),
            attributes: LangItem::ALL.map(|role| reserved.role_attributes(role)),
            runtime_checks: runtime_checks::RuntimeCheckSession::new(&report.package),
        })))
    }
    pub(in crate::sema) fn reserve_staged(
        report: &Report,
        work: &mut crate::resolve::crates::ReservationWork,
    ) -> Result<Box<Self>, Error> {
        binding_views::staged_catalog(report, work)
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
        match &self.bindings {
            binding_views::Bindings::Provider(languages) => languages[kind.index()],
            binding_views::Bindings::Imported(_) => None,
        }
    }
    pub(crate) fn forces_dyn_incompatible(&self, definition: DefId) -> Result<bool, &'static str> {
        for role in [
            LangItem::Sized,
            LangItem::Destruct,
            LangItem::Unsize,
            LangItem::PointeeTrait,
        ] {
            if self
                .role(role)?
                .is_some_and(|declaration| declaration.definition == definition)
                && self.attributes[role.index()]
                    .contains(crate::resolve::bindings::HeaderAttribute::DynIncompatible)
            {
                if role == LangItem::Sized {
                    if self.sizing.is_none() {
                        return Err("source dyn restriction requires completed sizing headers");
                    }
                } else {
                    self.validated_role(role)?;
                }
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// Reached source intrinsics require target-specific runtime selection even
    /// when bodyless declarations contributed no ordinary checked templates.
    pub(crate) fn has_intrinsics(&self) -> bool {
        !self.intrinsics.is_empty()
    }
    /// Reservation is enough to identify an obligation, never to answer it.
    pub(crate) fn sizing_kind(&self, definition: DefId) -> Result<Option<LangItem>, &'static str> {
        let mut found = None;
        for kind in [LangItem::Sized, LangItem::MetaSized, LangItem::PointeeSized] {
            if self.role(kind)?.is_some_and(|d| d.definition == definition) {
                found = Some(kind);
                break;
            }
        }
        if found.is_some() && self.sizing.is_none() {
            return Err("source sizing declarations require completed validation");
        }
        if let Some(kind) = found {
            self.validated_role(kind)?;
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

#[cfg(test)]
pub(crate) const fn validated_layout_bytes() -> usize {
    roles::layout_bytes()
}
