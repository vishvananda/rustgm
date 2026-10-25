//! Configured source providers own real declaration claims and explicit features.
mod admission;
mod allowances;
mod annotations;
mod claim_tokens;
mod client;
#[path = "bindings/allowance_owner.rs"]
mod macro_context;
pub(in crate::resolve) mod panic_contracts;
pub(super) mod policy;
mod positions;
mod reservations;
mod role_attributes;
pub(crate) mod root_options;
#[path = "bindings/claimed_macros.rs"]
mod source_builtin;
use crate::support::{
    bindings::Mode,
    crates::{CrateIdentity, Dependencies},
};
pub(super) use admission::validate as validate_features;
pub use allowances::{FeatureName, KnownFeature};
pub use client::SourceClient;
use macro_context::LocalMacros;
pub(super) use reservations::finish;
pub use reservations::{Declaration, IntrinsicDeclaration, Reservations};
pub(crate) use role_attributes::{HeaderAttribute, HeaderAttributes};
pub(super) use root_options::validate as validate_crate_options;
use std::sync::Arc;

pub(in crate::resolve) enum Source {
    Core(SourceCore),
    Client(SourceClient),
    LocalMacros(LocalMacros),
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
    pub(crate) fn requested_impl_restriction(&self) -> Result<bool, &'static str> {
        if let Some(Source::Client(client)) = self.source.as_deref() {
            if client.current().name() != self.crate_name || client.current() == client.provider() {
                return Err("implementation restriction client identity changed");
            }
            client.validate_inventory()?;
            return Ok(client.features.has(policy::Feature::ImplRestriction));
        }
        Ok(self
            .macro_context()?
            .is_some_and(|context| context.requested(policy::Feature::ImplRestriction)))
    }
    pub(crate) fn requested_defaultness(&self, function: bool) -> Result<bool, &'static str> {
        let requested = |features: &policy::Features| {
            features.has(policy::Feature::Specialization)
                || function && features.has(policy::Feature::MinSpecialization)
        };
        if let Some(client) = self.source_client() {
            if client.current().name() != self.crate_name || client.current() == client.provider() {
                return Err("specialization client identity changed");
            }
            client.validate_inventory()?;
            return Ok(requested(&client.features));
        }
        Ok(self.macro_context()?.is_some_and(|context| {
            context.requested(policy::Feature::Specialization)
                || function && context.requested(policy::Feature::MinSpecialization)
        }))
    }
    #[must_use]
    pub fn uses_source_bindings(&self) -> bool {
        self.source.as_deref().is_some_and(Source::owns_bindings)
    }
    /// Only the actual provider may claim local language declarations.
    #[must_use]
    pub fn source_core(&self) -> Option<&SourceCore> {
        match self.source.as_deref()? {
            Source::Core(source) => Some(source),
            Source::Client(_) | Source::LocalMacros(_) => None,
        }
    }
    #[must_use]
    pub fn source_client(&self) -> Option<&SourceClient> {
        match self.source.as_deref()? {
            Source::Core(_) | Source::LocalMacros(_) => None,
            Source::Client(source) => Some(source),
        }
    }
    pub(in crate::resolve) fn source_features(&self) -> Option<&policy::Features> {
        Some(match self.source.as_deref()? {
            Source::Core(source) => &source.features,
            Source::Client(source) => &source.features,
            Source::LocalMacros(_) => return None,
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
