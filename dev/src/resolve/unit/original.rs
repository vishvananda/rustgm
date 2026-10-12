//! Sparse overwrite journals retain original syntax for demanded module instances.
use super::Unit;
use crate::{
    expand::crate_syntax::ConfiguredSyntax,
    lex::token::LexError,
    literal::storage::{Storage, StoredValue},
    syntax::{
        ast::{Node, NodeId},
        token_tree::error,
        transfer::Extent,
    },
};
use std::{collections::HashMap, num::NonZeroU32};
struct Saved {
    node: Node,
    active: bool,
    owner: Option<NonZeroU32>,
}
pub(super) struct Original {
    syntax: Extent,
    owners: usize,
    effects: usize,
    decoded: usize,
    overwritten: HashMap<NodeId, Saved>,
}
impl Original {
    pub(super) fn new(configured: &ConfiguredSyntax<StoredValue>, literals: &Storage) -> Self {
        Self {
            syntax: Extent::new(&configured.ast),
            owners: configured.owners.len(),
            effects: configured.effects.len(),
            decoded: literals.decoded_bytes(),
            overwritten: HashMap::new(),
        }
    }
    pub(super) fn save(&mut self, id: NodeId, node: Node, active: bool, owner: Option<NonZeroU32>) {
        if (id.0 as usize) < self.syntax.nodes {
            self.overwritten.entry(id).or_insert(Saved {
                node,
                active,
                owner,
            });
        }
    }
}
impl Unit {
    /// Construct another module instance from original typed facts. Files are
    /// parsed once; expansion changes in a prior instance cannot escape into it.
    /// Only a requested additional instance copies syntax and literal storage.
    ///
    /// # Errors
    /// Rejects corrupted original arena bounds or stored literal ranges.
    pub fn fresh_instance(&self) -> Result<Self, LexError> {
        let original = &self.original;
        let mut ast = original.syntax.copy(&self.configured.ast)?;
        let mut active = self
            .configured
            .active
            .get(..original.syntax.nodes)
            .ok_or_else(|| error(0, "invalid original active-node prefix"))?
            .to_vec();
        let mut owner_indices = self
            .owner_indices
            .get(..original.syntax.nodes)
            .ok_or_else(|| error(0, "invalid original owner-index prefix"))?
            .to_vec();
        for (id, saved) in &original.overwritten {
            ast.nodes[id.0 as usize] = saved.node;
            active[id.0 as usize] = saved.active;
            owner_indices[id.0 as usize] = saved.owner;
        }
        let configured = ConfiguredSyntax {
            ast,
            owners: self
                .configured
                .owners
                .get(..original.owners)
                .ok_or_else(|| error(0, "invalid original attribute-owner prefix"))?
                .to_vec(),
            active,
            effects: self
                .configured
                .effects
                .get(..original.effects)
                .ok_or_else(|| error(0, "invalid original attribute-effect prefix"))?
                .to_vec(),
        };
        let literals = self
            .literals
            .copy_prefix(original.decoded)
            .map_err(|e| error(0, e))?;
        let original = Original::new(&configured, &literals);
        Ok(Self {
            source: self.source.physical_copy(),
            include_context: self.include_context,
            configured,
            literals,
            owner_indices,
            origins: Vec::new(),
            original,
        })
    }
}
