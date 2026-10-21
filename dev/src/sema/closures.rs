//! Typed closure captures and callable contracts; source inference supplies place uses.
pub mod captures;
pub mod imports;

use crate::{
    resolve::{model::DefId, unit::NodeKey},
    syntax::ast::List,
    types::{RegionId, TypeId, closure::ClosureId},
};
use std::collections::HashMap;

/// Source body identity is independent of its enclosing function's declaration.
#[derive(Clone, Copy)]
pub struct Body {
    pub identity: ClosureId,
    pub source: NodeKey,
    pub body: NodeKey,
    pub parameters: List,
    pub signature: TypeId,
    pub owner: Option<DefId>,
    pub universals: u32,
}
#[derive(Default)]
pub struct Bodies {
    pub(crate) inputs: HashMap<NodeKey, Vec<RegionId>>,
    pub(crate) values: Vec<Body>,
    pub(crate) sources: HashMap<NodeKey, usize>,
    identities: Vec<u32>,
}
impl Bodies {
    #[must_use]
    pub fn identity(&self, identity: ClosureId) -> Option<&Body> {
        let index = self.identities.get(identity.0 as usize)?.checked_sub(1)?;
        self.values.get(index as usize)
    }
    pub(crate) fn insert(&mut self, body: Body) -> Result<(), &'static str> {
        let identity = body.identity.0 as usize;
        if self.sources.contains_key(&body.source)
            || self
                .identities
                .get(identity)
                .is_some_and(|&index| index != 0)
        {
            return Err("closure body published twice");
        }
        let index = u32::try_from(self.values.len())
            .ok()
            .and_then(|index| index.checked_add(1))
            .ok_or("too many closure bodies")?;
        self.identities
            .resize(self.identities.len().max(identity + 1), 0);
        self.identities[identity] = index;
        self.sources.insert(body.source, self.values.len());
        self.values.push(body);
        Ok(())
    }
    #[must_use]
    pub fn get(&self, source: NodeKey) -> Option<&Body> {
        self.values.get(*self.sources.get(&source)?)
    }
    #[must_use]
    pub fn all(&self) -> &[Body] {
        &self.values
    }
}
