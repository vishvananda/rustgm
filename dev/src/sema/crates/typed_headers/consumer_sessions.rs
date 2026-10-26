//! Actual fresh consumer reservation is opaque and cannot expose checked facts.
use crate::{
    expand::config::Config,
    resolve::{
        bindings::{
            Context,
            consumer_policy::{ProviderAdmission, Request},
        },
        crates::{ConsumerReservation, ReservationWork, unpaid_provided::Inputs},
        expansion::{self, Report},
        load::{InputFile, Sources},
        package::Error,
    },
    sema::model::Declarations,
    types::arena::Identity,
};

#[cfg(test)]
#[path = "../../../../tests/invariants/consumer_authorities.rs"]
mod tests;

pub(crate) struct ProofSession {
    authority: ConsumerAuthority,
}
struct ConsumerAuthority {
    // Canonical identities retire before their actual resolver owner.
    facts: Declarations,
    report: Report,
    reservation: ConsumerReservation,
    arena: Identity,
}
impl ProofSession {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "selected imported header/role collectors must consume this opaque reserved authority before body proof can open"
        )
    )]
    pub(crate) fn new(
        root: InputFile,
        sources: &mut impl Sources,
        config: &Config,
        context: &Context,
        admission: ProviderAdmission,
        work: &mut ReservationWork,
    ) -> Result<Box<Self>, Error> {
        work.charge(root.source.text().len().saturating_add(1))?;
        work.reserve(size_of::<Self>().saturating_add(Identity::allocation_bytes()))?;
        let request = Request::new(admission, config, work);
        let report = expansion::analyze_staged(root, sources, config, context, request)?;
        let report = crate::resolve::analyze_expanded(report)?;
        let (report, reservation) = Inputs::reserve_consumer(report, work)?;
        let mut facts = Declarations::new_staged(&report, work)?;
        work.charge(facts.definitions.len())?;
        if facts.definitions.iter().any(Option::is_some) {
            return Err(Error::unlocated(
                "private consumer reservation contains invented completed definitions",
            ));
        }
        let arena = facts.types.arena_identity();
        let session = Box::new(Self {
            authority: ConsumerAuthority {
                facts,
                report,
                reservation,
                arena,
            },
        });
        session.require(work)?;
        Ok(session)
    }
    fn require(&self, work: &mut ReservationWork) -> Result<(), Error> {
        let ConsumerAuthority {
            facts,
            report,
            reservation,
            arena,
        } = &self.authority;
        reservation.require(report, work)?;
        work.charge(6)?;
        let catalog = facts.source_bindings.as_deref().ok_or_else(|| {
            Error::unlocated("private consumer lost its actual reserved role catalog")
        })?;
        catalog
            .require_provider_report(report)
            .map_err(Error::unlocated)?;
        if !facts.types.has_identity(arena)
            || facts.definitions.len() != report.resolution.definitions.len()
            || !facts.functions.is_empty()
            || !facts.lifetimes.callables.is_empty()
            || facts
                .crates
                .as_deref()
                .and_then(crate::sema::crates::Linkage::origin_catalog)
                .is_none_or(|actual| !std::ptr::eq(actual, reservation.catalog().as_ref()))
        {
            return Err(Error::unlocated(
                "private consumer changed its actual reserved store/header authority",
            ));
        }
        Ok(())
    }
}
