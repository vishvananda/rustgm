//! Required type declarations and their implementation contracts share resolved identities.
mod conformance;
use super::super::generics::bounds::Syntax;
use super::super::{Checker, DefId, Error, ItemId, Kind, NodeKey, Type};
use crate::{
    resolve::model::DefKind,
    sema::associated::{Owner, OwnerKind, TypeContract},
    syntax::{
        ast::List,
        item::{ItemKind, Visibility},
    },
    types::{GenericArgs, parameters::predicates::Bound, projection},
};
pub(super) use conformance::Members;

pub(in crate::sema::declarations) struct Bounds {
    builtin: Vec<Bound>,
    pub(super) syntax: Vec<Syntax>,
}

impl Checker<'_> {
    pub(in crate::sema::declarations) fn reserve_associated_type_bounds(
        &mut self,
    ) -> Result<(), Error> {
        let mut inherited = None;
        for index in 0..self.facts.associated.members().len() {
            let member = self.facts.associated.members()[index];
            let declaration = &self.report.resolution.definitions[member.definition.0 as usize];
            let owner = self.facts.associated.owners()[member.owner.0 as usize];
            let OwnerKind::Trait {
                definition: trait_item,
                ..
            } = owner.kind
            else {
                continue;
            };
            if declaration.kind != DefKind::TypeAlias {
                continue;
            }
            let key = declaration
                .node
                .ok_or_else(|| self.error(owner.source, "missing associated type source"))?;
            let ItemKind::TypeAlias { bounds, .. } = self.item(key)?.kind else {
                return Err(self.error(key, "missing associated type syntax"));
            };
            let (mut sized, bounds) =
                self.associated_type_bounds(member.definition, key, bounds)?;
            let mut copies = crate::types::copy::Properties {
                copy: bounds.builtin.contains(&Bound::Copy),
                clone_: bounds
                    .builtin
                    .iter()
                    .any(|bound| matches!(bound, Bound::Copy | Bound::Clone)),
            };
            for syntax in &bounds.syntax {
                let Some((_, trait_)) = syntax.trait_path() else {
                    continue;
                };
                if inherited.is_none() {
                    inherited = Some(self.inherited_builtins()?);
                }
                if let Some(promises) = inherited.as_ref().and_then(|all| all.get(&trait_)) {
                    sized |= promises.sized;
                    copies.copy |= promises.copies.copy;
                    copies.clone_ |= promises.copies.clone_;
                }
            }
            self.facts
                .types
                .declare_projection(
                    ItemId(member.definition.0),
                    projection::Schema {
                        trait_item: ItemId(trait_item.0),
                        sized,
                        copies,
                    },
                )
                .map_err(|e| self.error(key, e))?;
            self.associated_type_bounds
                .insert(member.definition, bounds);
        }
        Ok(())
    }

    pub(in crate::sema::declarations) fn prepare_associated_types(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let mut pending = Vec::new();
        for index in 0..self.facts.associated.members().len() {
            let member = self.facts.associated.members()[index];
            let definition = &self.report.resolution.definitions[member.definition.0 as usize];
            if definition.kind != DefKind::TypeAlias {
                continue;
            }
            let owner = self.facts.associated.owners()[member.owner.0 as usize];
            let key = definition
                .node
                .ok_or_else(|| self.error(owner.source, "missing associated type source"))?;
            let item = self.item(key)?;
            let ItemKind::TypeAlias { ty, bounds } = item.kind else {
                return Err(self.error(key, "missing associated type syntax"));
            };
            if item.visibility != Visibility::Private {
                return Err(self.error(key, "associated types require inherited visibility"));
            }
            if item.generics.start != item.generics.end
                || item.where_clause.start != item.where_clause.end
            {
                return Err(self.error(key, "associated type parameters and where clauses require advanced generic relationships"));
            }
            let parent = match owner.kind {
                OwnerKind::Trait { definition, .. } => {
                    if ty.is_some() {
                        return Err(self.error(
                            key,
                            "associated type defaults are not supported by the stable profile",
                        ));
                    }
                    self.declare_associated_type(member.definition, key, owner, definition)?;
                    if self.bound_syntax.contains_key(&member.definition) {
                        pending.push((member.definition, key));
                    }
                    definition
                }
                OwnerKind::Impl {
                    trait_type: Some(_),
                    ..
                } => {
                    if bounds.start != bounds.end || ty.is_none() {
                        return Err(self.error(
                            key,
                            "associated type definitions require a type and cannot declare bounds",
                        ));
                    }
                    owner.self_definition
                }
                OwnerKind::Impl {
                    trait_type: None, ..
                } => {
                    return Err(self.error(
                        key,
                        "inherent associated types are not supported by the stable profile",
                    ));
                }
            };
            self.facts
                .generics
                .parents
                .insert(member.definition, parent);
        }
        for (id, key) in pending {
            let context = super::super::regions::Context {
                binder: Some(self.owner_binders[&id]),
                position: super::super::regions::Position::Output,
            };
            self.schedule_bounds(id, context)?;
            self.drain(steps, limit)?;
            if self.bound_environment_pending(id) {
                return Err(self.error(
                    key,
                    "associated result bound arguments require scoped projection obligations",
                ));
            }
            let predicates = self
                .facts
                .generics
                .predicates
                .get(&id)
                .cloned()
                .unwrap_or_default();
            self.facts
                .associated
                .types
                .get_mut(&id)
                .ok_or_else(|| Error::at(key.file, 0, "missing associated type contract"))?
                .bounds
                .extend(predicates.into_iter().map(|predicate| predicate.value));
        }
        Ok(())
    }

    /// A result bound must itself be well-formed even without an implementation.
    /// Keep its typed premises for the ordinary declaration and lifetime gates.
    pub(in crate::sema::declarations) fn validate_associated_type_bounds(
        &mut self,
    ) -> Result<(), Error> {
        for index in 0..self.facts.associated.members().len() {
            let id = self.facts.associated.members()[index].definition;
            if !self.facts.associated.types.contains_key(&id)
                || !self.facts.generics.predicates.contains_key(&id)
            {
                continue;
            }
            let key = self.report.resolution.definitions[id.0 as usize]
                .node
                .ok_or_else(|| {
                    Error::at(
                        crate::resolve::unit::FileId(0),
                        0,
                        "missing associated bound source",
                    )
                })?;
            self.facts
                .generics
                .complete_environment(&mut self.facts.types, id)
                .map_err(|e| self.error(key, e))?;
            let environment = self.facts.generics.environments.get(&id).cloned();
            let mut proof = self.declaration_proof(id, environment);
            self.validate_bounds(id, &mut proof)?;
            self.retain_trait_proof(id, key, &mut proof);
        }
        Ok(())
    }

    fn declare_associated_type(
        &mut self,
        id: DefId,
        key: NodeKey,
        owner: Owner,
        trait_item: DefId,
    ) -> Result<(), Error> {
        let bounds = self
            .associated_type_bounds
            .remove(&id)
            .ok_or_else(|| self.error(key, "missing reserved associated type bounds"))?;
        let predicate = self.facts.generics.trait_self_bounds[&trait_item];
        let Bound::Trait(trait_application) = predicate.bound else {
            return Err(self.error(key, "missing associated type trait application"));
        };
        let projection = self.intern(
            key,
            Type::Projection(projection::Application {
                item: ItemId(id.0),
                subject: predicate.ty,
                trait_application,
                arguments: GenericArgs::EMPTY,
            }),
        )?;
        let mut predicates: Vec<_> = bounds
            .builtin
            .into_iter()
            .map(|bound| crate::types::parameters::predicates::Predicate {
                ty: projection,
                bound,
            })
            .collect();
        if let Some(contracts) = &self.facts.generics.traits {
            predicates.extend_from_slice(contracts.associated_bounds(ItemId(id.0)));
        }
        if !bounds.syntax.is_empty() {
            self.bound_syntax.insert(id, bounds.syntax);
        }
        self.facts.associated.types.insert(
            id,
            TypeContract {
                projection,
                bounds: predicates,
            },
        );
        self.owner_binders
            .insert(id, self.owner_binders[&owner.self_definition]);
        self.complete(id, key, projection)
    }

    fn associated_type_bounds(
        &self,
        id: DefId,
        key: NodeKey,
        bounds: List,
    ) -> Result<(bool, Bounds), Error> {
        use super::super::generics::bounds::Written;
        let ast = &self.report.package.unit(key.file).configured.ast;
        let mut required = Vec::new();
        let mut syntax = Vec::new();
        let mut relaxed = false;
        for &node in ast.list(bounds) {
            let key = NodeKey { node, ..key };
            if self.lifetime_argument(key) {
                syntax.push(Syntax::associated(key, id, Written::Outlives(key)));
                continue;
            }
            let Kind::Bound {
                optional,
                binders,
                value,
            } = self.kind(key)
            else {
                return Err(self.error(key, "invalid associated type bound"));
            };
            if binders.start != binders.end {
                return Err(self.error(
                    key,
                    "higher-ranked associated bounds require advanced generic relationships",
                ));
            }
            let value = NodeKey { node: value, ..key };
            if self.lifetime_argument(value) {
                if optional {
                    return Err(self.error(key, "associated lifetime bounds cannot be relaxed"));
                }
                syntax.push(Syntax::associated(key, id, Written::Outlives(value)));
                continue;
            }
            let written = self.trait_bound(value)?;
            if matches!(written, Written::User { .. }) && !optional {
                syntax.push(Syntax::associated(key, id, written));
                continue;
            }
            let Written::Trait(bound) = written else {
                return Err(self.error(
                    key,
                    "associated trait bounds require projection obligations",
                ));
            };
            if optional {
                if bound != Bound::Sized || relaxed {
                    return Err(self.error(
                        key,
                        "only one relaxed Sized bound is allowed on an associated type",
                    ));
                }
                relaxed = true;
            } else if !required.contains(&bound) {
                required.push(bound);
            }
        }
        if !relaxed && !required.contains(&Bound::Sized) {
            required.push(Bound::Sized);
        }
        // Copy and Clone also entail Sized in this language profile.
        let sized = required
            .iter()
            .any(|b| matches!(b, Bound::Sized | Bound::Copy | Bound::Clone));
        Ok((
            sized,
            Bounds {
                builtin: required,
                syntax,
            },
        ))
    }
}
