//! Required type declarations and their implementation contracts share resolved identities.
mod conformance;
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

impl Checker<'_> {
    pub(in crate::sema::declarations) fn reserve_associated_type_bounds(
        &mut self,
    ) -> Result<(), Error> {
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
            let (sized, bounds) = self.associated_type_bounds(key, bounds)?;
            self.facts
                .types
                .declare_projection(
                    ItemId(member.definition.0),
                    projection::Schema {
                        trait_item: ItemId(trait_item.0),
                        sized,
                    },
                )
                .map_err(|e| self.error(key, e))?;
            self.associated_type_bounds
                .insert(member.definition, bounds);
        }
        Ok(())
    }

    pub(in crate::sema::declarations) fn prepare_associated_types(&mut self) -> Result<(), Error> {
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
        self.facts
            .associated
            .types
            .insert(id, TypeContract { projection, bounds });
        self.owner_binders
            .insert(id, self.owner_binders[&owner.self_definition]);
        self.complete(id, key, projection)
    }

    fn associated_type_bounds(
        &self,
        key: NodeKey,
        bounds: List,
    ) -> Result<(bool, Vec<Bound>), Error> {
        use super::super::generics::bounds::Written;
        let ast = &self.report.package.unit(key.file).configured.ast;
        let mut required = Vec::new();
        let mut relaxed = false;
        for &node in ast.list(bounds) {
            let key = NodeKey { node, ..key };
            let Kind::Bound {
                optional,
                binders,
                value,
            } = self.kind(key)
            else {
                return Err(self.error(
                    key,
                    "associated lifetime bounds require projection obligations",
                ));
            };
            if binders.start != binders.end {
                return Err(self.error(
                    key,
                    "higher-ranked associated bounds require advanced generic relationships",
                ));
            }
            let Written::Trait(bound) = self.trait_bound(NodeKey { node: value, ..key })? else {
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
        Ok((sized, required))
    }
}
