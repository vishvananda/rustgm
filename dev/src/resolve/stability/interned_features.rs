//! One capture-time feature index shares spelling and retains the first policy.
use super::{Draft, Policy, PolicyId, relationships::release};
use crate::{
    resolve::{package::Error, unit::FileId},
    support::{hash::HashMap, retained::Growth},
};
use std::rc::Rc;

#[derive(Default)]
pub(super) struct Index {
    pub definitions: HashMap<Rc<str>, PolicyId>,
    pub conflict: Option<PolicyId>,
    pub spelling_bytes: usize,
}

#[cfg(test)]
#[path = "../../../tests/invariants/shared_feature_names.rs"]
mod tests;
impl Draft {
    /// The returned first policy belongs to this draft, before the next publication.
    pub(super) fn feature(
        &mut self,
        spelling: &str,
        file: FileId,
        offset: u32,
    ) -> Result<(Rc<str>, Option<PolicyId>), Error> {
        self.charge(spelling.len().saturating_add(1), 0)?;
        if let Some((name, &first)) = self.features.definitions.get_key_value(spelling) {
            return Ok((Rc::clone(name), Some(first)));
        }
        // A miss validates/copies the spelling and later hashes its insertion.
        self.charge(
            spelling.len().saturating_mul(3).saturating_add(1),
            spelling.len(),
        )?;
        if !super::attributes::identifier(spelling) {
            return Err(Error::at(
                file,
                offset,
                "library feature must be an identifier",
            ));
        }
        Ok((Rc::from(spelling), None))
    }

    /// Reserve only a new canonical entry; repeated spellings have no map growth.
    pub(super) fn reserve_feature(&mut self, first: Option<PolicyId>) -> Result<(), Error> {
        if first.is_some() {
            return Ok(());
        }
        let map = &self.features.definitions;
        let growth = Growth::map::<Rc<str>, PolicyId>(map.len(), map.capacity(), 1);
        let rehash = if growth.bytes == 0 {
            0
        } else {
            self.features.spelling_bytes
        };
        self.charge(growth.work.saturating_add(rehash), growth.bytes)?;
        self.features
            .definitions
            .try_reserve(1)
            .map_err(|_| Error::unlocated("library feature index allocation failed"))
    }

    /// No fallible work remains. Consistency errors retain their completion phase.
    pub(super) fn publish_feature(&mut self, policy: PolicyId, first: Option<PolicyId>) {
        let row: &Policy = &self.policies[policy.get() - 1];
        if let Some(first) = first {
            if self.features.conflict.is_none()
                && release(&row.level) != release(&self.policies[first.get() - 1].level)
            {
                self.features.conflict = Some(policy);
            }
        } else {
            self.features.spelling_bytes = self
                .features
                .spelling_bytes
                .saturating_add(row.feature.len());
            self.features
                .definitions
                .insert(Rc::clone(&row.feature), policy);
        }
    }
}
