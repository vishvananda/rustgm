//! Sparse overwrite journals retain original syntax for demanded module instances.
use super::Unit;
use crate::support::hash::HashMap;
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
use std::num::NonZeroU32;
#[cfg(test)]
#[path = "../../../tests/invariants/retirement_snapshots.rs"]
mod tests;
#[repr(transparent)]
struct SavedFlags(u8);
impl SavedFlags {
    fn new(active: bool, owner_enabled: bool) -> Self {
        Self(u8::from(active) | (u8::from(owner_enabled) << 1))
    }
    fn active(&self) -> bool {
        self.0 & 1 != 0
    }
    fn owner_enabled(&self) -> bool {
        self.0 & 2 != 0
    }
}
struct Saved {
    node: Node,
    flags: SavedFlags,
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
            overwritten: HashMap::default(),
        }
    }
    pub(super) fn needs_retirement_save(
        &self,
        id: NodeId,
        active: bool,
        owner_enabled: bool,
    ) -> bool {
        (active || owner_enabled)
            && (id.get() as usize) < self.syntax.nodes
            && !self.overwritten.contains_key(&id)
    }
    pub(super) fn retirement_row_bytes() -> usize {
        crate::support::retained::new_map::<NodeId, Saved>(1)
    }
    pub(super) fn reserve_retirements(&mut self, count: usize) -> Result<(), &'static str> {
        self.overwritten
            .try_reserve(count)
            .map_err(|_| "Test original journal allocation failed")
    }
    pub(super) fn save(
        &mut self,
        id: NodeId,
        node: Node,
        active: bool,
        owner: Option<NonZeroU32>,
        owner_enabled: bool,
    ) {
        if (id.get() as usize) < self.syntax.nodes {
            self.overwritten.entry(id).or_insert(Saved {
                node,
                flags: SavedFlags::new(active, owner_enabled),
                owner,
            });
        }
    }
}
impl Unit {
    #[cfg(test)]
    pub(crate) fn original_layout_bytes() -> (usize, usize) {
        (size_of::<Original>(), size_of::<Saved>())
    }
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
        let mut owners = self
            .configured
            .owners
            .get(..original.owners)
            .ok_or_else(|| error(0, "invalid original attribute-owner prefix"))?
            .to_vec();
        for (id, saved) in &original.overwritten {
            *ast.nodes
                .get_mut(id.get() as usize)
                .ok_or_else(|| error(0, "saved original node is outside its prefix"))? = saved.node;
            *active
                .get_mut(id.get() as usize)
                .ok_or_else(|| error(0, "saved original active flag is outside its prefix"))? =
                saved.flags.active();
            if let Some(owner) = saved.owner {
                owners
                    .get_mut(owner.get() as usize - 1)
                    .filter(|owner| owner.node == *id)
                    .ok_or_else(|| error(0, "invalid saved original attribute owner"))?
                    .enabled = saved.flags.owner_enabled();
            }
            *owner_indices
                .get_mut(id.get() as usize)
                .ok_or_else(|| error(0, "saved original owner index is outside its prefix"))? =
                saved.owner;
        }
        let source = self.source.physical_copy();
        ast.fresh_defaultness_source(self.source.original_image(), source.original_image())?;
        let configured = ConfiguredSyntax {
            ast,
            cfg_uses: self.configured.cfg_uses.as_ref().and_then(|uses| {
                // Every installation appends effects and their uses together.
                // A fresh instance copies only the original prefix, without
                // traversing feature uses from prior expansion instances.
                uses.original_prefix(original.effects)
            }),
            owners,
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
            source,
            include_context: self.include_context,
            configured,
            literals,
            owner_indices,
            origins: Vec::new(),
            original,
            edition: self.edition,
        })
    }
}
