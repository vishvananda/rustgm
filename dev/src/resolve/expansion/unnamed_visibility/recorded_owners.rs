//! A warning retains the allocated declaration and actual immutable source owner.
use super::{Error, NodeKey, Package, Producer, Resolution, Tail, Visit};
use crate::{
    expand::marks::{ExpansionId, SyntaxContextId},
    resolve::{
        model::{DefId, DefKind, Namespace, ScopeId},
        package::ModuleId,
    },
    support::source::OriginalImage,
    syntax::{
        ast::Kind,
        item::{Item, ItemKind},
        token_tree::Span,
    },
};
use std::sync::Arc;
pub(super) struct Record {
    pub key: NodeKey,
    pub tail: Option<Tail>,
    pub context: SyntaxContextId,
    definition: DefId,
    module: ModuleId,
    scope: ScopeId,
    owner: Option<NodeKey>,
    introduction: ExpansionId,
    item: Item,
    span: Span,
    image: Arc<OriginalImage>,
}
impl Record {
    pub(super) fn capture(
        package: &Package,
        resolution: &Resolution,
        visit: Visit,
        item: Item,
        tail: Option<Tail>,
    ) -> Result<Self, Error> {
        let module = visit.module;
        let key = NodeKey {
            file: package.modules[module.0 as usize].file,
            node: visit.node,
        };
        let owner = visit.owner.map(|node| NodeKey { node, ..key });
        let introduction = visit.introduction;
        let definition = resolution
            .item_definitions
            .get(&(module, key.node, Namespace::Value))
            .copied()
            .ok_or_else(|| {
                Error::at(key.file, 0, "visibility cause has no allocated definition")
            })?;
        let actual = resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| Error::at(key.file, 0, "visibility definition is missing"))?;
        let unit = package.unit(key.file);
        let context =
            package.syntax_context(key.file, unit.configured.ast.node(key.node).span.start)?;
        Ok(Self {
            key,
            tail,
            context,
            definition,
            module,
            scope: actual.scope,
            owner,
            introduction,
            item,
            span: unit.configured.ast.node(key.node).span,
            image: Arc::clone(unit.source.original_image()),
        })
    }
    pub(super) fn offset(&self) -> u32 {
        self.span.start
    }
    pub(super) fn validate(
        &self,
        package: &Package,
        resolution: &Resolution,
        producer: &Producer,
    ) -> Result<(), Error> {
        let invalid = || {
            Error::at(
                self.key.file,
                self.offset(),
                "unused visibility actual source owner changed",
            )
        };
        let unit = package
            .units
            .get(self.key.file.0 as usize)
            .ok_or_else(invalid)?;
        let actual = resolution
            .definitions
            .get(self.definition.0 as usize)
            .ok_or_else(invalid)?;
        let scope = resolution
            .scopes
            .get(self.scope.0 as usize)
            .ok_or_else(invalid)?;
        let node = unit
            .configured
            .ast
            .nodes
            .get(self.key.node.get() as usize)
            .ok_or_else(invalid)?;
        let Kind::Item(index) = node.kind else {
            return Err(invalid());
        };
        let item = unit
            .configured
            .ast
            .items
            .get(index as usize)
            .ok_or_else(invalid)?;
        if !Arc::ptr_eq(&self.image, unit.source.original_image())
            || actual.kind != DefKind::Const
            || actual.namespace != Namespace::Value
            || actual.node != Some(self.key)
            || actual.scope != self.scope
            || package.names.get(actual.name) != "_"
            || scope.module != self.module
            || scope.associated
            || package
                .modules
                .get(self.module.0 as usize)
                .is_none_or(|module| module.file != self.key.file)
            || resolution
                .item_definitions
                .get(&(self.module, self.key.node, Namespace::Value))
                != Some(&self.definition)
            || resolution
                .introductions
                .definitions
                .get(&self.definition)
                .copied()
                .unwrap_or(ExpansionId(0))
                != self.introduction
            || node.span != self.span
            || item != &self.item
            || !matches!(item.kind, ItemKind::Const { .. })
            || item
                .name
                .is_none_or(|name| unit.spelling(name).ok() != Some("_"))
            || unit.configured.active.get(self.key.node.get() as usize) != Some(&true)
            || package.syntax_context(self.key.file, self.offset())? != self.context
        {
            return Err(invalid());
        }
        if let Some(owner) = self.owner {
            let configured = unit
                .owner_indices
                .get(owner.node.get() as usize)
                .copied()
                .flatten()
                .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
                .ok_or_else(invalid)?;
            if owner.file != self.key.file || configured.node != owner.node || !configured.enabled {
                return Err(invalid());
            }
            let mut current = owner.node;
            let mut remaining = unit.configured.ast.nodes.len();
            loop {
                remaining = remaining.checked_sub(1).ok_or_else(invalid)?;
                producer.diagnostic_read(self.key, 0)?;
                match unit
                    .configured
                    .ast
                    .nodes
                    .get(current.get() as usize)
                    .ok_or_else(invalid)?
                    .kind
                {
                    Kind::Attributed { value, .. } => current = value,
                    _ if current == self.key.node => break,
                    _ => return Err(invalid()),
                }
            }
        }
        Ok(())
    }
}
