//! Shorthand selects a unique declaration application; ordinary normalization proves its use.
use super::{Checker, DefId, Error, NodeKey, Type, Work, regions};
use crate::{
    resolve::{
        model::{DefKind, Namespace},
        names::NameId,
    },
    sema::associated::OwnerKind,
    types::{
        GenericArgs, ItemId,
        parameters::predicates::{Bound, Predicate},
        projection,
    },
};
use std::collections::HashSet;

#[derive(Clone, Copy)]
struct Path {
    prefix: NodeKey,
    owner: DefId,
    name: NameId,
    suffix: NodeKey,
}

impl Checker<'_> {
    fn implicit_projection_path(&self, key: NodeKey) -> Result<Option<Path>, Error> {
        let (prefix, owner, suffix) =
            if let Some(path) = self.report.resolution.associated_paths.get(&key) {
                if path.namespace != Namespace::Type || path.suffix.len() != 1 {
                    return Ok(None);
                }
                (
                    NodeKey {
                        node: path.prefix,
                        ..key
                    },
                    Some(path.owner),
                    path.suffix[0],
                )
            } else if let Some(path) = self.report.resolution.qualified_paths.get(&key) {
                if path.namespace != Namespace::Type
                    || path.trait_path.is_some()
                    || path.suffix.len() != 1
                {
                    return Ok(None);
                }
                let prefix = NodeKey {
                    node: path.ty,
                    ..key
                };
                (prefix, self.bound_parameter(prefix, true), path.suffix[0])
            } else {
                return Ok(None);
            };
        let owner = owner
            .filter(|owner| {
                matches!(
                    self.report.resolution.definitions[owner.0 as usize].kind,
                    DefKind::Generic | DefKind::SelfType
                )
            })
            .ok_or_else(|| {
                self.error(
                    key,
                    "shorthand associated types require a bounded parameter or trait Self",
                )
            })?;
        Ok(Some(Path {
            prefix,
            owner,
            name: suffix.1,
            suffix: NodeKey {
                node: suffix.0,
                ..key
            },
        }))
    }

    pub(in super::super) fn schedule_implicit_projection(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<bool, Error> {
        let Some(path) = self.implicit_projection_path(key)? else {
            return Ok(false);
        };
        if self
            .projection_alias_owner(context)
            .is_some_and(|template| {
                self.report.resolution.definitions[template.owner.0 as usize].kind
                    == DefKind::Generic
            })
        {
            return Err(self.error(key, "generic defaults require explicit trait qualification"));
        }
        if crate::sema::lifetimes::arguments(self.report, path.suffix, false)?.is_some() {
            return Err(self.error(
                path.suffix,
                "associated type arguments require advanced generic relationships",
            ));
        }
        self.work.push(Work::ImplicitProjection(key, context));
        if self.report.resolution.associated_paths.contains_key(&key) {
            self.method_path_segments(key, path.prefix, path.suffix)?;
            self.facts
                .node_mut(path.prefix)
                .ok_or_else(|| Error::at(key.file, 0, "missing shorthand prefix fact"))?
                .reference = Some(crate::resolve::model::Target::Definition(path.owner));
        }
        if self.projection_uses_parameter_bounds(path.owner) {
            self.schedule_projection_bounds(key, path.owner, context, path.name)?;
        }
        self.work.push(Work::Node(path.prefix, false, context));
        Ok(true)
    }

    pub(in super::super) fn complete_implicit_projection(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        let path = self
            .implicit_projection_path(key)?
            .ok_or_else(|| self.error(key, "missing shorthand projection path"))?;
        let mut predicates = if self.projection_uses_parameter_bounds(path.owner) {
            self.projection_bound_candidates(key, context)?
        } else {
            Vec::new()
        };
        if self.report.resolution.definitions[path.owner.0 as usize].kind == DefKind::SelfType {
            predicates.push(self.self_projection_candidate(key, path.owner, context)?);
        }
        if self.header_names_active() {
            return self.schedule_header_projection(key, context, path.name, predicates);
        }
        self.finish_implicit_projection(key, context, predicates)
    }

    pub(super) fn finish_implicit_projection(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        predicates: Vec<Predicate>,
    ) -> Result<(), Error> {
        let path = self
            .implicit_projection_path(key)?
            .ok_or_else(|| self.error(key, "missing completed shorthand path"))?;
        let application = self.lookup_projection_member(key, path.name, predicates)?;
        if application.subject != self.ty(path.prefix)? {
            return Err(self.error(key, "shorthand projection subject does not match its bound"));
        }
        let ty = self.intern(key, Type::Projection(application))?;
        self.normalize_annotation(key, ty, context)
    }

    fn projection_uses_parameter_bounds(&self, owner: DefId) -> bool {
        self.report.resolution.definitions[owner.0 as usize].kind == DefKind::Generic
            || self.facts.associated.trait_owner(owner).is_some()
    }

    fn self_projection_candidate(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
    ) -> Result<Predicate, Error> {
        let kind = self
            .facts
            .associated
            .owner(owner)
            .and_then(|id| self.facts.associated.get(id))
            .ok_or_else(|| self.error(key, "missing shorthand Self owner"))?
            .kind;
        let predicate = match kind {
            OwnerKind::Trait { definition, .. } => self
                .facts
                .generics
                .trait_self_bounds
                .get(&definition)
                .copied(),
            OwnerKind::Impl {
                trait_type: Some(_),
                ..
            } => self
                .facts
                .generics
                .implementations
                .as_ref()
                .and_then(|registry| registry.header(owner))
                .map(|header| header.predicate),
            OwnerKind::Impl {
                trait_type: None, ..
            } => None,
        }
        .ok_or_else(|| {
            self.error(
                key,
                "shorthand Self associated type requires a prepared trait owner",
            )
        })?;
        self.capture_projection_predicate(
            key,
            predicate,
            self.owner_binders.get(&owner).copied(),
            context,
        )
    }

    fn lookup_projection_member(
        &mut self,
        key: NodeKey,
        name: NameId,
        predicates: Vec<Predicate>,
    ) -> Result<projection::Application, Error> {
        let mut selected = None;
        for predicate in predicates {
            if let Some(candidate) = self.projection_member_candidate(key, name, predicate)? {
                if selected.is_some_and(|previous| previous != candidate) {
                    return Err(self.error(key, "ambiguous shorthand associated type"));
                }
                selected = Some(candidate);
            }
        }
        selected
            .ok_or_else(|| self.error(key, "no associated type in the parameter's declared bounds"))
    }

    fn projection_member_candidate(
        &mut self,
        key: NodeKey,
        name: NameId,
        root: Predicate,
    ) -> Result<Option<projection::Application>, Error> {
        if let Some(&cached) = self.projections.lookup.members.get(&(root, name)) {
            return Ok(cached);
        }
        let mut work = vec![root];
        let mut seen = HashSet::new();
        let mut selected = None;
        while let Some(predicate) = work.pop() {
            if !seen.insert(predicate) {
                continue;
            }
            if seen.len() + work.len() > 1_000_000 {
                return Err(self.error(key, "shorthand supertrait work budget exceeded"));
            }
            let Bound::Trait(application) = predicate.bound else {
                continue;
            };
            let trait_item = self
                .facts
                .types
                .trait_application(application)
                .ok_or_else(|| self.error(key, "missing shorthand trait application"))?
                .item;
            let scope = self.report.resolution.definitions[trait_item.0 as usize]
                .members
                .ok_or_else(|| self.error(key, "missing shorthand trait scope"))?;
            if let Some(binding) = self.report.resolution.scopes[scope.0 as usize]
                .bindings
                .get(&(name, Namespace::Type))
                && self.facts.associated.trait_owner(binding.target) == Some(DefId(trait_item.0))
                && self
                    .facts
                    .types
                    .projection_schema(ItemId(binding.target.0))
                    .is_some()
            {
                let candidate = projection::Application {
                    item: ItemId(binding.target.0),
                    subject: predicate.ty,
                    trait_application: application,
                    arguments: GenericArgs::EMPTY,
                };
                if selected.is_some_and(|previous| previous != candidate) {
                    return Err(self.error(key, "ambiguous shorthand associated type"));
                }
                selected = Some(candidate);
            }
            if self.header_names_active() {
                work.extend(self.header_projection_parents(key, name, predicate)?);
                continue;
            }
            let requirements = self
                .facts
                .generics
                .traits
                .as_mut()
                .ok_or_else(|| Error::at(key.file, 0, "missing shorthand trait contracts"))?
                .requirements(&mut self.facts.types, predicate)
                .map_err(|e| self.error(key, e))?;
            for clause in &requirements.implied {
                let implied = clause.ordinary().map_err(|e| self.error(key, e))?;
                if implied.ty == predicate.ty && matches!(implied.bound, Bound::Trait(_)) {
                    work.push(implied);
                }
            }
        }
        self.projections
            .lookup
            .members
            .insert((root, name), selected);
        Ok(selected)
    }
}
