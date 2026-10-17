//! Type-qualified trait lookup preserves the prefix even when names found a direct member.
use super::{Checker, DefId, DefKind, Error, NodeId, NodeKey, Target, regions};
use crate::{
    resolve::{
        model::{Namespace, ScopeId},
        names::NameId,
    },
    sema::{
        generics::TraitUse,
        methods::selection::{self, Caller},
    },
};

struct Path {
    prefix: NodeKey,
    owner: Option<DefId>,
    suffix: NodeKey,
    name: NameId,
    scope: ScopeId,
}

impl Checker<'_> {
    pub(super) fn schedule_implicit_method(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<bool, Error> {
        let Some(path) = self.implicit_method_path(key) else {
            return Ok(false);
        };
        if !self.facts.methods.trait_names.contains(&path.name) {
            return Ok(false);
        }
        if self.facts.generics.trait_uses.contains_key(&key)
            && self.facts.generics.item_uses.contains_key(&key)
        {
            return Ok(true);
        }
        self.method_path_segments(key, path.prefix, path.suffix)?;
        self.work.push(super::Work::ImplicitMethod(key, context));
        self.schedule_method_prefix(path.prefix, path.owner, context, steps, limit, scan)?;
        Ok(true)
    }
    pub(super) fn complete_implicit_method(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<(), Error> {
        let path = self
            .implicit_method_path(key)
            .ok_or_else(|| self.error(key, "missing pending trait path"))?;
        self.selection_prefix(path.prefix, path.owner, context)?;
        let environment = self.selection_environment(key, context);
        let traits = self.selection_trait_assumptions(key, context, environment.as_ref())?;
        let method = selection::choose_trait_path(
            self.report,
            &mut self.facts,
            key,
            path.prefix,
            (path.scope, path.name),
            Caller {
                environment: environment.as_ref(),
                traits: traits.as_deref(),
            },
        )?;
        let owner = self
            .facts
            .associated
            .trait_owner(method)
            .ok_or_else(|| self.error(key, "selected method has no trait owner"))?;
        self.facts.generics.trait_uses.insert(
            key,
            TraitUse {
                owner,
                arguments: None,
                subject: Some(path.prefix),
            },
        );
        self.facts
            .node_mut(key)
            .ok_or_else(|| Error::at(key.file, 0, "missing selected trait method fact"))?
            .reference = Some(Target::Definition(method));
        self.work
            .push(super::Work::MethodArguments(key, path.suffix));
        self.schedule_item_arguments(path.suffix, method, context, steps, limit, scan)?;
        Ok(())
    }
    pub(super) fn complete_method_arguments(
        &mut self,
        key: NodeKey,
        suffix: NodeKey,
    ) -> Result<(), Error> {
        let arguments = self
            .facts
            .generics
            .item_uses
            .remove(&suffix)
            .ok_or_else(|| self.error(key, "missing selected method arguments"))?;
        self.facts.generics.item_uses.insert(key, arguments);
        Ok(())
    }

    fn implicit_method_path(&self, key: NodeKey) -> Option<Path> {
        if let Some(path) = self.report.resolution.qualified_paths.get(&key) {
            if path.namespace != Namespace::Value
                || path.trait_path.is_some()
                || path.suffix.len() != 1
            {
                return None;
            }
            return Some(Path {
                prefix: NodeKey {
                    node: path.ty,
                    ..key
                },
                owner: None,
                suffix: NodeKey {
                    node: path.suffix[0].0,
                    ..key
                },
                name: path.suffix[0].1,
                scope: path.scope,
            });
        }
        let path = self.report.resolution.associated_paths.get(&key)?;
        if path.namespace != Namespace::Value
            || path.suffix.len() != 1
            || !matches!(
                self.report.resolution.definitions[path.owner.0 as usize].kind,
                DefKind::Primitive
                    | DefKind::Generic
                    | DefKind::SelfType
                    | DefKind::TypeAlias
                    | DefKind::Struct
                    | DefKind::Enum
            )
        {
            return None;
        }
        Some(Path {
            prefix: NodeKey {
                node: path.prefix,
                ..key
            },
            owner: Some(path.owner),
            suffix: NodeKey {
                node: path.suffix[0].0,
                ..key
            },
            name: path.suffix[0].1,
            scope: path.scope,
        })
    }
}
