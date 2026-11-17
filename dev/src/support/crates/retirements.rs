//! Removed library features are crate data, never an active API declaration.
use super::{CrateIdentity, MAX_BYTES, MAX_COMPONENTS, MAX_NAME_BYTES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemovedVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemovedFeature {
    pub feature: String,
    pub reason: String,
    pub link: String,
    pub since: RemovedVersion,
}
impl RemovedFeature {
    pub(crate) fn string_bytes(&self) -> usize {
        self.feature
            .len()
            .saturating_add(self.reason.len())
            .saturating_add(self.link.len())
    }
    pub(crate) fn allocation_bytes(&self) -> usize {
        self.feature
            .capacity()
            .saturating_add(self.reason.capacity())
            .saturating_add(self.link.capacity())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemovedFeatures {
    owner: CrateIdentity,
    entries: Vec<RemovedFeature>,
}
impl RemovedFeatures {
    /// Preserve declaration order, including repeated feature data and opaque names.
    /// # Errors
    /// Rejects empty or oversized metadata attachments before publication.
    pub fn new(owner: CrateIdentity, entries: Vec<RemovedFeature>) -> Result<Self, &'static str> {
        if entries.is_empty() || entries.len() > MAX_COMPONENTS {
            return Err("invalid removed library feature count");
        }
        let result = Self { owner, entries };
        if result.allocation_bytes() > MAX_BYTES || result.name_bytes() > MAX_NAME_BYTES {
            return Err("removed library feature data exceeds storage budget");
        }
        Ok(result)
    }
    #[must_use]
    pub const fn owner(&self) -> &CrateIdentity {
        &self.owner
    }
    #[must_use]
    pub fn entries(&self) -> &[RemovedFeature] {
        &self.entries
    }
    pub(crate) fn allocation_bytes(&self) -> usize {
        self.entries.iter().fold(
            size_of::<Self>().saturating_add(crate::support::retained::vector(&self.entries)),
            |bytes, row| bytes.saturating_add(row.allocation_bytes()),
        )
    }
    pub(crate) fn name_bytes(&self) -> usize {
        self.entries.iter().fold(0usize, |bytes, row| {
            bytes.saturating_add(row.string_bytes())
        })
    }
}
