//! Configured source providers own real declaration claims and explicit features.
mod admission;
mod annotations;
mod claim_tokens;
mod client;
pub(in crate::resolve) mod panic_contracts;
pub(super) mod policy;
mod positions;
mod reservations;
mod role_attributes;
#[path = "bindings/claimed_macros.rs"]
mod source_builtin;
use crate::support::{
    bindings::Mode,
    crates::{CrateIdentity, Dependencies},
};
pub(super) use admission::validate as validate_features;
pub use client::SourceClient;
pub(super) use reservations::finish;
pub use reservations::{Declaration, IntrinsicDeclaration, Reservations};
pub(crate) use role_attributes::{HeaderAttribute, HeaderAttributes};
use std::sync::Arc;

pub(in crate::resolve) enum Source {
    Core(SourceCore),
    Client(SourceClient),
}

/// Driver-independent source selection, fixed before expansion or ordinary lookup.
#[derive(Default)]
pub struct Context {
    pub crate_name: Option<String>,
    pub current: Option<CrateIdentity>,
    pub dependencies: Option<Arc<Dependencies>>,
    pub mode: Mode,
}

pub struct SourceCore {
    pub current: CrateIdentity,
    pub provider: CrateIdentity,
    pub(super) features: policy::Features,
    pub(in crate::resolve) macros: panic_contracts::Contracts,
}
impl super::package::Package {
    #[must_use]
    pub fn uses_source_bindings(&self) -> bool {
        self.source.is_some()
    }
    /// Only the actual provider may claim local language declarations.
    #[must_use]
    pub fn source_core(&self) -> Option<&SourceCore> {
        match self.source.as_deref()? {
            Source::Core(source) => Some(source),
            Source::Client(_) => None,
        }
    }
    #[must_use]
    pub fn source_client(&self) -> Option<&SourceClient> {
        match self.source.as_deref()? {
            Source::Core(_) => None,
            Source::Client(source) => Some(source),
        }
    }
    pub(in crate::resolve) fn source_features(&self) -> Option<&policy::Features> {
        Some(match self.source.as_deref()? {
            Source::Core(source) => &source.features,
            Source::Client(source) => &source.features,
        })
    }
    pub(crate) fn permits_source_negative_impls(&self) -> bool {
        self.source_features()
            .is_some_and(|features| features.has(policy::Feature::NegativeImpls))
    }
    /// This admits declaration syntax; it does not establish callable evidence.
    pub(crate) fn permits_source_freeze_impls(&self) -> bool {
        self.source_features()
            .is_some_and(|features| features.has(policy::Feature::FreezeImpls))
    }
    pub(crate) fn permits_source_rust_call(&self) -> bool {
        self.source_features()
            .is_some_and(|features| features.has(policy::Feature::UnboxedClosures))
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/provider_reservations.rs"]
mod tests;
