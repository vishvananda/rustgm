//! Actual lexical containers and macro definition sites, separate from names.
use super::model::{Introductions, ScopeId};
use crate::{
    expand::marks::ExpansionId,
    support::hash::{HashMap, HashSet},
};

/// One cold owner per expanded crate; physical scopes have implicit root ancestry.
#[derive(Default)]
pub(crate) struct ItemScopes {
    introduced: HashMap<ScopeId, ExpansionId>,
    definitions: Vec<ScopeId>,
    /// Original item/statement-macro blocks remain lookup boundaries even when
    /// expansion removes all their declarations. Value-only scopes are not.
    blocks: HashSet<ScopeId>,
}

#[cfg(test)]
#[path = "../../tests/invariants/container_origins.rs"]
mod tests;

/// Observe typed statement categories before any expansion replaces the node.
/// Parenthesized expression tails and opaque expression captures keep their
/// expression category; a brace/semicolon statement macro creates a boundary.
pub(super) fn block_has_items(
    package: &super::package::Package,
    file: super::unit::FileId,
    node: crate::syntax::ast::NodeId,
) -> bool {
    use crate::syntax::ast::{Kind, MacroContext};
    let unit = package.unit(file);
    let ast = &unit.configured.ast;
    let Kind::Block {
        statements, tail, ..
    } = ast.node(node).kind
    else {
        return false;
    };
    ast.list(statements)
        .iter()
        .copied()
        .chain(tail)
        .any(|mut node| {
            loop {
                if !unit.configured.active[node.get() as usize] {
                    return false;
                }
                match ast.node(node).kind {
                    Kind::Attributed { value, .. } | Kind::Statement(value) => node = value,
                    Kind::Item(_)
                    | Kind::Macro {
                        context: MacroContext::Statements,
                        ..
                    } => return true,
                    _ => return false,
                }
            }
        })
}

impl Introductions {
    pub(crate) fn introduce_block(&mut self, scope: ScopeId) {
        self.item_scopes
            .get_or_insert_with(Box::default)
            .blocks
            .insert(scope);
    }

    pub(crate) fn item_block(&self, scope: ScopeId) -> bool {
        self.item_scopes
            .as_ref()
            .is_some_and(|sites| sites.blocks.contains(&scope))
    }

    pub(crate) fn scope(&self, scope: ScopeId) -> ExpansionId {
        self.item_scopes
            .as_ref()
            .and_then(|sites| sites.introduced.get(&scope))
            .copied()
            .unwrap_or(ExpansionId(0))
    }

    pub(crate) fn introduce_scope(&mut self, scope: ScopeId, expansion: ExpansionId) {
        if expansion.0 != 0 {
            self.item_scopes
                .get_or_insert_with(Box::default)
                .introduced
                .insert(scope, expansion);
        }
    }

    /// Publish the lexical site beside its actual invocation ancestry. Failed
    /// identities cannot leave a site that a later lookup could mistake for proof.
    pub(crate) fn invocation(
        &mut self,
        expansion: ExpansionId,
        parent: ExpansionId,
        definition: ScopeId,
    ) -> Result<(), &'static str> {
        let count = self
            .item_scopes
            .as_ref()
            .map_or(0, |sites| sites.definitions.len());
        if usize::try_from(expansion.0).ok() != count.checked_add(1) {
            return Err("item scope requires consecutive invocation identities");
        }
        self.ancestry.append(expansion, parent)?;
        self.item_scopes
            .get_or_insert_with(Box::default)
            .definitions
            .push(definition);
        Ok(())
    }

    pub(crate) fn definition_scope(&self, expansion: ExpansionId) -> Result<ScopeId, &'static str> {
        expansion
            .0
            .checked_sub(1)
            .and_then(|index| {
                self.item_scopes
                    .as_ref()?
                    .definitions
                    .get(index as usize)
                    .copied()
            })
            .ok_or("unknown item lookup expansion")
    }
}
