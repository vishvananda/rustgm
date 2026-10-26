//! Local library availability is distinct from language and const-call permission.
mod attributes;
mod constness;
mod inventory;
mod owners;
use super::{
    model::DefId,
    package::Error,
    unit::{FileId, NodeKey},
};
use crate::support::hash::HashMap;
use std::num::{NonZeroU32, NonZeroUsize};

/// A validated library release annotation, retained independently of callability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Since {
    Current,
    Version { major: u16, minor: u16, patch: u16 },
}
/// Regular API policy; const and provided-body contracts have separate consumers.
#[derive(Debug)]
pub enum Level {
    Stable(Since),
    Unstable {
        issue: Option<NonZeroU32>,
        reason: Option<Box<str>>,
    },
}
impl Level {
    fn unstable(&self) -> bool {
        matches!(self, Self::Unstable { .. })
    }
}
/// One parsed annotation. Shared module instances retain separate effective owners.
#[derive(Debug)]
pub struct Policy {
    pub feature: Box<str>,
    pub level: Level,
    pub origin: NodeKey,
}
impl Policy {
    /// The current producer accepts ASCII identifier feature names; later
    /// metadata must preserve this canonical-symbol contract when it expands.
    /// # Errors
    /// Rejects invalid public-policy identifier spellings.
    pub fn feature_name(&self) -> Result<super::bindings::FeatureName<'_>, &'static str> {
        super::bindings::FeatureName::new(&self.feature)
    }
}
type PolicyId = NonZeroUsize;

/// Cold root requests and their transactionally completed local inventory.
#[derive(Default)]
pub struct Library {
    staged: bool,
    requests: Vec<Box<str>>,
    request_bytes: usize,
    inventory: Option<Inventory>,
}
impl Library {
    pub(super) fn request(&mut self, name: &str) -> Result<(), &'static str> {
        if name == "staged_api" {
            self.staged = true;
            return Ok(());
        }
        self.request_bytes = self
            .request_bytes
            .saturating_add(name.len() + 4 * size_of::<Box<str>>());
        if self.requests.len() >= 100_000 || self.request_bytes > 64 * 1024 * 1024 {
            return Err("library feature requests exceed the profile budget");
        }
        self.requests.push(name.into());
        Ok(())
    }
    #[must_use]
    pub const fn inventory(&self) -> Option<&Inventory> {
        self.inventory.as_ref()
    }
    pub(super) fn publish(&mut self, inventory: Inventory) {
        self.inventory = Some(inventory);
    }
}
/// Completed declaration policies. Numerical IDs belong to this resolved package.
pub struct Inventory {
    policies: Vec<Policy>,
    declarations: Vec<Option<PolicyId>>,
    imports: Vec<Option<PolicyId>>,
    consts: Option<Box<constness::ConstPolicies>>,
    steps: usize,
}
impl Inventory {
    #[must_use]
    pub fn declaration(&self, id: DefId) -> Option<&Policy> {
        let id = self.declarations.get(id.0 as usize).copied().flatten()?;
        self.policies.get(id.get() - 1)
    }
    #[must_use]
    pub fn import(&self, index: usize) -> Option<&Policy> {
        let id = self.imports.get(index).copied().flatten()?;
        self.policies.get(id.get() - 1)
    }
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.steps
    }
}

#[derive(Default)]
pub(super) struct Draft {
    policies: Vec<Policy>,
    attributes: HashMap<NodeKey, PolicyId>,
    import_attributes: HashMap<NodeKey, PolicyId>,
    const_attributes: HashMap<NodeKey, PolicyId>,
    steps: usize,
    bytes: usize,
}
impl Draft {
    fn charge(&mut self, steps: usize, bytes: usize) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(steps);
        self.bytes = self.bytes.saturating_add(bytes);
        if self.steps > 1_000_000 || self.bytes > 64 * 1024 * 1024 {
            return Err(Error::at(
                FileId(0),
                0,
                "library stability inventory exceeds its work or storage budget",
            ));
        }
        Ok(())
    }
    fn inherit(&self, policy: Option<PolicyId>) -> Option<PolicyId> {
        policy.filter(|id| self.policies[id.get() - 1].level.unstable())
    }
}
pub(super) use attributes::collect;
pub(super) use inventory::complete;

#[cfg(test)]
#[path = "../../tests/invariants/local_availability.rs"]
mod tests;
