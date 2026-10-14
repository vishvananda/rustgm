//! Enum aliases select a variant after their namespace prefix has a typed target.
use super::{Checker, DefId, DefKind, Error, NodeId, NodeKey, Target, Type, Work, regions};
use crate::resolve::model::Access;

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
        self.work.push(Work::PathArguments(prefix, owner, context));
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
        let scope = self.report.resolution.definitions[item.0 as usize]
            .members
            .ok_or_else(|| self.error(key, "missing enum member scope"))?;
        let binding = self.report.resolution.scopes[scope.0 as usize]
            .bindings
            .get(&(path.suffix[0].1, path.namespace))
            .ok_or_else(|| self.error(key, "unknown enum variant"))?;
        let module = self.report.resolution.scopes[path.scope.0 as usize].module;
        if let Access::Within(ancestor) = binding.access
            && !self.report.package.is_descendant(module, ancestor)
        {
            return Err(self.error(key, "enum variant is private"));
        }
        let target = binding.target;
        if !self.partial_generic_path(prefix, owner, context)? {
            let ty = self.path_type(prefix, owner, context)?;
            self.store(prefix, ty)?;
        }
        self.facts
            .node_mut(prefix)
            .ok_or_else(|| Error::at(prefix.file, 0, "missing alias prefix fact"))?
            .reference = Some(Target::Definition(owner));
        self.facts
            .node_mut(key)
            .ok_or_else(|| Error::at(key.file, 0, "missing alias variant fact"))?
            .reference = Some(Target::Definition(target));
        self.facts.generics.constructor_uses.insert(key, prefix);
        self.constructor(target)
    }
}
