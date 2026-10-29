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
        crates::{CrateIdentity, Dependencies},
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
    pub(crate) fn availability_required(
        &self,
        dependencies: Option<&Dependencies>,
    ) -> Result<bool, &'static str> {
        let dependencies = dependencies.unwrap_or(&self.dependencies);
        let index = self.require_inventory(dependencies)?;
        Ok(dependencies
            .artifacts()
            .get(index as usize)
            .and_then(crate::support::crates::Interface::source_descriptor)
            .ok_or("source client availability descriptor is absent")?
            .profile()
            .availability())
    }
    pub(super) fn validate_inventory(&self) -> Result<(), &'static str> {
        self.require_inventory(&self.dependencies).map(|_| ())
    }
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
        {
            return Err("source client provider descriptor changed");
        }
        if !descriptor.profile().ordinary_functions() {
            match descriptor.profile() {
                crate::support::crates::generics::source::Profile::Source2024ModuleContracts
                | crate::support::crates::generics::source::Profile::Source2024ModuleAvailability | crate::support::crates::generics::source::Profile::Source2024ModuleFeaturePredicates | crate::support::crates::generics::source::Profile::Source2024ModuleDefinitions | crate::support::crates::generics::source::Profile::Source2024ModuleCompatibility =>
                {
                    descriptor.module_contracts()?;
                }
                _ => return Err("source client retained provider profile is unsupported"),
            }
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
    let current = identity(unit, context, provider, no_core, &features)?;
    let error = |message| Error::at(FileId(0), 0, message);
    let dependencies = context
        .dependencies
        .as_ref()
        .ok_or_else(|| error("source client requires an authenticated dependency inventory"))?;
    let provider_index = dependencies
        .source_provider_index(provider)
        .map_err(error)?;
    let macros = features.macros();
    Ok(Prepared {
        source: Some(Box::new(Source::Client(SourceClient {
            current: current.clone(),
            provider: provider.clone(),
            features,
            dependencies: Arc::clone(dependencies),
            provider_index,
        }))),
        library,
        macros,
    })
}

pub(super) fn identity<'a>(
    unit: &Unit,
    context: &'a Context,
    provider: &CrateIdentity,
    no_core: bool,
    features: &Features,
) -> Result<&'a CrateIdentity, Error> {
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
    Ok(current)
}

#[cfg(test)]
#[path = "../../../tests/invariants/client_selection_policy.rs"]
mod tests;
