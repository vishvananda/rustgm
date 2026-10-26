//! Trait Self has its own identity; source generic parameter indices remain unchanged.
mod associated_types;
mod implementations;
mod override_identity;
pub(super) use implementations::impl_contract_obligations;
pub(super) use implementations::{
    EarlyImplementations, ImplementationPhase, Readiness as ConstantReadiness,
};
mod promises;
use super::{Checker, Error, ItemId, Kind, NodeKey, regions};
use crate::{
    sema::{associated::OwnerKind, model::Function},
    syntax::item::{ItemKind, Safety, Visibility},
};
pub(super) use associated_types::{Bounds as AssociatedBounds, MetadataPhase};

impl Checker<'_> {
    pub(super) fn prepare_trait_owners(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        self.reserve_alias_owners()?;
        // Reserve every trait Self before elaborating references to other trait headers.
        for index in 0..self.facts.associated.owners().len() {
            let Some(owner) = self
                .facts
                .associated
                .source_at(index)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
            if let OwnerKind::Trait { definition, .. } = owner.kind {
                self.reserve_trait(owner, definition)?;
            }
        }
        // All parameter lists precede bounds, including forward supertrait applications.
        for index in 0..self.facts.associated.owners().len() {
            let Some(owner) = self
                .facts
                .associated
                .source_at(index)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
            let OwnerKind::Trait { definition, .. } = owner.kind else {
                continue;
            };
            let context = regions::Context {
                binder: Some(self.owner_binders[&owner.self_definition]),
                position: regions::Position::Output,
            };
            self.declaration_parameters(definition, owner.source, context)?;
            self.facts
                .generics
                .parameters
                .entry(definition)
                .or_default();
            self.drain(steps, limit)?;
        }
        self.prepare_alias_parameters(steps, limit)?;
        self.reserve_family_parameters(true, steps, limit)?;
        self.reserve_associated_type_bounds()?;
        self.begin_header_names()?;
        if self.header_names_active() {
            for index in 0..self.facts.associated.owners().len() {
                let Some(owner) = self
                    .facts
                    .associated
                    .source_at(index)
                    .map_err(Error::unlocated)?
                else {
                    continue;
                };
                if let OwnerKind::Trait { definition, .. } = owner.kind {
                    self.reserve_trait_application(owner, definition)?;
                }
            }
        }
        self.associated_bindings.begin_trait_headers();
        for index in 0..self.facts.associated.owners().len() {
            let Some(owner) = self
                .facts
                .associated
                .source_at(index)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
            let OwnerKind::Trait { definition, .. } = owner.kind else {
                continue;
            };
            let context = regions::Context {
                binder: Some(self.owner_binders[&owner.self_definition]),
                position: regions::Position::Output,
            };
            let self_bounds = self
                .declared_self_bounds(definition, owner.self_definition)
                .map(|(source, _)| source)
                .collect();
            self.schedule_bounds(definition, context)?;
            self.drain(steps, limit)?;
            self.retain_trait_contract(owner, definition, self_bounds)?;
            self.implicit_trait_bound(owner, definition)?;
        }
        self.end_header_names();
        self.complete_trait_bindings(steps, limit)?;
        if let Some(selected) = self.selected.as_deref_mut() {
            selected.sizing(self.report, &mut self.facts)?;
        }
        crate::sema::compiler_items::validate_sizing(self.report, &mut self.facts)?;
        self.complete_trait_contracts()?;
        self.retain_header_projection_requirements()?;
        self.prepare_pointee_member(steps, limit)?;
        self.complete_alias_headers(steps, limit)?;
        Ok(())
    }

    /// Drop headers consume trait contracts but retain their own conformance path.
    /// Other trait impls still need an elaborated, validated owner before use.
    pub(super) fn validate_trait_impl_owners(&self) -> Result<(), Error> {
        for index in 0..self.facts.associated.owners().len() {
            let Some(owner) = self
                .facts
                .associated
                .source_at(index)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
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
                && !self
                    .facts
                    .generics
                    .implementations
                    .as_ref()
                    .is_some_and(|registry| registry.is_validated(owner.self_definition))
            {
                return Err(self.error(
                    owner.source,
                    "trait implementation requires conformance and selection semantics",
                ));
            }
        }
        Ok(())
    }
    fn reserve_trait(
        &mut self,
        owner: crate::sema::associated::SourceOwner,
        definition: crate::resolve::model::DefId,
    ) -> Result<(), Error> {
        self.reserve_lexical_self(
            owner.source,
            definition,
            owner.self_definition,
            |facts, schema| facts.associated.declare(owner.self_definition, schema),
        )
    }
    fn implicit_trait_bound(
        &mut self,
        owner: crate::sema::associated::SourceOwner,
        definition: crate::resolve::model::DefId,
    ) -> Result<(), Error> {
        self.reserve_trait_application(owner, definition)?;
        let value = self.facts.generics.trait_self_bounds[&definition];
        self.facts
            .generics
            .predicates
            .entry(definition)
            .or_default()
            .push(crate::sema::generics::Predicate {
                source: owner.source,
                value: value.into(),
            });
        Ok(())
    }

    fn reserve_trait_application(
        &mut self,
        owner: crate::sema::associated::SourceOwner,
        definition: crate::resolve::model::DefId,
    ) -> Result<(), Error> {
        use crate::types::{
            parameters::predicates::{Bound, Predicate},
            traits::Application,
        };
        if self
            .facts
            .generics
            .trait_self_bounds
            .contains_key(&definition)
        {
            return Ok(());
        }
        let arguments = self.facts.generics.parameters[&definition]
            .iter()
            .map(|parameter| parameter.argument)
            .collect::<Vec<_>>();
        let arguments = self
            .facts
            .types
            .intern_arguments(&arguments)
            .map_err(|e| self.error(owner.source, e))?;
        let application = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(definition.0),
                arguments,
            })
            .map_err(|e| self.error(owner.source, e))?;
        let ty = self
            .facts
            .associated
            .schema(definition)
            .ok_or_else(|| self.error(owner.source, "missing trait Self schema"))?
            .self_type;
        let value = Predicate {
            ty,
            bound: Bound::Trait(application),
        };
        self.facts
            .generics
            .trait_self_bounds
            .insert(definition, value);
        Ok(())
    }

    pub(super) fn validate_trait_method(&self, function: &Function) -> Result<(), Error> {
        if self
            .facts
            .associated
            .trait_owner(function.definition)
            .is_none()
        {
            return Ok(());
        }
        let item = self.item(function.node)?;
        let ItemKind::Function { flags, .. } = item.kind else {
            return Err(self.error(function.node, "missing trait method syntax"));
        };
        if item.visibility != Visibility::Private || flags.safety == Safety::Safe {
            return Err(self.error(
                function.node,
                "trait methods require inherited visibility and cannot be explicitly safe",
            ));
        }
        if flags.constant {
            return Err(self.error(function.node, "trait methods cannot be const"));
        }
        if function.body.is_some() {
            return Ok(());
        }
        let unit = self.report.package.unit(function.node.file);
        for node in unit.configured.ast.list(function.parameters) {
            if !unit.configured.active[node.get() as usize] {
                continue;
            }
            let key = self.unattributed(NodeKey {
                node: *node,
                ..function.node
            });
            let Kind::Parameter { pattern, .. } = self.kind(key) else {
                return Err(self.error(key, "missing required method parameter"));
            };
            if let Some(receiver) = self.facts.receivers.get(&key) {
                if receiver.mutable {
                    return Err(
                        self.error(key, "required trait receivers cannot have mutable bindings")
                    );
                }
                continue;
            }
            if !matches!(
                self.kind(NodeKey {
                    node: pattern,
                    ..key
                }),
                Kind::Wildcard
                    | Kind::Binding {
                        mutable: false,
                        by_ref: false,
                        subpattern: None,
                        ..
                    }
            ) {
                return Err(self.error(
                    key,
                    "required trait parameters must be identifiers or wildcards",
                ));
            }
        }
        Ok(())
    }
}
