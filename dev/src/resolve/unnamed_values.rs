//! Free unnamed constants keep declaration identity without a lookup binding.
use super::{DefId, DefKind, Definition, Error, FileId, Namespace, Resolver};
use crate::syntax::{ast::Kind, item::ItemKind};

impl Resolver<'_> {
    pub(in crate::resolve) fn unnamed_constant(
        &self,
        definition: &Definition,
    ) -> Result<bool, Error> {
        if definition.kind != DefKind::Const
            || definition.namespace != Namespace::Value
            || self.package.names.get(definition.name) != "_"
        {
            return Ok(false);
        }
        let invalid = || Error::at(FileId(0), 0, "unnamed constant lost its actual source item");
        let key = definition.node.ok_or_else(invalid)?;
        let scope = self
            .facts
            .scopes
            .get(definition.scope.0 as usize)
            .ok_or_else(invalid)?;
        let unit = self
            .package
            .units
            .get(key.file.0 as usize)
            .ok_or_else(invalid)?;
        let Kind::Item(index) = unit
            .configured
            .ast
            .nodes
            .get(key.node.get() as usize)
            .ok_or_else(invalid)?
            .kind
        else {
            return Err(invalid());
        };
        let item = unit.configured.ast.items.get(index as usize).ok_or_else(invalid)?;
        if scope.associated
            || !matches!(item.kind, ItemKind::Const { .. })
            || item.name.is_none_or(|name| unit.spelling(name).ok() != Some("_"))
            || unit.configured.active.get(key.node.get() as usize) != Some(&true)
        {
            return Err(invalid());
        }
        Ok(true)
    }

    pub(super) fn retain_unnamed_constant(
        &mut self,
        definition: Definition,
    ) -> Result<DefId, Error> {
        if !self.unnamed_constant(&definition)? {
            return Err(Error::at(FileId(0), 0, "unnameable declaration is not a free constant"));
        }
        let id = DefId(u32::try_from(self.facts.definitions.len()).map_err(|_| {
            Error::at(FileId(0), 0, "too many definitions")
        })?);
        self.facts.definitions.push(definition);
        Ok(id)
    }
}
