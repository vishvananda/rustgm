//! Only the real collected row owner can bind a source-completion receipt.
use super::{CollectedRows, g, s};
use crate::{
    resolve::{
        bindings::consumer_policy::{HeaderDataIdentity, HeaderDataOwner, ProviderAdmission},
        crates::ReservationWork,
        expansion::Report,
        package::Error,
    },
    support::{
        crates::{CrateIdentity, Dependencies, Dependency, Interface},
        retained,
    },
};
use std::sync::Arc;

pub(crate) struct CollectedArtifact {
    inventory: Arc<Dependencies>,
    contracts: s::HeaderAttributeColumns,
    contract_bytes: usize,
    names: s::NamespaceColumns,
    names_bytes: usize,
}
/// Private fields prevent an unpaid caller from certifying arbitrary columns.
pub(crate) struct CollectedReceipt {
    inventory: Arc<Dependencies>,
    data: HeaderDataIdentity,
}
impl CollectedRows {
    pub(crate) fn into_provided_interface(
        self,
        base: Interface,
        modifiers: s::HeaderModifierColumns,
    ) -> Result<Interface, Error> {
        if base.identity() != &self.provider
            || base.generic_payload().is_some()
            || !base.functions().is_empty()
            || !base.constants().is_empty()
            || base.nominals().is_some()
            || base.storage().is_some()
            || base.public_names().is_some()
        {
            return Err(Error::unlocated(
                "provided publication requires its genuine dedicated artifact base",
            ));
        }
        let base_bytes = base.base_retained_bytes();
        if base_bytes
            .saturating_add(self.work.reserved_bytes)
            .saturating_add(self.work.scratch_bytes)
            > crate::support::crates::MAX_BYTES
        {
            return Err(Error::unlocated(
                "provided publication exceeds combined collection storage",
            ));
        }
        let mut work = g::Work {
            steps: self.work.steps,
            ..g::Work::default()
        };
        let contracts = s::ProvidedContractRows {
            namespace: self.names,
            modifiers,
        };
        let payload = g::Payload::from_source_provided_contracts(
            &self.provider,
            self.rows,
            self.contracts,
            contracts,
            base_bytes,
            &mut work,
        )
        .map_err(Error::unlocated)?;
        base.with_generic_payload(payload).map_err(Error::unlocated)
    }
    pub(crate) fn into_artifact(
        self,
        base: Interface,
        work: &mut ReservationWork,
    ) -> Result<CollectedArtifact, Error> {
        self.into_artifact_with_alias(base, None, work)
    }
    pub(crate) fn into_named_artifact(
        self,
        base: Interface,
        alias: Dependency,
        work: &mut ReservationWork,
    ) -> Result<CollectedArtifact, Error> {
        self.into_artifact_with_alias(base, Some(alias), work)
    }
    fn into_artifact_with_alias(
        self,
        base: Interface,
        alias: Option<Dependency>,
        work: &mut ReservationWork,
    ) -> Result<CollectedArtifact, Error> {
        work.charge(self.work.steps.saturating_add(8))?;
        let alias_bytes = if let Some(alias) = &alias {
            work.charge(alias.alias().len().saturating_add(1))?;
            if alias.identity() != &self.provider {
                return Err(Error::unlocated(
                    "named source artifact alias selects another provider",
                ));
            }
            alias.storage_bytes()
        } else {
            0
        };
        if base.identity() != &self.provider
            || !base.dependencies().is_empty()
            || !base.functions().is_empty()
            || !base.constants().is_empty()
            || base.nominals().is_some()
            || base.storage().is_some()
            || base.public_names().is_some()
            || base.generic_payload().is_some()
        {
            return Err(Error::unlocated(
                "collected source artifact requires its actual dedicated provider base",
            ));
        }
        let before = work.retained;
        let owned = size_of::<CollectedArtifact>()
            .saturating_add(size_of::<Dependencies>())
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(base.base_retained_bytes())
            .saturating_add(alias_bytes);
        let scratch = retained::new_map::<&CrateIdentity, usize>(1)
            .saturating_add(size_of::<Vec<usize>>())
            .saturating_add(6 * size_of::<usize>());
        // These are the SAME already-validated header/domain/numbering kernels.
        // Reuse their actual producer peak; only freeze's new transfer buffers
        // need the owning preflight observer below. No envelope is re-derived.
        work.reserve(
            self.work
                .reserved_bytes
                .saturating_add(self.work.scratch_bytes)
                .saturating_add(owned)
                .saturating_add(scratch),
        )?;
        let tail = self
            .contract_bytes
            .saturating_sub(size_of::<s::HeaderAttributeColumns>())
            .saturating_add(
                self.names_bytes
                    .saturating_sub(size_of::<s::NamespaceColumns>()),
            );
        let metadata_base = before.saturating_add(owned).saturating_add(tail);
        let mut validation = g::Work {
            steps: work.steps,
            ..g::Work::default()
        };
        let start = validation.steps;
        let payload = g::Payload::from_source_provided_observed(
            &self.provider,
            self.rows,
            metadata_base,
            &mut validation,
            &mut |_, peak| {
                work.peak = work.peak.max(peak);
                work.retained = work.retained.max(peak);
            },
        );
        work.charge(validation.steps.saturating_sub(start))?;
        let payload = payload.map_err(Error::unlocated)?;
        let actual = owned
            .saturating_add(tail)
            .saturating_add(payload.retained_bytes());
        if before.saturating_add(actual) > work.retained {
            return Err(Error::unlocated(
                "collected source artifact exceeded its actual metadata prepayment",
            ));
        }
        work.charge(9)?;
        let artifact = base
            .with_generic_payload(payload)
            .map_err(Error::unlocated)?;
        let aliases = alias.into_iter().collect();
        let inventory =
            Arc::new(Dependencies::new(vec![artifact], aliases).map_err(Error::unlocated)?);
        // All producer-local identities and collection scratch have retired.
        work.retained = before.saturating_add(actual);
        Ok(CollectedArtifact {
            inventory,
            contracts: self.contracts,
            contract_bytes: self.contract_bytes,
            names: self.names,
            names_bytes: self.names_bytes,
        })
    }
}
impl CollectedArtifact {
    pub(crate) fn inventory(&self) -> &Arc<Dependencies> {
        &self.inventory
    }
    pub(crate) fn into_admission(
        self,
        work: &mut ReservationWork,
    ) -> Result<(ProviderAdmission, CollectedReceipt), Error> {
        let before = work.retained;
        let old_owner = size_of::<Self>()
            .saturating_add(
                self.contract_bytes
                    .saturating_sub(size_of::<s::HeaderAttributeColumns>()),
            )
            .saturating_add(
                self.names_bytes
                    .saturating_sub(size_of::<s::NamespaceColumns>()),
            );
        work.charge(2)?;
        work.reserve(size_of::<CollectedReceipt>())?;
        let admission = ProviderAdmission::from_data_with_names(
            Arc::clone(&self.inventory),
            0,
            self.contracts,
            Some(self.names),
            work,
        )?;
        let data = admission.header_identity(work)?;
        let actual = before
            .checked_sub(old_owner)
            .ok_or_else(|| {
                Error::unlocated("collected admission lost its owned sidecar retention")
            })?
            .saturating_add(admission.retained_bytes())
            .saturating_add(size_of::<CollectedReceipt>());
        if actual > work.retained {
            return Err(Error::unlocated(
                "collected admission exceeded its actual sidecar prepayment",
            ));
        }
        work.retained = actual;
        Ok((
            admission,
            CollectedReceipt {
                inventory: self.inventory,
                data,
            },
        ))
    }
}
impl CollectedReceipt {
    pub(crate) fn retain_data(
        &self,
        report: &Report,
        work: &mut ReservationWork,
    ) -> Result<HeaderDataOwner, Error> {
        self.columns(report, work)?;
        let policy = report
            .package
            .staged_consumer()
            .ok_or_else(|| Error::unlocated("source completion lost its actual consumer"))?;
        self.data.retain(policy, work)
    }
    pub(crate) fn retained_columns<'a>(
        &'a self,
        inventory: &Arc<Dependencies>,
        owner: &'a HeaderDataOwner,
        index: u32,
        work: &mut ReservationWork,
    ) -> Result<&'a s::HeaderAttributeColumns, Error> {
        work.charge(2)?;
        if index != 0 || !Arc::ptr_eq(inventory, &self.inventory) {
            return Err(Error::unlocated(
                "retained source completion changed its actual inventory",
            ));
        }
        self.data.require_owner(owner, work)?;
        Ok(owner.columns())
    }
    pub(crate) fn columns<'a>(
        &self,
        report: &'a Report,
        work: &mut ReservationWork,
    ) -> Result<&'a s::HeaderAttributeColumns, Error> {
        work.charge(4)?;
        let policy = report.package.staged_consumer().ok_or_else(|| {
            Error::unlocated("source completion requires its genuine staged consumer")
        })?;
        policy
            .require_package(&report.package)
            .map_err(Error::unlocated)?;
        if policy.index() != 0 || !Arc::ptr_eq(policy.inventory(), &self.inventory) {
            return Err(Error::unlocated(
                "source completion belongs to another actual consumer inventory",
            ));
        }
        self.data.require(policy, work)?;
        Ok(policy.unpaid_header_columns())
    }
}
