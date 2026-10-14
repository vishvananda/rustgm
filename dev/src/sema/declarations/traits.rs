//! Trait Self has its own identity; source generic parameter indices remain unchanged.
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

impl Checker<'_> {
    pub(super) fn prepare_trait_owners(
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
                .associated
                .schema(owner.self_definition)
                .is_none()
            {
                return Err(self.error(
                    owner.source,
                    "trait implementation requires conformance and selection semantics",
                ));
            }
            let OwnerKind::Trait { definition, .. } = owner.kind else {
                continue;
            };
            let key = owner.source;
            let declared = self.declared_regions(key, self.item(key)?.generics)?;
            let binder = self.regions.binder(None, declared.clone());
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
            let context = regions::Context {
                binder: Some(binder),
                position: regions::Position::Impl,
            };
            self.declaration_parameters(definition, key, context)?;
            self.drain(steps, limit)?;
            self.schedule_bounds(definition, context)?;
            self.drain(steps, limit)?;
            self.facts
                .generics
                .complete_environment(&self.facts.types, definition)
                .map_err(|e| self.error(key, e))?;
        }
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
