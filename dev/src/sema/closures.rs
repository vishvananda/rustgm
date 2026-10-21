//! Typed closure captures and callable contracts; source inference supplies place uses.
pub mod captures;

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
}
impl Bodies {
    #[must_use]
    pub fn get(&self, source: NodeKey) -> Option<&Body> {
        self.values.get(*self.sources.get(&source)?)
    }
    #[must_use]
    pub fn all(&self) -> &[Body] {
        &self.values
    }
}
