//! Enum aliases select a variant after their namespace prefix has a typed target.
use super::{Checker, DefId, DefKind, Error, NodeId, NodeKey, Target, Type, Work, regions};
use crate::{
    resolve::{
        model::{Access, Namespace, ScopeId},
        names::NameId,
    },
    types::ItemId,
};

#[derive(Clone, Copy)]
pub(super) struct Path {
    pub prefix: NodeKey,
    pub suffix: NodeKey,
    pub owner: Option<DefId>,
    pub head: crate::types::subject_head::Head,
    pub name: NameId,
    pub scope: ScopeId,
    pub namespace: Namespace,
}

impl Checker<'_> {
    pub(super) fn schedule_alias_variant(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<bool, Error> {
        let Some(Target::Associated(owner)) = self.facts.node(key).and_then(|f| f.reference) else {
            return Ok(false);
        };
        if self.report.resolution.definitions[owner.0 as usize].kind != DefKind::TypeAlias {
            return Ok(false);
        }
        let Some(Type::Adt(item, _)) =
            self.facts.definitions[owner.0 as usize].and_then(|ty| self.facts.types.kind(ty))
        else {
            return Ok(false);
        };
        if self.report.resolution.definitions[item.0 as usize].kind != DefKind::Enum {
            return Ok(false);
        }
        let path = self
            .report
            .resolution
            .associated_paths
            .get(&key)
            .ok_or_else(|| self.error(key, "missing associated path facts"))?;
        if path.suffix.len() != 1 {
            return Err(self.error(key, "enum variant cannot have a further path suffix"));
        }
        // Arguments on a variant reached through an alias are not parameters of
        // that alias. Rust rejects these; only the resolved prefix can supply them.
        let suffix = NodeKey {
            node: path.suffix[0].0,
            ..key
        };
        if crate::sema::lifetimes::arguments(self.report, suffix, false)?.is_some() {
            return Err(self.error(suffix, "alias variant does not accept generic arguments"));
        }
        let prefix = NodeKey {
            node: path.prefix,
            ..key
        };
        self.annotation_holes(prefix, steps, limit, scan)?;
        self.work.push(Work::AliasVariant(key, owner, context));
        if !self.inferred_expression_alias(prefix, owner, context)? {
            self.work.push(Work::PathArguments(prefix, owner, context));
        }
        Ok(true)
    }
    pub(super) fn complete_alias_variant(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        let path = self
            .report
            .resolution
            .associated_paths
            .get(&key)
            .ok_or_else(|| self.error(key, "missing associated path facts"))?;
        let prefix = NodeKey {
            node: path.prefix,
            ..key
        };
        let template = self.facts.definitions[owner.0 as usize]
            .ok_or_else(|| self.error(key, "missing alias template"))?;
        let Some(Type::Adt(item, _)) = self.facts.types.kind(template) else {
            return Err(self.error(key, "missing enum alias target"));
        };
        let target = self
            .enum_variant(
                key,
                NodeKey {
                    node: path.suffix[0].0,
                    ..key
                },
                item,
                path.suffix[0].1,
                path.namespace,
                path.scope,
            )?
            .ok_or_else(|| self.error(key, "unknown enum variant"))?;
        self.complete_associated_prefix(prefix, owner, context)?;
        self.facts
            .node_mut(prefix)
            .ok_or_else(|| Error::at(prefix.file, 0, "missing alias prefix fact"))?
            .reference = Some(Target::Definition(owner));
        self.publish_variant(key, prefix, target)
    }
    pub(super) fn enum_variant(
        &self,
        key: NodeKey,
        suffix: NodeKey,
        item: ItemId,
        name: NameId,
        namespace: crate::resolve::model::Namespace,
        use_scope: ScopeId,
    ) -> Result<Option<DefId>, Error> {
        let definition = &self.report.resolution.definitions[item.0 as usize];
        if definition.kind != DefKind::Enum {
            return Ok(None);
        }
        let scope = definition
            .members
            .ok_or_else(|| self.error(key, "missing enum member scope"))?;
        let module = self.report.resolution.scopes[use_scope.0 as usize].module;
        let offset = self
            .report
            .package
            .unit(suffix.file)
            .configured
            .ast
            .node(suffix.node)
            .span
            .start;
        let context = self.report.package.item_context(suffix.file, offset)?;
        let adjustment = self
            .report
            .adjust_item(DefId(item.0), context, module)
            .map_err(|message| self.error(key, message))?;
        let Some(binding) =
            self.report
                .resolution
                .bindings
                .get_key((scope, name, namespace, adjustment.context))
        else {
            return Ok(None);
        };
        if let Access::Within(ancestor) = binding.access
            && !self
                .report
                .package
                .is_descendant(adjustment.privacy, ancestor)
        {
            return Err(self.error(key, "enum variant is private"));
        }
        Ok(Some(binding.target))
    }
    pub(super) fn select_qualified_variant(
        &mut self,
        key: NodeKey,
        path: &Path,
        target: DefId,
    ) -> Result<(), Error> {
        if crate::sema::lifetimes::arguments(self.report, path.suffix, false)?.is_some() {
            return Err(self.error(
                path.suffix,
                "qualified variant does not accept generic arguments",
            ));
        }
        self.publish_variant(key, path.prefix, target)
    }
    fn publish_variant(
        &mut self,
        key: NodeKey,
        prefix: NodeKey,
        target: DefId,
    ) -> Result<(), Error> {
        self.facts
            .node_mut(key)
            .ok_or_else(|| Error::at(key.file, 0, "missing alias variant fact"))?
            .reference = Some(Target::Definition(target));
        self.facts
            .generics
            .constructor_uses
            .insert(key, crate::sema::generics::ConstructorUse::Prefix(prefix));
        self.constructor(target)
    }
    pub(super) fn complete_associated_prefix(
        &mut self,
        prefix: NodeKey,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        if self.facts.generics.annotation_uses.contains_key(&prefix) {
            return Ok(());
        }
        if !self.partial_generic_path(prefix, owner, context)? {
            let ty = self.path_type(prefix, owner, context)?;
            self.store(prefix, ty)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/variant_contexts.rs"]
mod tests;
