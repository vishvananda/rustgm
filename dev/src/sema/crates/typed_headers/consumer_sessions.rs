//! Actual fresh consumer reservation and selected append remain opaque.
use super::{recipe_cursor::Demand, unpaid_provided::SelectedHeaders};
use crate::sema::crates::authored_export::{CollectedArtifact, CollectedReceipt};
use crate::{
    resolve::{
        crates::{ConsumerInput, ConsumerRequest, ConsumerReservation, ReservationWork},
        expansion::Report,
        package::Error,
    },
    sema::model::Declarations,
    support::crates::owner_catalog::DeclarationRef,
    types::arena::Identity,
};

#[path = "reserved_consumer.rs"]
mod continuation;
pub(in crate::sema) use continuation::SelectedConsumer;

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
    completion: Option<Box<CollectedReceipt>>,
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
            reason = "the real collected-artifact consumer remains private until retained role/member and original/instance conformance completes"
        )
    )]
    pub(super) fn from_collected(
        artifact: CollectedArtifact,
        request: ConsumerRequest<'_, impl crate::resolve::load::Sources>,
        work: &mut ReservationWork,
    ) -> Result<Box<HeaderDraft>, Error> {
        request.require_inventory(artifact.inventory(), work)?;
        let (admission, receipt) = artifact.into_admission(work)?;
        let input = request.admit(admission, work)?;
        Self::new(input, work)?
            .selected_headers(work)?
            .source_headers(receipt, work)
    }
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
        if !self.authority.facts.functions.is_empty() {
            return Err(Error::unlocated(
                "reserved consumer already contains checked source functions",
            ));
        }
        if !self.authority.facts.lifetimes.callables.is_empty() {
            return Err(Error::unlocated(
                "reserved consumer already contains callable registrations",
            ));
        }
        Ok(())
    }
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
            completion: None,
        }))
    }
}
impl ConsumerAuthority {
    fn synchronize(&mut self, work: &mut ReservationWork) -> Result<(), Error> {
        synchronize_headers(&mut self.facts, work)
    }
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
    pub(super) fn source_headers(
        mut self: Box<Self>,
        receipt: CollectedReceipt,
        work: &mut ReservationWork,
    ) -> Result<Box<Self>, Error> {
        self.session.authority.synchronize(work)?;
        self.session.authority.require(work)?;
        work.charge(1)?;
        if self.completion.is_some() {
            return Err(Error::unlocated(
                "selected source header completion cannot be replaced",
            ));
        }
        receipt.columns(&self.session.authority.report, work)?;
        work.reserve(size_of::<CollectedReceipt>())?;
        self.completion = Some(Box::new(receipt));
        Ok(self)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "selected method data awaits actual demanded original/instance execution"
        )
    )]
    pub(super) fn method_declaration(
        mut self: Box<Self>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<HeaderProgress, Error> {
        self.session.authority.synchronize(work)?;
        self.session.authority.require(work)?;
        let completion = self.completion.as_deref().ok_or_else(|| {
            Error::unlocated("selected method omitted its actual collector completion")
        })?;
        let ConsumerAuthority {
            facts,
            report,
            reservation,
            ..
        } = &mut self.session.authority;
        let authority = super::unpaid_provided::SchemaAuthority {
            report,
            reservation,
            completion,
        };
        let result = self
            .selected
            .method_declaration(facts, &authority, owner, work)?;
        self.session.authority.synchronize(work)?;
        Ok((self, result))
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the selected owner transaction awaits its real method and demanded body continuation"
        )
    )]
    pub(super) fn trait_owner(
        mut self: Box<Self>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<HeaderProgress, Error> {
        self.session.authority.synchronize(work)?;
        self.session.authority.require(work)?;
        let completion = self
            .completion
            .as_deref()
            .ok_or_else(|| Error::unlocated("selected owner lacks actual source completion"))?;
        let ConsumerAuthority {
            facts,
            report,
            reservation,
            ..
        } = &mut self.session.authority;
        let authority = super::unpaid_provided::SchemaAuthority {
            report,
            reservation,
            completion,
        };
        let result = self.selected.trait_owner(facts, &authority, owner, work)?;
        self.session.authority.synchronize(work)?;
        Ok((self, result))
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "selected declaration installation remains internal until complete owner/conformance and demanded body proof"
        )
    )]
    pub(super) fn trait_declaration(
        mut self: Box<Self>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<HeaderProgress, Error> {
        self.session.authority.synchronize(work)?;
        self.session.authority.require(work)?;
        let completion = self.completion.as_deref().ok_or_else(|| {
            Error::unlocated("selected declaration lacks its actual collector completion")
        })?;
        let ConsumerAuthority {
            facts,
            report,
            reservation,
            ..
        } = &mut self.session.authority;
        let authority = super::unpaid_provided::SchemaAuthority {
            report,
            reservation,
            completion,
        };
        let result = self
            .selected
            .trait_declaration(facts, &authority, owner, work)?;
        self.session.authority.synchronize(work)?;
        Ok((self, result))
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the selected schema transaction is private until full role/member and demanded-original proof commit"
        )
    )]
    pub(super) fn schema(
        mut self: Box<Self>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<HeaderProgress, Error> {
        self.session.authority.synchronize(work)?;
        self.session.authority.require(work)?;
        let completion = self.completion.as_deref().ok_or_else(|| {
            Error::unlocated("selected schema lacks its actual collector completion")
        })?;
        let ConsumerAuthority {
            facts,
            report,
            reservation,
            ..
        } = &mut self.session.authority;
        let authority = super::unpaid_provided::SchemaAuthority {
            report,
            reservation,
            completion,
        };
        let result = self.selected.schema(facts, &authority, owner, work)?;
        self.session.authority.synchronize(work)?;
        Ok((self, result))
    }
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
        self.session.authority.synchronize(work)?;
        self.session.authority.require(work)?;
        if let Some(completion) = self.completion.as_deref() {
            completion.columns(&self.session.authority.report, work)?;
        }
        let ConsumerAuthority {
            facts,
            report,
            reservation,
            ..
        } = &mut self.session.authority;
        let result = self.selected.callable(facts, owner, work, |work| {
            reservation.recipe_view(report, work)
        })?;
        self.session.authority.synchronize(work)?;
        Ok((self, result))
    }
}

fn synchronize_headers(facts: &mut Declarations, work: &mut ReservationWork) -> Result<(), Error> {
    let Some(headers) = facts
        .crates
        .as_deref_mut()
        .and_then(|linkage| linkage.originals.as_deref_mut())
    else {
        return Ok(());
    };
    if !matches!(
        headers.source,
        Some(crate::sema::crates::ImportedOwners::Provided(_))
    ) {
        return Err(Error::unlocated(
            "private consumer cannot synchronize an ordinary import budget",
        ));
    }
    work.steps = work.steps.max(headers.work);
    work.charge(1)?;
    headers.work = work.steps;
    work.reserve(
        headers
            .base
            .saturating_add(headers.bytes)
            .saturating_sub(work.retained),
    )?;
    headers.bytes = work.retained.saturating_sub(headers.base);
    Ok(())
}
