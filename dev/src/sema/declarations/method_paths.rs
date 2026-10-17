//! Qualified inherent paths consume typed owner buckets before body inference.
use super::associated::Path;
use super::{Checker, DefKind, Error, Kind, NodeId, NodeKey, Target, Type, Work, regions};
use crate::{
    resolve::model::{Access, Namespace},
    sema::{associated::OwnerKind, methods::Qualified},
    types::ItemId,
};

impl Checker<'_> {
    pub(super) fn index_inherent_methods(&mut self) -> Result<(), Error> {
        for member in self.facts.associated.members() {
            let owner = self.facts.associated.get(member.owner).ok_or_else(|| {
                Error::at(crate::resolve::unit::FileId(0), 0, "invalid method owner")
            })?;
            let definition = &self.report.resolution.definitions[member.definition.0 as usize];
            if !matches!(definition.kind, DefKind::Function | DefKind::Const) {
                continue;
            }
            if matches!(owner.kind, OwnerKind::Trait { .. }) {
                self.facts.methods.trait_names.insert(definition.name);
                continue;
            }
            if !matches!(
                owner.kind,
                OwnerKind::Impl {
                    trait_type: None,
                    ..
                }
            ) {
                continue;
            }
            let schema = self
                .facts
                .associated
                .schema(member.definition)
                .ok_or_else(|| self.error(owner.source, "missing inherent owner schema"))?;
            let Some(Type::Adt(nominal, _)) = self.facts.types.kind(schema.self_type) else {
                return Err(self.error(owner.source, "missing inherent nominal target"));
            };
            self.facts
                .methods
                .inherent
                .entry((nominal, definition.name))
                .or_default()
                .push(member.definition);
        }
        Ok(())
    }

    pub(super) fn schedule_method_path(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<bool, Error> {
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
        if let Some(variant) =
            self.enum_variant(key, path.nominal, path.name, path.namespace, path.scope)?
        {
            if path.owner.is_some() {
                return Ok(false);
            }
            self.select_qualified_variant(key, path, variant)?;
            return Ok(true);
        }
        if path.namespace == Namespace::Value
            && self
                .facts
                .methods
                .inherent
                .contains_key(&(path.nominal, path.name))
        {
            self.work
                .push(super::Work::InherentMethod(key, *path, context));
            self.schedule_method_prefix(path.prefix, path.owner, context, steps, limit, scan)?;
            return Ok(true);
        }
        self.schedule_implicit_method(key, context, steps, limit, scan)
    }

    fn completed_method_path(&self, key: NodeKey) -> bool {
        self.facts.methods.qualified.contains_key(&key)
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
        let Type::Adt(nominal, _) = self
            .facts
            .types
            .kind(self.facts.definitions[path.owner.0 as usize]?)?
        else {
            return None;
        };
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
            nominal,
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
        let nominal = self.nominal_prefix(prefix)?;
        Some(Path {
            prefix,
            suffix,
            owner: None,
            nominal,
            name,
            scope,
            namespace,
        })
    }

    fn nominal_prefix(&self, mut key: NodeKey) -> Option<ItemId> {
        loop {
            let fact = self.facts.node(key)?;
            let ty = fact.ty.or_else(|| match fact.reference {
                Some(Target::Definition(id)) => self.facts.definitions[id.0 as usize],
                _ => None,
            });
            if let Some(Type::Adt(item, _)) = ty.and_then(|ty| self.facts.types.kind(ty)) {
                return Some(item);
            }
            key.node = match self.kind(key) {
                Kind::Type {
                    flavor: crate::syntax::ast::TypeFlavor::Path,
                    elements,
                } => {
                    let [node] = self
                        .report
                        .package
                        .unit(key.file)
                        .configured
                        .ast
                        .list(elements)
                    else {
                        return None;
                    };
                    *node
                }
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
        let candidates = self
            .facts
            .methods
            .inherent
            .get(&(path.nominal, path.name))
            .ok_or_else(|| self.error(key, "missing queued inherent candidates"))?;
        let unique = if let [method] = candidates.as_slice() {
            Some(*method)
        } else {
            None
        };
        let prefix = path.prefix;
        let has_traits = self.facts.methods.trait_names.contains(&path.name);
        let method = if let Some(method) = unique.filter(|_| !has_traits) {
            method
        } else {
            self.selection_prefix(prefix, path.owner, context)?;
            let environment = self.selection_environment(key, context);
            let traits = self.selection_trait_assumptions(key, context, environment.as_ref())?;
            let selected = crate::sema::methods::selection::choose(
                self.report,
                &mut self.facts,
                key,
                prefix,
                (path.nominal, path.name),
                path.scope,
                crate::sema::methods::selection::Caller {
                    environment: environment.as_ref(),
                    traits: traits.as_deref(),
                },
            )?;
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
        let definition = &self.report.resolution.definitions[method.0 as usize];
        let module = self.report.resolution.scopes[path.scope.0 as usize].module;
        if let Access::Within(ancestor) = definition.access
            && !self.report.package.is_descendant(module, ancestor)
        {
            return Err(self.error(key, "inherent method is private"));
        }
        let implementation = self
            .facts
            .associated
            .owner(method)
            .and_then(|id| self.facts.associated.get(id))
            .ok_or_else(|| self.error(key, "missing selected method owner"))?
            .self_definition;
        self.facts.methods.qualified.insert(
            key,
            Qualified {
                prefix,
                nominal: path.nominal,
                implementation,
                annotation: path.owner.is_none(),
            },
        );
        self.facts
            .node_mut(key)
            .ok_or_else(|| Error::at(key.file, 0, "missing method path fact"))?
            .reference = Some(Target::Definition(method));
        self.work
            .push(super::Work::MethodArguments(key, path.suffix));
        self.schedule_item_arguments(path.suffix, method, context, steps, limit, scan)
    }
}
