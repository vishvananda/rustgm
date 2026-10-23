//! Configured source providers own real declaration claims and explicit features.
mod admission;
pub(super) mod policy;
mod positions;
mod reservations;
#[path = "bindings/claimed_macros.rs"]
mod source_builtin;
use crate::support::{
    bindings::Mode,
    crates::{CrateIdentity, Dependencies},
};
pub(super) use admission::validate as validate_features;
pub(super) use reservations::finish;
pub use reservations::{Declaration, IntrinsicDeclaration, Reservations};
pub(super) use source_builtin::claim as builtin_macro;
use std::sync::Arc;

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
}
impl SourceCore {
    /// This admits declaration syntax; it does not establish callable evidence.
    pub(crate) fn permits_rust_call(&self) -> bool {
        self.features.has(policy::Feature::UnboxedClosures)
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/provider_reservations.rs"]
mod tests;
