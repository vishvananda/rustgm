//! A derive list keeps its actual lexical placement beside its configured receipt.
use super::{Call, Error, ExpansionId, NodeKey};
use crate::resolve::{
    model::{Resolution, ScopeId},
    package::{ModuleId, Package},
};
#[cfg(test)]
#[path = "../../../../tests/invariants/localized_derivations.rs"]
mod tests;
#[derive(Clone, Copy)]
pub(super) struct Placement {
    module: ModuleId,
    scope: ScopeId,
    parent: Option<ScopeId>,
    scope_introduction: ExpansionId,
    pub(super) introduction: ExpansionId,
    in_block: bool,
}
impl Placement {
    pub(super) fn capture(
        package: &Package,
        facts: &Resolution,
        call: &Call,
    ) -> Result<Self, Error> {
        package
            .source_core()
            .ok_or_else(|| call.diagnostic("derive requires its actual provider"))?
            .macros
            .charge(8, 0)
            .map_err(|message| call.diagnostic(message))?;
        let frame = facts
            .scopes
            .get(call.visit.path_scope.0 as usize)
            .ok_or_else(|| call.diagnostic("derive lost its actual lexical scope"))?;
        let placement = Self {
            module: call.visit.module,
            scope: call.visit.path_scope,
            parent: frame.parent,
            scope_introduction: facts.introductions.scope(call.visit.path_scope),
            introduction: call.visit.introduction,
            in_block: call.visit.in_block,
        };
        placement
            .validate(
                facts,
                NodeKey {
                    file: call.file,
                    node: call.visit.node,
                },
            )
            .map_err(|message| call.diagnostic(message))?;
        Ok(placement)
    }
    pub(super) fn validate(&self, facts: &Resolution, owner: NodeKey) -> Result<(), &'static str> {
        if facts
            .scopes
            .get(self.scope.0 as usize)
            .is_none_or(|scope| scope.module != self.module || scope.parent != self.parent)
            || facts.node_scopes.get(&(self.module, owner.node)) != Some(&self.scope)
            || facts.introductions.scope(self.scope) != self.scope_introduction
            || self.in_block != facts.introductions.item_block(self.scope)
        {
            return Err("source derive actual lexical owner/scope changed");
        }
        Ok(())
    }
}
