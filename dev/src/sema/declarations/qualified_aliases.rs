//! Inherent type paths select ordinary aliases without manufacturing projections.
use super::{Checker, DefId, DefKind, Error, NodeKey, Target, Type, Work, regions};
use crate::{
    resolve::model::Namespace,
    sema::{
        associated::OwnerKind,
        methods::{buckets::Bucket, names::Use},
    },
    types::{GenericArgs, subject_head::Head},
};

impl Checker<'_> {
    pub(super) fn charge_inherent_type(&mut self, work: usize, bytes: usize) -> Result<(), Error> {
        if self.report.package.source_core().is_some()
            || self.report.package.source_client().is_some()
        {
            let catalog = self.facts.source_bindings.as_deref().ok_or_else(|| {
                Error::unlocated("inherent alias requires its actual source catalog")
            })?;
            catalog
                .require_provider_report(self.report)
                .map_err(Error::unlocated)?;
            crate::sema::header_attributes::charge_source_retention(&self.facts, work, bytes)
        } else {
            crate::sema::header_attributes::reserve_source_contracts(&mut self.facts, work, bytes)
        }
    }
    fn permits_inherent_type(&self, key: NodeKey) -> Result<bool, Error> {
        if self
            .report
            .package
            .requested_inherent_associated_types()
            .map_err(|error| self.error(key, error))?
        {
            return Ok(true);
        }
        let feature = crate::resolve::bindings::FeatureName::new("inherent_associated_types")
            .map_err(|error| self.error(key, error))?;
        let span = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .node(key.node)
            .span;
        self.report
            .macro_allowances()
            .allows_at(key.file, span, &feature)
            .map_err(|error| self.error(key, error.to_string()))
    }

    pub(super) fn require_inherent_type_feature(&self, key: NodeKey) -> Result<(), Error> {
        if self.permits_inherent_type(key)? {
            return Ok(());
        }
        Err(self.error(
            key,
            "inherent associated types require their actual feature request",
        ))
    }

    pub(super) fn schedule_inherent_type(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<bool, Error> {
        let (prefix, owner) = if let Some(path) = self.report.resolution.associated_paths.get(&key)
        {
            if path.namespace != Namespace::Type || path.suffix.len() != 1 {
                return Ok(false);
            }
            let definition = &self.report.resolution.definitions[path.owner.0 as usize];
            if definition.kind == DefKind::Generic {
                return Ok(false);
            }
            if definition.kind == DefKind::SelfType
                && self.facts.associated.trait_owner(path.owner).is_some()
            {
                return Ok(false);
            }
            (
                NodeKey {
                    node: path.prefix,
                    ..key
                },
                Some(path.owner),
            )
        } else if let Some(path) = self.report.resolution.qualified_paths.get(&key) {
            if path.namespace != Namespace::Type
                || path.trait_path.is_some()
                || path.suffix.len() != 1
                || self
                    .bound_parameter(
                        NodeKey {
                            node: path.ty,
                            ..key
                        },
                        true,
                    )
                    .is_some_and(|id| self.trait_type_prefix(id))
            {
                return Ok(false);
            }
            (
                NodeKey {
                    node: path.ty,
                    ..key
                },
                None,
            )
        } else {
            return Ok(false);
        };
        if !self.permits_inherent_type(key)? {
            return Ok(false);
        }
        if let Some(owner) = owner {
            self.facts
                .node_mut(prefix)
                .ok_or_else(|| Error::at(key.file, 0, "missing alias prefix"))?
                .reference = Some(Target::Definition(owner));
        }
        self.work.push(Work::InherentType(key, context));
        self.node_work(prefix.file, prefix.node, context);
        Ok(true)
    }

    fn inherent_type_path(&self, key: NodeKey) -> Result<super::associated::Path, Error> {
        let (prefix, owner, suffix, scope) =
            if let Some(p) = self.report.resolution.associated_paths.get(&key) {
                (p.prefix, Some(p.owner), p.suffix.as_slice(), p.scope)
            } else if let Some(p) = self.report.resolution.qualified_paths.get(&key) {
                (p.ty, None, p.suffix.as_slice(), p.scope)
            } else {
                return Err(self.error(key, "missing inherent type path"));
            };
        let [(node, name)] = suffix else {
            return Err(self.error(key, "invalid inherent type suffix"));
        };
        let prefix = NodeKey {
            node: prefix,
            ..key
        };
        let ty = self.ty(prefix)?;
        if !matches!(self.facts.types.kind(ty), Some(Type::Adt(..))) {
            return Err(self.error(
                key,
                "inherent associated type requires an actual nominal owner",
            ));
        }
        Ok(super::associated::Path {
            prefix,
            owner,
            suffix: NodeKey { node: *node, ..key },
            name: *name,
            scope,
            namespace: Namespace::Type,
            head: Head::canonical(
                self.facts
                    .types
                    .kind(ty)
                    .ok_or_else(|| self.error(key, "invalid alias owner type"))?,
            ),
        })
    }

    fn trait_type_prefix(&self, id: DefId) -> bool {
        let definition = &self.report.resolution.definitions[id.0 as usize];
        definition.kind == DefKind::Generic
            || (definition.kind == DefKind::SelfType
                && self.facts.associated.trait_owner(id).is_some())
    }

    fn trait_impl_prefix(&self, key: NodeKey) -> Option<NodeKey> {
        let (prefix, owner) = if let Some(path) = self.report.resolution.associated_paths.get(&key)
        {
            (
                NodeKey {
                    node: path.prefix,
                    ..key
                },
                path.owner,
            )
        } else {
            let path = self.report.resolution.qualified_paths.get(&key)?;
            let prefix = NodeKey {
                node: path.ty,
                ..key
            };
            (prefix, self.bound_parameter(prefix, true)?)
        };
        if self
            .report
            .resolution
            .definitions
            .get(owner.0 as usize)?
            .kind
            != DefKind::SelfType
        {
            return None;
        }
        let owner = self
            .facts
            .associated
            .get(self.facts.associated.owner(owner)?)?;
        matches!(
            owner.kind,
            OwnerKind::Impl {
                trait_type: Some(_),
                ..
            }
        )
        .then_some(prefix)
    }

    fn continue_trait_projection(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        if self.schedule_implicit_projection(key, context)? {
            return Ok(());
        }
        Err(self.error(
            key,
            "trait implementation Self lost its ordinary associated projection",
        ))
    }

    pub(super) fn complete_inherent_type(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        use crate::sema::methods::selection::{PathCaller, PathProgress, PathQuery};
        let trait_prefix = self.trait_impl_prefix(key);
        if let Some(prefix) = trait_prefix
            && !matches!(self.facts.types.kind(self.ty(prefix)?), Some(Type::Adt(..)))
        {
            return self.continue_trait_projection(key, context);
        }
        let path = self.inherent_type_path(key)?;
        let pending =
            self.inherent_headers
                .head_dependencies(&self.facts.types, path.head, path.name);
        if !pending.is_empty() {
            self.work.push(Work::InherentType(key, context));
            for owner in pending.into_iter().rev() {
                self.work
                    .push(Work::ImplHeader(owner, super::impl_headers::Phase::Begin));
            }
            return Ok(());
        }
        let bucket = Bucket::lookup(&self.facts.types, path.head, path.name)
            .ok_or_else(|| self.error(key, "missing inherent type owner bucket"))?;
        let identifier = Use::path(self.report, path.suffix, path.scope)?;
        if trait_prefix.is_some()
            && !self
                .facts
                .methods
                .has_inherent_type_name(self.report, &self.facts.associated, bucket, identifier)
                .map_err(|error| self.error(key, error))?
        {
            return self.continue_trait_projection(key, context);
        }
        let query = if let Some(query) = self.method_queries.remove(&key) {
            *query
        } else {
            let environment = self.selection_environment(context);
            let traits = self.selection_trait_assumptions(key, context, environment.as_ref())?;
            let ty = self.ty(path.prefix)?;
            PathQuery::inherent_alias(
                self.report,
                &mut self.facts,
                key,
                ty,
                bucket,
                identifier,
                PathCaller {
                    availability: crate::sema::traits::implementations::Availability::Declarations,
                    environment,
                    traits,
                },
            )?
        };
        let selected = match self.poll_method_query(key, query, Work::InherentType(key, context))? {
            PathProgress::Ready(Some(member)) => member,
            PathProgress::Ready(None) => {
                return Err(self.error(key, "no applicable inherent associated type"));
            }
            PathProgress::Pending(_) => return Ok(()),
        };
        self.check_closed_inherent_type(key, &path, selected)?;
        let growth =
            crate::support::retained::Growth::map::<NodeKey, crate::sema::methods::Qualified>(
                self.facts.methods.qualified.len(),
                self.facts.methods.qualified.capacity(),
                usize::from(!self.facts.methods.qualified.contains_key(&key)),
            );
        self.charge_inherent_type(growth.work.saturating_add(1), growth.bytes)?;
        self.retain_selected_path(key, &path, selected, identifier)?;
        self.work
            .push(Work::InherentTypeResult(key, selected, context));
        self.work.push(Work::Definition(selected, false));
        Ok(())
    }

    fn check_closed_inherent_type(
        &self,
        key: NodeKey,
        path: &super::associated::Path,
        member: DefId,
    ) -> Result<(), Error> {
        let owner = self
            .facts
            .associated
            .member_owner(member)
            .ok_or_else(|| self.error(key, "missing alias owner"))?;
        let actual = self
            .facts
            .associated
            .owner(member)
            .and_then(|id| self.facts.associated.get(id))
            .ok_or_else(|| self.error(key, "missing actual inherent alias declaration"))?;
        if self
            .report
            .resolution
            .definitions
            .get(member.0 as usize)
            .is_none_or(|d| d.kind != DefKind::TypeAlias || d.namespace != Namespace::Type)
            || actual.self_definition != owner
            || !matches!(
                actual.kind,
                OwnerKind::Impl {
                    trait_type: None,
                    ..
                }
            )
            || self.facts.associated.type_definition(owner, member) != Some(member)
        {
            return Err(self.error(
                key,
                "inherent alias member does not match its actual declaration owner",
            ));
        }
        let schema = self
            .facts
            .associated
            .schema(owner)
            .ok_or_else(|| self.error(key, "missing alias owner schema"))?;
        if schema.captures != 0
            || !schema.bounds.is_empty()
            || [owner, member].into_iter().any(|id| {
                self.facts
                    .generics
                    .parameters
                    .get(&id)
                    .is_some_and(|v| !v.is_empty())
                    || self
                        .facts
                        .generics
                        .predicates
                        .get(&id)
                        .is_some_and(|v| !v.is_empty())
            })
            || crate::sema::lifetimes::arguments(self.report, path.suffix, false)?.is_some()
        {
            return Err(self.error(key, "generic or conditional inherent associated type lookup requires its selected owner proof"));
        }
        if schema.self_type != self.ty(path.prefix)? {
            return Err(self.error(key, "inherent associated type owner application differs"));
        }
        Ok(())
    }

    pub(super) fn complete_inherent_type_result(
        &mut self,
        key: NodeKey,
        member: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        let template = self.facts.definitions[member.0 as usize]
            .ok_or_else(|| self.error(key, "inherent alias definition is incomplete"))?;
        let owner = self
            .facts
            .associated
            .member_owner(member)
            .ok_or_else(|| self.error(key, "missing inherent alias owner"))?;
        let ty = self
            .facts
            .instantiate_associated_definition(
                owner,
                GenericArgs::EMPTY,
                member,
                GenericArgs::EMPTY,
                template,
            )
            .map_err(|e| self.error(key, e))?;
        self.store(key, ty)?;
        self.normalize_annotation(key, ty, context)
    }
}
