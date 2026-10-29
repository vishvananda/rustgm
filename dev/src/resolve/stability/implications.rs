//! Relationships belong to exact policies, never to a global predicate grant.
use super::{Draft, Inventory, PolicyId};
use crate::resolve::package::{Error, Package};

pub(super) struct Implication {
    pub policy: PolicyId,
    pub by: Box<str>,
}
impl Draft {
    pub(super) fn prepay_implication(&mut self) -> Result<(), Error> {
        let growth = crate::support::retained::Growth::vector::<Implication>(
            self.implications.len(),
            self.implications.capacity(),
            1,
        );
        self.charge(growth.work, growth.bytes)
    }
    pub(super) fn implication_work(&self) -> usize {
        self.implications
            .iter()
            .map(|edge| edge.by.len().saturating_add(1))
            .sum()
    }
    /// The defining features may come from real imported policy rows, but a
    /// language feature flag alone does not establish a library definition.
    pub(super) fn validate_implications(
        &self,
        package: &Package,
        features: &crate::support::hash::HashSet<&str>,
        imported: Option<&super::availability_imports::Imports>,
    ) -> Result<(), Error> {
        for edge in &self.implications {
            if !features.contains(edge.by.as_ref())
                && !imported
                    .map(|imports| imports.has_feature(&edge.by))
                    .transpose()?
                    .unwrap_or(false)
            {
                return Err(super::owners::error(
                    package,
                    self.policies[edge.policy.get() - 1].origin,
                    "implied_by names an undefined library feature",
                ));
            }
        }
        Ok(())
    }
}
impl Inventory {
    pub(crate) fn has_implications(&self) -> bool {
        !self.implications.is_empty()
    }
}
