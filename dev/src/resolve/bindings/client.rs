//! Source clients retain one admitted immutable provider inventory, never call proof.
use super::{
    Context, Source,
    policy::{Feature, Features, Prepared},
};
use crate::{
    resolve::{
        package::Error,
        stability::Library,
        unit::{FileId, Unit},
    },
    support::{
        crates::{CrateIdentity, Dependencies, SourceProfile},
        edition::Edition,
    },
};
use std::sync::Arc;

pub struct SourceClient {
    current: CrateIdentity,
    provider: CrateIdentity,
    pub(super) features: Features,
    dependencies: Arc<Dependencies>,
    provider_index: u32,
}
impl SourceClient {
    #[must_use]
    pub const fn current(&self) -> &CrateIdentity {
        &self.current
    }
    #[must_use]
    pub const fn provider(&self) -> &CrateIdentity {
        &self.provider
    }
    /// Reuse the initial search only with its same immutable validated inventory.
    /// This checks provenance, not imported role or executable-body authority.
    /// # Errors
    /// Rejects a different inventory or an inconsistent actual provider descriptor.
    pub fn require_inventory(&self, dependencies: &Dependencies) -> Result<u32, &'static str> {
        if !std::ptr::eq(self.dependencies.as_ref(), dependencies) {
            return Err("source client dependency inventory changed");
        }
        let descriptor = dependencies
            .artifacts()
            .get(self.provider_index as usize)
            .and_then(crate::support::crates::Interface::source_descriptor)
            .ok_or("source client provider descriptor is missing")?;
        if descriptor.identity() != &self.provider
            || descriptor.provider() != &self.provider
            || descriptor.edition() != Edition::Rust2024
            || descriptor.profile() != SourceProfile::Source2024
        {
            return Err("source client provider descriptor changed");
        }
        Ok(self.provider_index)
    }
}

pub(super) fn prepare(
    unit: &Unit,
    context: &Context,
    provider: &CrateIdentity,
    no_core: bool,
    features: Features,
    library: Option<Box<Library>>,
) -> Result<Prepared, Error> {
    let error = |message| Error::at(FileId(0), 0, message);
    let current = context
        .current
        .as_ref()
        .ok_or_else(|| error("source client requires complete current crate identity"))?;
    if current == provider || context.crate_name.as_deref() != Some(current.name()) {
        return Err(error(
            "source client requires its distinct exact current crate identity",
        ));
    }
    if unit.edition() != Edition::Rust2024 {
        return Err(error("source client requires the Rust 2024 source profile"));
    }
    if !no_core || !features.has(Feature::NoCore) {
        return Err(error(
            "source client requires no_core and its explicit source feature",
        ));
    }
    let dependencies = context
        .dependencies
        .as_ref()
        .ok_or_else(|| error("source client requires an authenticated dependency inventory"))?;
    let provider_index = dependencies
        .source_provider_index(provider)
        .map_err(error)?;
    Ok(Prepared {
        source: Some(Box::new(Source::Client(SourceClient {
            current: current.clone(),
            provider: provider.clone(),
            features,
            dependencies: Arc::clone(dependencies),
            provider_index,
        }))),
        library,
    })
}

#[cfg(test)]
#[path = "../../../tests/invariants/client_selection_policy.rs"]
mod tests;
