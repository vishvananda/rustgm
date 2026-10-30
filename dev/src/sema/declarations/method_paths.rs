//! Qualified inherent paths consume typed owner buckets before body inference.
use super::associated::Path;
use super::{Checker, DefKind, Error, Kind, NodeId, NodeKey, Target, Work, regions};
use crate::{
    resolve::model::{Access, Namespace},
    sema::methods::buckets::Bucket,
    sema::methods::{PrefixKind, Qualified},
    types::subject_head::Head,
};

impl Checker<'_> {
    pub(super) fn schedule_method_path(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<bool, Error> {
        if self.schedule_primitive_constant(key, context, steps, limit, scan)? {
            return Ok(true);
        }
        // Every method path form below starts from one of these resolution entries.
        let resolution = &self.report.resolution;
        if !resolution.qualified_paths.contains_key(&key)
            && !resolution.associated_paths.contains_key(&key)
        {
            return Ok(false);
        }
        if self.schedule_trait_callable(key, context, steps, limit, scan)? {
            return Ok(true);
        }
        if let Some(path) = self.report.resolution.qualified_paths.get(&key) {
            if path.namespace != Namespace::Value
                || path.trait_path.is_some()
                || path.suffix.len() != 1
            {
                return Ok(false);
            }
            if self.completed_method_path(key) {
                return Ok(true);
            }
            let prefix = NodeKey {
                node: path.ty,
                ..key
            };
            self.work.push(super::Work::MethodPrefix(key, context));
            self.schedule_method_prefix(prefix, None, context, steps, limit, scan)?;
            return Ok(true);
        }
        if self.schedule_slice_alias(key, context, steps, limit, scan)? {
            return Ok(true);
        }
        let path = self.named_method_path(key);
        let Some(path) = path else {
            return self.schedule_implicit_method(key, context, steps, limit, scan);
        };
        self.schedule_nominal_path(key, &path, context, steps, limit, scan)
    }

    pub(super) fn complete_method_prefix(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<(), Error> {
        if self.complete_primitive_constant(key)? {
            return Ok(());
        }
        if self.complete_slice_prefix(key, context, steps, limit, scan)? {
            return Ok(());
        }
        if let Some(path) = self.explicit_method_path(key) {
            self.schedule_nominal_path(key, &path, context, steps, limit, scan)?;
        } else {
            self.schedule_implicit_method(key, context, steps, limit, scan)?;
        }
        Ok(())
    }

    fn schedule_nominal_path(
        &mut self,
        key: NodeKey,
        path: &Path,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<bool, Error> {
        if self.completed_method_path(key) {
            return Ok(true);
        }
        self.method_path_segments(key, path.prefix, path.suffix)?;
        if let Head::Nominal(nominal) = path.head
            && let Some(variant) = self.enum_variant(
                key,
                path.suffix,
                nominal,
                path.name,
                path.namespace,
                path.scope,
            )?
        {
            if path.owner.is_some() {
                return Ok(false);
            }
            self.select_qualified_variant(key, path, variant)?;
            return Ok(true);
        }
        if path.namespace == Namespace::Value
            && let Some(bucket) = Bucket::lookup(&self.facts.types, path.head, path.name)
            && (self
                .inherent_headers
                .contains_head(&self.facts.types, path.head, path.name)
                || self.facts.methods.has_inherent(bucket))
        {
            self.work
                .push(super::Work::InherentMethod(key, *path, context));
            self.schedule_method_prefix(path.prefix, path.owner, context, steps, limit, scan)?;
            return Ok(true);
        }
        self.schedule_implicit_method(key, context, steps, limit, scan)
    }

    fn completed_method_path(&self, key: NodeKey) -> bool {
        (self.facts.methods.qualified.contains_key(&key)
            || self.facts.methods.length_prefix(key).is_some()
            || self.collection_prefix(key).is_some())
            && self.facts.generics.item_uses.contains_key(&key)
    }

    fn named_method_path(&self, key: NodeKey) -> Option<Path> {
        match self.facts.node(key)?.reference? {
            Target::Associated(_) => {}
            Target::Definition(id)
                if matches!(
                    self.report.resolution.definitions[id.0 as usize].kind,
                    DefKind::Function | DefKind::Const
                ) => {}
            _ => return None,
        }
        let path = self.report.resolution.associated_paths.get(&key)?;
        let head = Head::canonical(
            self.facts
                .types
                .kind(self.facts.definitions[path.owner.0 as usize]?)?,
        );
        if path.namespace != Namespace::Value || path.suffix.len() != 1 {
            return None;
        }
        Some(Path {
            prefix: NodeKey {
                node: path.prefix,
                ..key
            },
            suffix: NodeKey {
                node: path.suffix[0].0,
                ..key
            },
            owner: Some(path.owner),
            head,
            name: path.suffix[0].1,
            scope: path.scope,
            namespace: path.namespace,
        })
    }

    fn explicit_method_path(&self, key: NodeKey) -> Option<Path> {
        let path = self.report.resolution.qualified_paths.get(&key)?;
        if path.namespace != Namespace::Value || path.trait_path.is_some() || path.suffix.len() != 1
        {
            return None;
        }
        let prefix = NodeKey {
            node: path.ty,
            ..key
        };
        let suffix = NodeKey {
            node: path.suffix[0].0,
            ..key
        };
        let (name, scope, namespace) = (path.suffix[0].1, path.scope, path.namespace);
        let head = self.inherent_prefix_head(prefix)?;
        Some(Path {
            prefix,
            suffix,
            owner: None,
            head,
            name,
            scope,
            namespace,
        })
    }

    fn inherent_prefix_head(&self, mut key: NodeKey) -> Option<Head> {
        loop {
            let fact = self.facts.node(key)?;
            let ty = fact.ty.or_else(|| match fact.reference {
                Some(Target::Definition(id)) => self.facts.definitions[id.0 as usize],
                _ => None,
            });
            if let Some(kind) = ty.and_then(|ty| self.facts.types.kind(ty)) {
                return Some(Head::canonical(kind));
            }
            key.node = match self.kind(key) {
                Kind::Group(node) | Kind::Attributed { value: node, .. } => node,
                _ => return None,
            };
        }
    }

    pub(super) fn method_path_segments(
        &self,
        key: NodeKey,
        prefix: NodeKey,
        suffix: NodeKey,
    ) -> Result<(), Error> {
        if let crate::syntax::ast::Kind::Path { segments, .. } = self.kind(key) {
            for &node in self
                .report
                .package
                .unit(key.file)
                .configured
                .ast
                .list(segments)
            {
                if node != prefix.node
                    && node != suffix.node
                    && crate::sema::lifetimes::arguments(
                        self.report,
                        NodeKey { node, ..key },
                        false,
                    )?
                    .is_some()
                {
                    return Err(
                        self.error(key, "generic arguments require the type or method segment")
                    );
                }
            }
        }
        Ok(())
    }

    pub(super) fn schedule_method_prefix(
        &mut self,
        prefix: NodeKey,
        owner: Option<super::DefId>,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<(), Error> {
        if let Some(owner) = owner {
            self.facts
                .node_mut(prefix)
                .ok_or_else(|| Error::at(prefix.file, 0, "missing method prefix fact"))?
                .reference = Some(Target::Definition(owner));
            if matches!(
                self.report.resolution.definitions[owner.0 as usize].kind,
                DefKind::TypeAlias | DefKind::SelfType
            ) {
                self.annotation_holes(prefix, steps, limit, scan)?;
                self.work
                    .push(Work::AssociatedPrefix(prefix, owner, context));
                if !self.inferred_expression_alias(prefix, owner, context)? {
                    self.work.push(Work::PathArguments(prefix, owner, context));
                }
            } else if self.facts.generics.parameters.contains_key(&owner) {
                self.schedule_item_arguments(prefix, owner, context, steps, limit, scan)?;
            } else {
                self.node_work(prefix.file, prefix.node, context);
            }
        }
        if owner.is_none() {
            self.annotation_holes(prefix, steps, limit, scan)?;
            self.node_work(prefix.file, prefix.node, context);
        }
        Ok(())
    }

    pub(super) fn complete_method_path(
        &mut self,
        key: NodeKey,
        path: &Path,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<(), Error> {
        let Some(bucket) = Bucket::lookup(&self.facts.types, path.head, path.name) else {
            return self
                .schedule_implicit_method(key, context, steps, limit, scan)
                .map(|_| ());
        };
        let pending =
            self.inherent_headers
                .head_dependencies(&self.facts.types, path.head, path.name);
        if !pending.is_empty() {
            self.work.push(Work::InherentMethod(key, *path, context));
            for owner in pending.into_iter().rev() {
                self.work
                    .push(Work::ImplHeader(owner, super::impl_headers::Phase::Begin));
            }
            return Ok(());
        }
        let Some(candidates) = self
            .facts
            .methods
            .inherent_members(bucket, crate::resolve::model::ItemContext::ROOT)
        else {
            return self
                .schedule_implicit_method(key, context, steps, limit, scan)
                .map(|_| ());
        };
        let identifier =
            crate::sema::methods::names::Use::path(self.report, path.suffix, path.scope)?;
        let contextual = self.facts.methods.has_contextual_inherent(bucket);
        let unique = if let [method] = candidates
            && !contextual
        {
            Some(*method)
        } else {
            None
        };
        let prefix = path.prefix;
        let has_traits = self.facts.methods.trait_names.contains(&path.name);
        let method = if let Some(method) =
            unique.filter(|_| !has_traits && matches!(path.head, Head::Nominal(_)))
        {
            method
        } else {
            use crate::sema::methods::selection::{PathCaller, PathProgress, PathQuery};
            let query = if let Some(query) = self.method_queries.remove(&key) {
                *query
            } else {
                self.selection_prefix(prefix, path.owner, context)?;
                let environment = self.selection_environment(context);
                let traits =
                    self.selection_trait_assumptions(key, context, environment.as_ref())?;
                PathQuery::inherent(
                    self.report,
                    &mut self.facts,
                    key,
                    prefix,
                    bucket,
                    identifier,
                    PathCaller {
                        availability:
                            crate::sema::traits::implementations::Availability::Declarations,
                        environment,
                        traits,
                    },
                )?
            };
            let selected = match self.poll_method_query(
                key,
                query,
                Work::InherentMethod(key, *path, context),
            )? {
                PathProgress::Ready(selected) => selected,
                PathProgress::Pending(_) => return Ok(()),
            };
            let Some(method) = selected else {
                return if has_traits {
                    self.schedule_implicit_method(key, context, steps, limit, scan)
                        .map(|_| ())
                } else {
                    Err(self.error(key, "no applicable inherent method for this prefix"))
                };
            };
            method
        };
        self.retain_selected_path(key, path, method, identifier)?;
        self.work
            .push(super::Work::MethodArguments(key, path.suffix));
        self.schedule_item_arguments(path.suffix, method, context, steps, limit, scan)
    }

    /// Publish the selected owner and its typed prefix as one semantic fact.
    fn retain_selected_path(
        &mut self,
        key: NodeKey,
        path: &Path,
        method: super::DefId,
        identifier: crate::sema::methods::names::Use,
    ) -> Result<(), Error> {
        let prefix = path.prefix;
        let implementation = self.selected_inherent_owner(key, method, identifier)?;
        self.facts.methods.qualified.insert(
            key,
            Qualified {
                prefix,
                kind: match (path.owner, path.head) {
                    (Some(_), Head::Nominal(item)) => PrefixKind::Nominal(item),
                    _ => PrefixKind::Annotation,
                },
                implementation,
            },
        );
        self.facts
            .node_mut(key)
            .ok_or_else(|| Error::at(key.file, 0, "missing method path fact"))?
            .reference = Some(Target::Definition(method));
        Ok(())
    }

    fn selected_inherent_owner(
        &self,
        key: NodeKey,
        method: super::DefId,
        identifier: crate::sema::methods::names::Use,
    ) -> Result<super::DefId, Error> {
        let implementation = self
            .facts
            .associated
            .member_owner(method)
            .ok_or_else(|| self.error(key, "missing selected method owner"))?;
        let Some(module) = identifier
            .matching(self.report, implementation, method)
            .map_err(|message| self.error(key, message))?
        else {
            return Err(self.error(key, "no inherent method with this identifier context"));
        };
        let permitted = match self.report.resolution.definitions[method.0 as usize].access {
            Access::Public => true,
            Access::Within(ancestor) => self.report.package.is_descendant(module, ancestor),
            Access::ExternalPrivate => false,
        };
        if !permitted {
            return Err(self.error(key, "inherent method is private"));
        }
        Ok(implementation)
    }
}
