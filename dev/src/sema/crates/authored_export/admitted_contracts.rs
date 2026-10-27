//! One completion source retains either a genuine collector receipt or encoded columns.
use super::CollectedReceipt;
use crate::{
    resolve::{
        bindings::consumer_policy::HeaderDataOwner, crates::ReservationWork, expansion::Report,
        package::Error,
    },
    support::crates::{Dependencies, generics::source as s},
};
use std::sync::Arc;

pub(crate) enum HeaderCompletion {
    Collected(CollectedReceipt),
    Encoded(HeaderDataOwner),
}
impl HeaderCompletion {
    pub(crate) fn encoded(report: &Report, work: &mut ReservationWork) -> Result<Self, Error> {
        let policy = report.package.staged_consumer().ok_or_else(|| {
            Error::unlocated("encoded header completion requires its actual consumer")
        })?;
        policy
            .require_package(&report.package)
            .map_err(Error::unlocated)?;
        let descriptor = policy
            .inventory()
            .artifacts()
            .get(policy.index() as usize)
            .and_then(crate::support::crates::Interface::source_descriptor)
            .ok_or_else(|| {
                Error::unlocated("encoded header completion lost its actual provider")
            })?;
        descriptor.provided_contracts().map_err(Error::unlocated)?;
        Ok(Self::Encoded(policy.header_data(work)?))
    }
    pub(crate) fn columns<'a>(
        &'a self,
        report: &'a Report,
        work: &mut ReservationWork,
    ) -> Result<&'a s::HeaderAttributeColumns, Error> {
        match self {
            Self::Collected(receipt) => receipt.columns(report, work),
            Self::Encoded(owner) => {
                work.charge(3)?;
                let policy = report.package.staged_consumer().ok_or_else(|| {
                    Error::unlocated("encoded header columns require their actual consumer")
                })?;
                policy
                    .require_package(&report.package)
                    .map_err(Error::unlocated)?;
                owner
                    .require_inventory(policy.inventory(), policy.index())
                    .map_err(Error::unlocated)?;
                owner.columns().map_err(Error::unlocated)
            }
        }
    }
    pub(crate) fn retain_data(
        &self,
        report: &Report,
        work: &mut ReservationWork,
    ) -> Result<HeaderDataOwner, Error> {
        match self {
            Self::Collected(receipt) => receipt.retain_data(report, work),
            Self::Encoded(owner) => {
                self.columns(report, work)?;
                owner.shared(work)
            }
        }
    }
    pub(crate) fn retained_columns<'a>(
        &'a self,
        inventory: &Arc<Dependencies>,
        owner: &'a HeaderDataOwner,
        index: u32,
        work: &mut ReservationWork,
    ) -> Result<&'a s::HeaderAttributeColumns, Error> {
        match self {
            Self::Collected(receipt) => receipt.retained_columns(inventory, owner, index, work),
            Self::Encoded(actual) => {
                work.charge(2)?;
                actual
                    .require_inventory(inventory, index)
                    .map_err(Error::unlocated)?;
                owner
                    .require_inventory(inventory, index)
                    .map_err(Error::unlocated)?;
                actual.identity(work)?.require_owner(owner, work)?;
                owner.columns().map_err(Error::unlocated)
            }
        }
    }
}
