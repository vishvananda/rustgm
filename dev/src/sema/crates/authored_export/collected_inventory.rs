//! Only the real collected row owner can bind a source-completion receipt.
use super::{CollectedRows, g, s};
use crate::{
    resolve::{
        bindings::consumer_policy::{HeaderDataIdentity, ProviderAdmission},
        crates::ReservationWork,
        expansion::Report,
        package::Error,
    },
    support::{
        crates::{CrateIdentity, Dependencies, Interface},
        retained,
    },
};
use std::sync::Arc;

pub(crate) struct CollectedArtifact {
    inventory: Arc<Dependencies>,
    contracts: s::HeaderAttributeColumns,
    contract_bytes: usize,
}
/// Private fields prevent an unpaid caller from certifying arbitrary columns.
pub(crate) struct CollectedReceipt {
    inventory: Arc<Dependencies>,
    data: HeaderDataIdentity,
}
impl CollectedRows {
    pub(crate) fn into_artifact(
        self,
        base: Interface,
        work: &mut ReservationWork,
    ) -> Result<CollectedArtifact, Error> {
        work.charge(self.work.steps.saturating_add(8))?;
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
            .saturating_add(base.base_retained_bytes());
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
            .saturating_sub(size_of::<s::HeaderAttributeColumns>());
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
        let inventory =
            Arc::new(Dependencies::new(vec![artifact], Vec::new()).map_err(Error::unlocated)?);
        // All producer-local identities and collection scratch have retired.
        work.retained = before.saturating_add(actual);
        Ok(CollectedArtifact {
            inventory,
            contracts: self.contracts,
            contract_bytes: self.contract_bytes,
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
        let old_owner = size_of::<Self>().saturating_add(
            self.contract_bytes
                .saturating_sub(size_of::<s::HeaderAttributeColumns>()),
        );
        work.charge(2)?;
        work.reserve(size_of::<CollectedReceipt>())?;
        let admission =
            ProviderAdmission::from_data(Arc::clone(&self.inventory), 0, self.contracts, work)?;
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
