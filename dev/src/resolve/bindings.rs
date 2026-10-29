//! Configured source providers own real declaration claims and explicit features.
mod admission;
mod allowances;
mod annotations;
mod claim_tokens;
mod client;
pub(crate) mod consumer_policy;
mod deprecation_feature;
mod diagnostic_extras;
#[path = "bindings/allowance_owner.rs"]
mod macro_context;
mod panic_claims;
pub(in crate::resolve) mod panic_contracts;
mod pointee_claims;
pub(super) mod policy;
mod positions;
#[path = "bindings/boundary_claims.rs"]
mod range_claims;
mod reservations;
mod role_attributes;
pub(crate) mod root_options;
#[path = "bindings/claimed_macros.rs"]
mod source_builtin;
mod try_claims;
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
    Staged(std::rc::Rc<consumer_policy::StagedConsumer>),
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
    pub(crate) fn requested_cfg_gate(
        &self,
        gate: crate::expand::config::CfgGate,
    ) -> Result<bool, &'static str> {
        let feature = match gate {
            crate::expand::config::CfgGate::UbChecks => return Ok(self.requested_cfg_ub_checks()),
            crate::expand::config::CfgGate::TargetHasAtomic => policy::Feature::CfgTargetHasAtomic,
            crate::expand::config::CfgGate::Sanitize => policy::Feature::CfgSanitize,
        };
        if let Some(source) = self.staged_consumer() {
            source.require_package(self)?;
            return Ok(source.features.has(feature));
        }
        if let Some(client) = self.source_client() {
            if client.current().name() != self.crate_name || client.current() == client.provider() {
                return Err("cfg feature client identity changed");
            }
            client.validate_inventory()?;
            return Ok(client.features.has(feature));
        }
        Ok(self
            .macro_context()?
            .is_some_and(|context| context.requested(feature)))
    }
    pub(crate) fn staged_consumer(&self) -> Option<&std::rc::Rc<consumer_policy::StagedConsumer>> {
        match self.source.as_deref()? {
            Source::Staged(consumer) => Some(consumer),
            Source::Core(_) | Source::Client(_) | Source::LocalMacros(_) => None,
        }
    }
    pub(crate) fn requested_impl_restriction(&self) -> Result<bool, &'static str> {
        if let Some(source) = self.staged_consumer() {
            source.require_package(self)?;
            return Ok(source.features.has(policy::Feature::ImplRestriction));
        }
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
    pub(crate) fn specialization_requests(&self) -> Result<(bool, bool), &'static str> {
        let requested = |features: &policy::Features| {
            (
                features.has(policy::Feature::MinSpecialization),
                features.has(policy::Feature::Specialization),
            )
        };
        if let Some(source) = self.staged_consumer() {
            source.require_package(self)?;
            return Ok(requested(&source.features));
        }
        if let Some(client) = self.source_client() {
            if client.current().name() != self.crate_name || client.current() == client.provider() {
                return Err("specialization client identity changed");
            }
            client.validate_inventory()?;
            return Ok(requested(&client.features));
        }
        Ok(self.macro_context()?.map_or((false, false), |context| {
            (
                context.requested(policy::Feature::MinSpecialization),
                context.requested(policy::Feature::Specialization),
            )
        }))
    }
    pub(crate) fn requested_defaultness(&self, function: bool) -> Result<bool, &'static str> {
        let (minimal, full) = self.specialization_requests()?;
        Ok(full || function && minimal)
    }
    pub(crate) fn requested_alias_feature(&self, constant: bool) -> Result<bool, &'static str> {
        let feature = if constant {
            policy::Feature::ConstTrait
        } else {
            policy::Feature::TraitAlias
        };
        if let Some(client) = self.source_client() {
            if client.current().name() != self.crate_name || client.current() == client.provider() {
                return Err("trait alias client identity changed");
            }
            client.validate_inventory()?;
            return Ok(client.features.has(feature));
        }
        Ok(self
            .macro_context()?
            .is_some_and(|context| context.requested(feature)))
    }
    pub(crate) fn requested_cpu_gate(
        &self,
        gate: crate::support::cpu_features::Gate,
    ) -> Result<bool, &'static str> {
        let feature = policy::Feature::cpu_gate(gate);
        if let Some(source) = self.staged_consumer() {
            source.require_package(self)?;
            return Ok(source.features.has(feature));
        }
        if let Some(client) = self.source_client() {
            if client.current().name() != self.crate_name || client.current() == client.provider() {
                return Err("target-feature client identity changed");
            }
            client.validate_inventory()?;
            return Ok(client.features.has(feature));
        }
        Ok(self
            .macro_context()?
            .is_some_and(|context| context.requested(feature)))
    }
    pub(crate) fn requested_simd_feature(&self, foreign: bool) -> Result<bool, &'static str> {
        let feature = if foreign {
            policy::Feature::SimdFfi
        } else {
            policy::Feature::ReprSimd
        };
        if let Some(source) = self.staged_consumer() {
            source.require_package(self)?;
            return Ok(source.features.has(feature));
        }
        if let Some(client) = self.source_client() {
            if client.current().name() != self.crate_name || client.current() == client.provider() {
                return Err("SIMD client identity changed");
            }
            client.validate_inventory()?;
            return Ok(client.features.has(feature));
        }
        Ok(self
            .macro_context()?
            .is_some_and(|context| context.requested(feature)))
    }
    pub(crate) fn requested_pattern_feature(&self, syntax: bool) -> Result<bool, &'static str> {
        let feature = if syntax {
            policy::Feature::PatternTypes
        } else {
            policy::Feature::PatternTypeMacro
        };
        if let Some(source) = self.staged_consumer() {
            source.require_package(self)?;
            return Ok(source.features.has(feature));
        }
        if let Some(client) = self.source_client() {
            if client.current().name() != self.crate_name || client.current() == client.provider() {
                return Err("pattern type client identity changed");
            }
            client.validate_inventory()?;
            return Ok(client.features.has(feature));
        }
        Ok(self
            .macro_context()?
            .is_some_and(|context| context.requested(feature)))
    }
    pub(crate) fn requested_foreign_types(&self) -> Result<bool, &'static str> {
        if let Some(client) = self.source_client() {
            if client.current().name() != self.crate_name || client.current() == client.provider() {
                return Err("foreign-type client identity changed");
            }
            client.validate_inventory()?;
            return Ok(client.features.has(policy::Feature::ExternTypes));
        }
        Ok(self
            .macro_context()?
            .is_some_and(|context| context.requested(policy::Feature::ExternTypes)))
    }
    pub(crate) fn requested_marker_traits(&self) -> Result<bool, &'static str> {
        if let Some(source) = self.staged_consumer() {
            source.require_package(self)?;
            return Ok(source.features.has(policy::Feature::MarkerTraitAttr));
        }
        if let Some(client) = self.source_client() {
            if client.current().name() != self.crate_name || client.current() == client.provider() {
                return Err("marker-trait client identity changed");
            }
            client.validate_inventory()?;
            return Ok(client.features.has(policy::Feature::MarkerTraitAttr));
        }
        Ok(self
            .macro_context()?
            .is_some_and(|context| context.requested(policy::Feature::MarkerTraitAttr)))
    }
    pub(crate) fn requested_final_functions(&self) -> Result<bool, &'static str> {
        self.requested_expression_feature(
            policy::Feature::FinalAssociatedFunctions,
            "final-method client identity changed",
        )
    }
    pub(crate) fn requested_try_blocks(&self) -> Result<bool, &'static str> {
        self.requested_expression_feature(
            policy::Feature::TryBlocks,
            "try-block client identity changed",
        )
    }
    pub(crate) fn requested_const_closures(&self) -> Result<bool, &'static str> {
        self.requested_expression_feature(
            policy::Feature::ConstClosures,
            "const-closure client identity changed",
        )
    }
    pub(crate) fn requested_builtin_syntax(&self) -> Result<bool, &'static str> {
        self.requested_expression_feature(
            policy::Feature::BuiltinSyntax,
            "builtin-syntax client identity changed",
        )
    }
    pub(crate) fn requested_offset_of_enum(&self) -> Result<bool, &'static str> {
        self.requested_expression_feature(
            policy::Feature::OffsetOfEnum,
            "offset-of-enum client identity changed",
        )
    }
    pub(crate) fn requested_metavariable_concat(&self) -> Result<bool, &'static str> {
        self.requested_expression_feature(
            policy::Feature::MetavariableConcat,
            "metavariable-concat client identity changed",
        )
    }
    pub(crate) fn requested_metavariable_expressions(&self) -> Result<bool, &'static str> {
        self.requested_expression_feature(
            policy::Feature::MetavariableExpressions,
            "metavariable-expression client identity changed",
        )
    }
    pub(crate) fn requested_coverage_attribute(&self) -> Result<bool, &'static str> {
        self.requested_expression_feature(
            policy::Feature::CoverageAttribute,
            "coverage client identity changed",
        )
    }
    fn requested_expression_feature(
        &self,
        feature: policy::Feature,
        changed: &'static str,
    ) -> Result<bool, &'static str> {
        if let Some(source) = self.staged_consumer() {
            source.require_package(self)?;
            return Ok(source.features.has(feature));
        }
        if let Some(client) = self.source_client() {
            if client.current().name() != self.crate_name || client.current() == client.provider() {
                return Err(changed);
            }
            client.validate_inventory()?;
            return Ok(client.features.has(feature));
        }
        Ok(self
            .macro_context()?
            .is_some_and(|context| context.requested(feature)))
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
            Source::Client(_) | Source::Staged(_) | Source::LocalMacros(_) => None,
        }
    }
    #[must_use]
    pub fn source_client(&self) -> Option<&SourceClient> {
        match self.source.as_deref()? {
            Source::Core(_) | Source::Staged(_) | Source::LocalMacros(_) => None,
            Source::Client(source) => Some(source),
        }
    }
    pub(in crate::resolve) fn source_features(&self) -> Option<&policy::Features> {
        Some(match self.source.as_deref()? {
            Source::Core(source) => &source.features,
            Source::Client(source) => &source.features,
            Source::Staged(source) => &source.features,
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

#[cfg(test)]
pub(crate) const fn cpu_request_layouts() -> (usize, usize, usize) {
    (
        size_of::<policy::Features>(),
        size_of::<policy::MacroFeatures>(),
        size_of::<SourceCore>(),
    )
}
