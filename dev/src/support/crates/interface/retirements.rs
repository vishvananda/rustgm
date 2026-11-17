//! One immutable crate owns its optional removed-feature attachment.
use super::Interface;
use crate::support::crates::{MAX_BYTES, MAX_COMPONENTS, MAX_NAME_BYTES, RemovedFeatures};
impl Interface {
    #[must_use]
    pub fn removed_features(&self) -> Option<&RemovedFeatures> {
        self.retirements.as_deref()
    }
    /// Attach checked declaration data without changing an API or callable policy.
    /// # Errors
    /// Rejects foreign ownership, repeated attachments and combined artifact quotas.
    pub fn with_removed_features(mut self, rows: RemovedFeatures) -> Result<Self, &'static str> {
        if self.retirements.is_some() || rows.owner() != self.identity() {
            return Err("removed library features are foreign or already published");
        }
        let bytes = self
            .base_retained_bytes()
            .saturating_add(
                self.generic_payload
                    .as_deref()
                    .map_or(0, crate::support::crates::generics::Payload::retained_bytes),
            )
            .saturating_add(rows.allocation_bytes());
        let atoms = self.atoms.saturating_add(rows.entries().len());
        let names = self.names.saturating_add(rows.name_bytes());
        if bytes > MAX_BYTES || atoms > MAX_COMPONENTS || names > MAX_NAME_BYTES {
            return Err("removed library feature attachment exceeds combined artifact budget");
        }
        self.atoms = atoms;
        self.names = names;
        self.retirements = Some(Box::new(rows));
        Ok(self)
    }
}
impl Interface {
    /// This closed transport carries no definition, foreign edge or executable data.
    pub(crate) fn primitive_empty(&self) -> bool {
        self.dependencies.is_empty()
            && self.functions.is_empty()
            && self.constants.is_empty()
            && self.nominals.is_none()
            && self.storage.is_none()
            && self.public_names.is_none()
            && self.generic_payload.is_none()
            && self.native_links.is_none()
            && self.macros.is_none()
    }
}
