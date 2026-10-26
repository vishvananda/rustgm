//! Actual fresh consumer reservation and selected append remain opaque.
use super::{recipe_cursor::Demand, unpaid_provided::SelectedHeaders};
use crate::{
    resolve::{
        crates::{ConsumerInput, ConsumerReservation, ReservationWork},
        expansion::Report,
        package::Error,
    },
    sema::model::Declarations,
    support::crates::owner_catalog::DeclarationRef,
    types::arena::Identity,
};

#[cfg(test)]
#[path = "../../../../tests/invariants/consumer_authorities.rs"]
mod tests;

pub(crate) struct ProofSession {
    authority: ConsumerAuthority,
}
/// Fatal errors consume this owner; only an explicit blocked edge is resumable.
pub(crate) struct HeaderDraft {
    session: Box<ProofSession>,
    selected: SelectedHeaders,
}
type HeaderProgress = (Box<HeaderDraft>, Demand<()>);
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
        input: ConsumerInput,
        work: &mut ReservationWork,
    ) -> Result<Box<Self>, Error> {
        work.charge(1)?;
        work.reserve(size_of::<Self>().saturating_add(Identity::allocation_bytes()))?;
        let (report, reservation) = input.into_parts();
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
        self.authority.require(work)?;
        if !self.authority.facts.lifetimes.callables.is_empty() {
            return Err(Error::unlocated(
                "reserved consumer already contains callable registrations",
            ));
        }
        Ok(())
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private selected headers precede real role/member conformance and original-body admission"
        )
    )]
    pub(crate) fn selected_headers(
        self: Box<Self>,
        work: &mut ReservationWork,
    ) -> Result<Box<HeaderDraft>, Error> {
        self.require(work)?;
        work.charge(1)?;
        work.reserve(size_of::<HeaderDraft>())?;
        Ok(Box::new(HeaderDraft {
            session: self,
            selected: SelectedHeaders::default(),
        }))
    }
}
impl ConsumerAuthority {
    fn require(&self, work: &mut ReservationWork) -> Result<(), Error> {
        let ConsumerAuthority {
            facts,
            report,
            reservation,
            arena,
        } = self;
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
impl HeaderDraft {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "selected callable registration grants no header/role/body completion; the genuine retained family still requires conformance"
        )
    )]
    pub(super) fn callable_template(
        mut self: Box<Self>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<HeaderProgress, Error> {
        self.session.authority.require(work)?;
        let ConsumerAuthority {
            facts,
            report,
            reservation,
            ..
        } = &mut self.session.authority;
        let result = self.selected.callable(facts, owner, work, |work| {
            reservation.recipe_view(report, work)
        })?;
        Ok((self, result))
    }
}
