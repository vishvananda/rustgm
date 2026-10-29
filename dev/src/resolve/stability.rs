//! Local library availability is distinct from language and const-call permission.
mod attributes;
pub(in crate::resolve) mod availability_imports;
mod compatibility_notes;
mod constness;
mod exported_scopes;
mod implications;
mod inventory;
mod owners;
mod request_order;
use super::{
    model::DefId,
    package::Error,
    unit::{FileId, NodeKey},
};
use crate::support::hash::HashMap;
pub(crate) use exported_scopes::{Budget as PublicBudget, visit as visit_public_bindings};
use std::num::{NonZeroU32, NonZeroUsize};

/// A validated library release annotation, retained independently of callability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Since {
    Current,
    Version { major: u16, minor: u16, patch: u16 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PolicyKind {
    Regular,
    Const,
}
/// Regular API policy; const and provided-body contracts have separate consumers.
#[derive(Debug)]
pub enum Level {
    Stable(Since),
    StableWithCompatibility {
        since: Since,
        note: Box<ModuleCompatibility>,
    },
    Unstable {
        issue: Option<NonZeroU32>,
        reason: Option<Box<str>>,
    },
}
/// The message is opaque user-facing guidance, never an enabling feature or path.
#[derive(Debug)]
pub struct ModuleCompatibility {
    pub message: Box<str>,
    pub span: crate::syntax::token_tree::Span,
}
impl Level {
    fn unstable(&self) -> bool {
        matches!(self, Self::Unstable { .. })
    }
}
/// One parsed annotation. Shared module instances retain separate effective owners.
#[derive(Debug)]
pub struct Policy {
    pub(crate) kind: PolicyKind,
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
    /// Account the existing cold policy owner before a second semantic phase
    /// allocates temporary reachability tables. No new persistent field is needed.
    pub(crate) fn charge_retained(&self, budget: &mut impl PublicBudget) -> Result<(), Error> {
        use crate::support::retained::{sum, vector};
        let inventory = self
            .inventory
            .as_ref()
            .ok_or_else(|| Error::unlocated("library inventory is incomplete"))?;
        budget.charge(
            self.requests.len().saturating_add(inventory.policies.len()),
            sum([
                size_of::<Self>(),
                vector(&self.requests),
                size_of::<Inventory>(),
                vector(&inventory.policies),
                vector(&inventory.implications),
                vector(&inventory.declarations),
                vector(&inventory.imports),
                inventory.consts.as_deref().map_or(0, |consts| {
                    size_of::<constness::ConstPolicies>()
                        .saturating_add(vector(&consts.declarations))
                }),
            ]),
        )?;
        for request in &self.requests {
            budget.charge(0, request.len())?;
        }
        for policy in &inventory.policies {
            let reason = match &policy.level {
                Level::Stable(_) => 0,
                Level::StableWithCompatibility { note, .. } => {
                    size_of::<ModuleCompatibility>().saturating_add(note.message.len())
                }
                Level::Unstable { reason, .. } => reason.as_deref().map_or(0, str::len),
            };
            budget.charge(0, policy.feature.len().saturating_add(reason))?;
        }
        for edge in &inventory.implications {
            budget.charge(1, edge.by.len())?;
        }
        Ok(())
    }
    #[must_use]
    pub const fn inventory(&self) -> Option<&Inventory> {
        self.inventory.as_ref()
    }
    #[must_use]
    pub(crate) const fn staged_inventory(&self) -> Option<&Inventory> {
        if self.staged {
            self.inventory.as_ref()
        } else {
            None
        }
    }
    pub(super) fn publish(&mut self, inventory: Inventory) {
        self.inventory = Some(inventory);
    }
}
/// Completed declaration policies. Numerical IDs belong to this resolved package.
pub struct Inventory {
    policies: Vec<Policy>,
    implications: Vec<implications::Implication>,
    declarations: Vec<Option<PolicyId>>,
    imports: Vec<Option<PolicyId>>,
    consts: Option<Box<constness::ConstPolicies>>,
    steps: usize,
}
impl Inventory {
    /// Observe a completed local row; missing rows are not known absence.
    pub(crate) fn local_declaration(&self, id: DefId) -> Result<Option<&Policy>, &'static str> {
        let row = self
            .declarations
            .get(id.0 as usize)
            .ok_or("local stability declaration inventory is incomplete")?;
        row.map(|id| {
            self.policies
                .get(id.get() - 1)
                .ok_or("local stability declaration policy identity is missing")
        })
        .transpose()
    }
    pub(crate) fn policies(&self) -> &[Policy] {
        &self.policies
    }
    pub(crate) fn declaration_ids(
        &self,
        id: DefId,
    ) -> Result<(Option<usize>, Option<usize>), &'static str> {
        let regular = self
            .declarations
            .get(id.0 as usize)
            .ok_or("availability declaration is outside its completed inventory")?;
        let constant = self
            .consts
            .as_deref()
            .and_then(|rows| rows.declarations.get(id.0 as usize))
            .copied()
            .flatten();
        Ok((
            regular.map(|id| id.get() - 1),
            constant.map(|id| id.get() - 1),
        ))
    }
    pub(crate) fn import_id(&self, index: usize) -> Result<Option<usize>, &'static str> {
        Ok(self
            .imports
            .get(index)
            .ok_or("availability import is outside its completed inventory")?
            .map(|id| id.get() - 1))
    }
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
    implications: Vec<implications::Implication>,
    attributes: HashMap<NodeKey, PolicyId>,
    import_attributes: HashMap<NodeKey, PolicyId>,
    const_attributes: HashMap<NodeKey, PolicyId>,
    comptime: Vec<NodeKey>,
    steps: usize,
    bytes: usize,
}
impl Draft {
    fn record_comptime(&mut self, key: NodeKey) -> Result<(), Error> {
        let growth = crate::support::retained::Growth::vector::<NodeKey>(
            self.comptime.len(),
            self.comptime.capacity(),
            1,
        );
        self.charge(growth.work.saturating_add(1), growth.bytes)?;
        self.comptime
            .try_reserve(1)
            .map_err(|_| Error::unlocated("comptime API owner allocation failed"))?;
        self.comptime.push(key);
        Ok(())
    }
    fn finish_comptime(&mut self) -> Result<(), Error> {
        let count = self.comptime.len();
        let depth = usize::try_from(usize::BITS - count.leading_zeros()).unwrap_or(usize::MAX);
        self.charge(count.saturating_mul(depth.saturating_add(1)), 0)?;
        self.comptime
            .sort_unstable_by_key(|key| (key.file.0, key.node.get()));
        self.comptime.dedup();
        Ok(())
    }
    fn is_comptime(&mut self, key: NodeKey) -> Result<bool, Error> {
        let depth = usize::try_from(usize::BITS - self.comptime.len().leading_zeros())
            .unwrap_or(usize::MAX);
        self.charge(depth.saturating_add(1), 0)?;
        Ok(self
            .comptime
            .binary_search_by_key(&(key.file.0, key.node.get()), |key| {
                (key.file.0, key.node.get())
            })
            .is_ok())
    }
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
pub(crate) use availability_imports::{
    check as check_imported_api, declaration_deprecation_absent,
    visit_warnings as visit_compatibility_warnings,
};
pub(in crate::resolve) use compatibility_notes::{
    collect as collect_compatibility, finish as finish_compatibility,
};
pub(super) use inventory::complete;

#[cfg(test)]
#[path = "../../tests/invariants/local_availability.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/invariants/policy_transport.rs"]
mod transport_tests;

#[cfg(test)]
#[path = "../../tests/invariants/versioned_predicates.rs"]
mod predicate_transport_tests;

#[cfg(test)]
#[path = "../../tests/invariants/stability_implications.rs"]
mod implication_tests;
