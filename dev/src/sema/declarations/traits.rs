//! Trait Self has its own identity; source generic parameter indices remain unchanged.
mod associated_types;
mod implementations;
mod promises;
use super::{Checker, Error, ItemId, Kind, NodeKey, Type, regions};
use crate::{
    sema::{
        associated::{OwnerKind, Schema},
        generics::Parameter,
        model::Function,
    },
    syntax::item::{ItemKind, Safety, Visibility},
    types::{
        arguments::Argument,
        parameters::{Parameter as TypeParameter, ParameterKind},
    },
};
pub(super) use associated_types::Bounds as AssociatedBounds;

impl Checker<'_> {
    pub(super) fn prepare_trait_owners(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        // Reserve every trait Self before elaborating references to other trait headers.
        for index in 0..self.facts.associated.owners().len() {
            let owner = self.facts.associated.owners()[index];
            if let OwnerKind::Trait { definition, .. } = owner.kind {
                self.reserve_trait(owner, definition)?;
            }
        }
        // All parameter lists precede bounds, including forward supertrait applications.
        for index in 0..self.facts.associated.owners().len() {
            let owner = self.facts.associated.owners()[index];
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
        self.reserve_associated_type_bounds()?;
        self.begin_header_names()?;
        if self.header_names_active() {
            for index in 0..self.facts.associated.owners().len() {
                let owner = self.facts.associated.owners()[index];
                if let OwnerKind::Trait { definition, .. } = owner.kind {
                    self.reserve_trait_application(owner, definition)?;
                }
            }
        }
        self.associated_bindings.begin_trait_headers();
        for index in 0..self.facts.associated.owners().len() {
            let owner = self.facts.associated.owners()[index];
            let OwnerKind::Trait { definition, .. } = owner.kind else {
                continue;
            };
            let context = regions::Context {
                binder: Some(self.owner_binders[&owner.self_definition]),
                position: regions::Position::Output,
            };
            self.schedule_bounds(definition, context)?;
            self.drain(steps, limit)?;
            self.retain_trait_contract(owner, definition)?;
            self.implicit_trait_bound(owner, definition)?;
        }
        self.end_header_names();
        self.complete_trait_bindings()?;
        self.complete_trait_contracts()?;
        self.retain_header_projection_requirements()?;
        Ok(())
    }

    /// Drop headers consume trait contracts but retain their own conformance path.
    /// Other trait impls still need an elaborated, validated owner before use.
    pub(super) fn validate_trait_impl_owners(&self) -> Result<(), Error> {
        for owner in self.facts.associated.owners() {
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
        owner: crate::sema::associated::Owner,
        definition: crate::resolve::model::DefId,
    ) -> Result<(), Error> {
        let key = owner.source;
        let declared = self.declared_regions(key, self.item(key)?.generics)?;
        let binder = self.regions.binder(None, declared.clone());
        self.regions.associate(binder, owner.self_definition);
        for &parameter in &declared {
            self.regions
                .named(Some(binder), parameter)
                .map_err(|e| self.error(key, e))?;
        }
        let bounds = self.function_bounds(key, binder)?;
        let parameter = self
            .facts
            .types
            .intern_parameter(TypeParameter {
                owner: ItemId(owner.self_definition.0),
                index: 0,
                kind: ParameterKind::Type { sized: false },
            })
            .map_err(|e| self.error(key, e))?;
        let self_type = self.intern(key, Type::Parameter(parameter))?;
        self.facts.generics.parameters.insert(
            owner.self_definition,
            vec![Parameter {
                binding: owner.self_definition,
                argument: Argument::Type(self_type),
                default: None,
            }],
        );
        self.facts
            .generics
            .parents
            .insert(definition, owner.self_definition);
        self.facts
            .associated
            .declare(
                owner.self_definition,
                Schema {
                    self_type,
                    captures: self.regions.count(binder),
                    bounds,
                },
            )
            .map_err(|e| self.error(key, e))?;
        self.owner_binders.insert(owner.self_definition, binder);
        self.complete(owner.self_definition, key, self_type)?;
        self.facts.lifetimes.parameters.insert(definition, declared);
        Ok(())
    }
    fn implicit_trait_bound(
        &mut self,
        owner: crate::sema::associated::Owner,
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
                value,
            });
        Ok(())
    }

    fn reserve_trait_application(
        &mut self,
        owner: crate::sema::associated::Owner,
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
            if !unit.configured.active[node.0 as usize] {
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
