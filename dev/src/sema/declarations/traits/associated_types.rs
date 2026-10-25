//! Required type declarations and their implementation contracts share resolved identities.
mod conformance;
mod families;
mod parameters;
mod required;
use super::super::generics::bounds::Syntax;
use super::super::{Checker, DefId, Error, ItemId, Kind, NodeKey, Type};
use crate::{
    resolve::model::DefKind,
    sema::associated::{OwnerKind, SourceOwner as Owner, TypeContract},
    syntax::{
        ast::List,
        item::{ItemKind, Visibility},
    },
    types::{
        parameters::{clauses::Clause, predicates::Bound},
        projection,
    },
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
            let Some(owner) = self
                .facts
                .associated
                .source_at(member.owner.0 as usize)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
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
            let context = self.associated_type_context(member.definition, key)?;
            let (mut sized, bounds) =
                self.associated_type_bounds(member.definition, key, bounds, context)?;
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
        self.reserve_family_parameters(false, steps, limit)?;
        let mut pending = Vec::new();
        for index in 0..self.facts.associated.members().len() {
            let member = self.facts.associated.members()[index];
            let definition = &self.report.resolution.definitions[member.definition.0 as usize];
            if definition.kind != DefKind::TypeAlias {
                continue;
            }
            let Some(owner) = self
                .facts
                .associated
                .source_at(member.owner.0 as usize)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
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
                OwnerKind::ImportedImpl(_) => {
                    return Err(self.error(
                        key,
                        "source associated type cannot use an imported syntax owner",
                    ));
                }
            };
            self.facts
                .generics
                .parents
                .insert(member.definition, parent);
        }
        for (id, key) in pending {
            self.complete_associated_result_bounds(id, key, steps, limit)?;
        }
        Ok(())
    }

    fn complete_associated_result_bounds(
        &mut self,
        id: DefId,
        key: NodeKey,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let context = self.associated_type_context(id, key)?;
        self.schedule_bounds(id, context)?;
        self.drain(steps, limit)?;
        if let Some(family) = self.facts.associated.families.get_mut(&id) {
            family.requirements.get_or_insert_with(Vec::new);
        }
        let inputs = self
            .facts
            .associated
            .families
            .get(&id)
            .and_then(|family| family.requirements.as_ref())
            .map_or(0, Vec::len);
        let predicates = self
            .facts
            .generics
            .predicates
            .get(&id)
            .map_or(&[][..], Vec::as_slice);
        let results = predicates
            .get(inputs..)
            .ok_or_else(|| self.error(key, "missing associated input predicates"))?;
        let contract = self
            .facts
            .associated
            .types
            .get_mut(&id)
            .ok_or_else(|| Error::at(key.file, 0, "missing associated type contract"))?;
        contract.bounds.extend(results.iter().map(|p| p.value));
        Ok(())
    }

    /// A result bound must itself be well-formed even without an implementation.
    /// Keep its typed premises for the ordinary declaration and lifetime gates.
    pub(in crate::sema::declarations) fn validate_associated_type_bounds(
        &mut self,
    ) -> Result<(), Error> {
        let mut steps = 0;
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
            self.validate_bound_projection_requirements(id, &mut steps, 1_000_000)?;
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
        let Some((trait_application, _)) = predicate.bound.trait_parts() else {
            return Err(self.error(key, "missing associated type trait application"));
        };
        let arguments = self.template_arguments(id, key)?;
        let projection = self.intern(
            key,
            Type::Projection(projection::Application {
                item: ItemId(id.0),
                subject: predicate.ty,
                trait_application,
                arguments,
            }),
        )?;
        let mut predicates: Vec<_> = bounds
            .builtin
            .into_iter()
            .map(|bound| {
                Clause::Predicate(crate::types::parameters::predicates::Predicate {
                    ty: projection,
                    bound,
                })
            })
            .collect();
        if let Some(contracts) = &self.facts.generics.traits {
            for clause in contracts.associated_bounds(ItemId(id.0)) {
                predicates.push(*clause);
            }
        }
        if !bounds.syntax.is_empty() {
            // Input clauses precede result clauses in the one bound worklist.
            self.bound_syntax
                .entry(id)
                .or_default()
                .extend(bounds.syntax);
        }
        self.facts.associated.types.insert(
            id,
            TypeContract {
                projection,
                bounds: predicates,
            },
        );
        let parent = self.owner_binders[&owner.self_definition];
        self.owner_binders.entry(id).or_insert(parent);
        self.complete(id, key, projection)
    }

    fn associated_type_bounds(
        &mut self,
        id: DefId,
        key: NodeKey,
        bounds: List,
        context: super::super::regions::Context,
    ) -> Result<(bool, Bounds), Error> {
        use super::super::generics::bounds::Written;
        let nodes = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .list(bounds)
            .to_vec();
        let mut required = Vec::new();
        let mut syntax = Vec::new();
        let mut relaxed = false;
        for node in nodes {
            let key = NodeKey { node, ..key };
            if self.lifetime_argument(key) {
                syntax.push(Syntax::associated(key, id, Written::Outlives(key), None));
                continue;
            }
            let Kind::Bound {
                optional,
                constness,
                quantified,
                binders,
                value,
            } = self.kind(key)
            else {
                return Err(self.error(key, "invalid associated type bound"));
            };
            let scope = self.bound_scope(key, quantified.then_some(binders), context, false)?;
            let value = NodeKey { node: value, ..key };
            if self.lifetime_argument(value) {
                if optional {
                    return Err(self.error(key, "associated lifetime bounds cannot be relaxed"));
                }
                syntax.push(Syntax::associated(key, id, Written::Outlives(value), scope));
                continue;
            }
            let written = self.qualified_trait_bound(value, constness)?;
            let scope = self.call_bound_scope(key, written, scope, context)?;
            if matches!(written, Written::User { .. }) && !optional {
                syntax.push(Syntax::associated(key, id, written, scope));
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
            } else {
                if scope.is_some() {
                    syntax.push(Syntax::associated(key, id, written, scope));
                }
                if !required.contains(&bound) {
                    required.push(bound);
                }
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
