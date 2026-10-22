//! Configured source providers own real declaration claims and explicit features.
pub(super) mod policy;
mod reservations;
use crate::support::{
    bindings::Mode,
    crates::{CrateIdentity, Dependencies},
};
pub(super) use reservations::finish;
pub use reservations::{Declaration, IntrinsicDeclaration, Reservations};
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

#[cfg(test)]
#[path = "../../tests/invariants/provider_reservations.rs"]
mod tests;
