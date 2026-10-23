//! Retain complete impl applications; headers alone do not prove an implementation.
mod constants;
mod copying;
mod methods;
mod negative;
mod orphans;
mod readiness;
pub(in crate::sema::declarations) use readiness::Readiness;
mod validation;
use super::super::{Checker, DefId, Error, ItemId, NodeKey, Work};
use crate::{
    resolve::model::DefKind,
    sema::{
        associated::OwnerKind,
        traits::implementations::{Identity, Implementation},
    },
    types::parameters::predicates::{Bound, Predicate},
};
pub(in crate::sema::declarations) use validation::impl_contract_obligations;

impl Checker<'_> {
    pub(in crate::sema::declarations) fn prepare_trait_implementations(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        for index in 0..self.facts.associated.owners().len() {
            let owner = self.facts.associated.owners()[index];
            if matches!(
                owner.kind,
                OwnerKind::Impl {
                    trait_type: Some(_),
                    ..
                }
            ) && self
                .facts
                .destructors
                .self_type(owner.self_definition)
                .is_none()
            {
                self.work.push(Work::ImplHeader(
                    owner.self_definition,
                    super::super::impl_headers::Phase::Begin,
                ));
                self.drain(steps, limit)?;
            }
        }
        if let Some(registry) = &mut self.facts.generics.implementations {
            registry
                .finish_constant_headers()
                .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        }
        Ok(())
    }

    pub(in crate::sema::declarations) fn validate_trait_header(
        &mut self,
        owner: crate::sema::associated::Owner,
        path: NodeKey,
        unsafe_: bool,
        negative: bool,
    ) -> Result<(), Error> {
        if negative
            && !self
                .report
                .package
                .source_core
                .as_deref()
                .is_some_and(crate::resolve::bindings::SourceCore::permits_negative_impls)
        {
            return Err(self.error(
                owner.source,
                "negative trait implementations require trait coherence",
            ));
        }
        let (path, definition) = self.resolved_impl_trait_path(path)?;
        if self
            .facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(crate::support::bindings::LangItem::Freeze))
            .is_some_and(|role| role.definition == definition)
            && !self
                .report
                .package
                .source_core
                .as_deref()
                .is_some_and(crate::resolve::bindings::SourceCore::permits_freeze_impls)
        {
            return Err(self.error(
                owner.source,
                "explicit source Freeze implementations require freeze_impls",
            ));
        }
        if self
            .report
            .resolution
            .source_bindings
            .as_deref()
            .is_some_and(|bindings| bindings.denies_implementation(definition))
        {
            return Err(self.error(
                owner.source,
                "explicit implementation is prohibited by its source trait declaration",
            ));
        }
        let trait_unsafe = self.impl_trait_safety(path, definition)?;
        if if negative {
            unsafe_
        } else {
            trait_unsafe != unsafe_
        } {
            return Err(self.error(
                owner.source,
                "trait and implementation safety declarations disagree",
            ));
        }
        let auto_ = matches!(self.impl_trait_identity(definition), Identity::Auto(_))
            || self
                .facts
                .associated
                .owner(definition)
                .and_then(|id| self.facts.associated.get(id))
                .is_some_and(|owner| matches!(owner.kind, OwnerKind::Trait { auto_: true, .. }));
        if negative && !auto_ {
            return Err(self.error(
                owner.source,
                "negative non-auto trait selection is not implemented",
            ));
        }
        if auto_ {
            self.trait_bound(path)?;
            if self
                .facts
                .associated
                .owner(owner.self_definition)
                .and_then(|id| self.facts.associated.owner_members(id))
                .is_some_and(|members| !members.is_empty())
            {
                return Err(self.error(
                    owner.source,
                    "auto trait implementations cannot declare members",
                ));
            }
            return Ok(());
        }
        let unsupported = self.impl_headers.unsupported.get_or_insert_with(|| {
            self.facts
                .associated
                .members()
                .iter()
                .filter(|member| {
                    !matches!(
                        self.report.resolution.definitions[member.definition.0 as usize].kind,
                        DefKind::Function | DefKind::TypeAlias | DefKind::Const
                    )
                })
                .map(|member| member.owner)
                .collect()
        });
        if [owner.self_definition, definition].into_iter().any(|id| {
            self.facts
                .associated
                .owner(id)
                .is_some_and(|id| unsupported.contains(&id))
        }) {
            return Err(self.error(
                owner.source,
                "trait implementation requires conformance and selection semantics",
            ));
        }
        Ok(())
    }

    pub(in crate::sema::declarations) fn publish_trait_header(
        &mut self,
        owner: crate::sema::associated::Owner,
        path: NodeKey,
        ty: crate::types::TypeId,
        context: super::super::regions::Context,
    ) -> Result<(), Error> {
        let (path, definition) = self.resolved_impl_trait_path(path)?;
        let identity = self.impl_trait_identity(definition);
        let bound = match identity {
            Identity::User(_) => Bound::Trait(self.trait_path(path, definition, ty, context)?),
            Identity::Auto(marker) => {
                if !matches!(self.facts.types.kind(ty), Some(crate::types::Kind::Adt(..))) {
                    return Err(self.error(
                        owner.source,
                        "auto trait implementations require a local nominal type",
                    ));
                }
                Bound::Auto(marker)
            }
        };
        self.finish_impl_regions(owner, context)?;
        let predicate = Predicate { ty, bound };
        self.validate_language_orphan(owner.source, definition, predicate)?;
        self.impl_parameters(owner.self_definition, owner.source, predicate)?;
        self.facts
            .generics
            .implementations
            .get_or_insert_with(Default::default)
            .prepare_source(
                &mut self.facts.types,
                &self.facts.associated,
                identity,
                owner,
                Implementation {
                    owner: owner.self_definition,
                    source: owner.source,
                    predicate,
                    methods: Vec::new(),
                    types: Vec::new(),
                    constants: Vec::new(),
                },
            )
            .map_err(|e| self.error(owner.source, e))
    }

    fn impl_trait_safety(&self, path: NodeKey, definition: DefId) -> Result<bool, Error> {
        if let Identity::Auto(marker) = self.impl_trait_identity(definition) {
            use crate::types::auto_traits::AutoTrait;
            return Ok(matches!(marker, AutoTrait::Send | AutoTrait::Sync));
        }
        if self.facts.associated.language_origin(definition).is_some() {
            return Ok(false);
        }
        let trait_owner = self
            .facts
            .associated
            .owner(definition)
            .and_then(|id| self.facts.associated.get(id))
            .ok_or_else(|| {
                self.error(
                    path,
                    "trait implementation requires a local trait declaration",
                )
            })?;
        let OwnerKind::Trait { unsafe_, .. } = trait_owner.kind else {
            return Err(self.error(path, "implementation path is not a trait"));
        };
        Ok(unsafe_)
    }

    pub(in crate::sema::declarations) fn impl_trait_identity(&self, definition: DefId) -> Identity {
        match self.report.resolution.builtin_traits.identify(definition) {
            Some(crate::resolve::model::BuiltinTrait::Auto(marker)) => Identity::Auto(marker),
            _ => Identity::User(ItemId(definition.0)),
        }
    }
}
