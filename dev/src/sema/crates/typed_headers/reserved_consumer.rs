//! One genuine consumer lends its selected producers to declaration checking.
use super::{
    CollectedReceipt, ConsumerAuthority, ConsumerReservation, Declarations, Error, HeaderDraft,
    Identity, ProofSession, Report, ReservationWork, SelectedHeaders,
};
use crate::{
    resolve::model::DefId,
    sema::crates::typed_headers::{recipe_cursor::Demand, unpaid_provided::SchemaAuthority},
    support::crates::generics::source as s,
};

pub(in crate::sema) struct SelectedConsumer<'a> {
    reservation: ConsumerReservation,
    completion: Box<CollectedReceipt>,
    selected: SelectedHeaders,
    arena: Identity,
    work: &'a mut ReservationWork,
}
impl HeaderDraft {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the genuine private consumer awaits demanded original/instance execution and encoded Provided admission"
        )
    )]
    pub(super) fn analyze(mut self, work: &mut ReservationWork) -> Result<Box<Self>, Error> {
        self.session.authority.synchronize(work)?;
        self.session.authority.require(work)?;
        work.charge(1)?;
        work.reserve(size_of::<SelectedConsumer<'_>>())?;
        let Self {
            session,
            selected,
            completion,
        } = self;
        let ProofSession { authority } = *session;
        let ConsumerAuthority {
            facts,
            report,
            reservation,
            arena,
        } = authority;
        let completion = completion.ok_or_else(|| {
            Error::unlocated("staged declaration checking omitted its actual collected completion")
        })?;
        let service = Box::new(SelectedConsumer {
            reservation,
            completion,
            selected,
            arena,
            work,
        });
        let (facts, service) =
            crate::sema::declarations::analyze_reserved(&report, facts, service)?;
        let SelectedConsumer {
            reservation,
            completion,
            selected,
            arena,
            work,
        } = *service;
        work.charge(2)?;
        work.reserve(size_of::<Self>().saturating_add(size_of::<ProofSession>()))?;
        Ok(Box::new(Self {
            session: Box::new(ProofSession {
                authority: ConsumerAuthority {
                    facts,
                    report,
                    reservation,
                    arena,
                },
            }),
            selected,
            completion: Some(completion),
        }))
    }
}
impl SelectedConsumer<'_> {
    pub(in crate::sema) fn begin(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        state_bytes: usize,
    ) -> Result<(), Error> {
        self.require(report, facts)?;
        self.work
            .charge(report.resolution.definitions.len().saturating_add(1))?;
        self.work.reserve(state_bytes)?;
        facts.complete_staged_sources(report, self.work)
    }

    pub(in crate::sema) fn sizing(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
    ) -> Result<(), Error> {
        self.require(report, facts)?;
        for kind in [
            crate::support::bindings::LangItem::Sized,
            crate::support::bindings::LangItem::MetaSized,
            crate::support::bindings::LangItem::PointeeSized,
        ] {
            self.work.charge(1)?;
            let role = facts
                .source_bindings
                .as_deref()
                .ok_or_else(|| Error::unlocated("selected sizing lost its actual role catalog"))?
                .role(kind)
                .map_err(Error::unlocated)?
                .ok_or_else(|| {
                    Error::unlocated("selected sizing lacks its actual role declaration")
                })?;
            if !self.definition(report, facts, role.definition)? {
                return Err(Error::unlocated(
                    "selected sizing role is not an actual imported declaration",
                ));
            }
        }
        Ok(())
    }
    pub(in crate::sema) fn is_imported(&self, definition: DefId) -> Result<bool, Error> {
        // Exclusion from eager walking grants no semantic completion.
        self.reservation
            .catalog()
            .declaration_if_imported(definition)
            .map(|origin| origin.is_some())
            .map_err(Error::unlocated)
    }
    fn require(&mut self, report: &Report, facts: &Declarations) -> Result<(), Error> {
        self.reservation.require(report, self.work)?;
        self.work.charge(5)?;
        if !facts.types.has_identity(&self.arena)
            || facts.definitions.len() != report.resolution.definitions.len()
            || facts
                .crates
                .as_deref()
                .and_then(crate::sema::crates::Linkage::origin_catalog)
                .is_none_or(|actual| !std::ptr::eq(actual, self.reservation.catalog().as_ref()))
        {
            return Err(Error::unlocated(
                "staged declaration demand changed its actual consumer authority",
            ));
        }
        facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| {
                Error::unlocated("staged declaration demand lost its actual role catalog")
            })?
            .require_provider_report(report)
            .map_err(Error::unlocated)?;
        self.completion.columns(report, self.work)?;
        Ok(())
    }
    pub(in crate::sema) fn definition(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        definition: DefId,
    ) -> Result<bool, Error> {
        self.require(report, facts)?;
        let catalog = self.reservation.catalog();
        let Some(id) = catalog
            .declaration_if_imported(definition)
            .map_err(Error::unlocated)?
        else {
            return Ok(false);
        };
        let view = self.reservation.recipe_view(report, self.work)?;
        let rows = view.rows()?;
        let (provider, mut owner, _) = catalog
            .declaration_coordinate(id)
            .map_err(Error::unlocated)?;
        let artifact = self
            .reservation
            .inventory()
            .artifacts()
            .get(self.reservation.index() as usize)
            .ok_or_else(|| Error::unlocated("selected declaration lost its actual artifact"))?;
        if provider != artifact.identity() {
            return Err(Error::unlocated(
                "staged declaration demand requires its actual dependency producer",
            ));
        }
        let raw = rows
            .declarations
            .get(owner.0 as usize)
            .ok_or_else(|| Error::unlocated("staged declaration demand lost its actual row"))?;
        catalog
            .validate_declaration(
                id,
                definition,
                crate::sema::crates::typed_headers::mapped_source::slot_bindings::kind(
                    raw.definition.kind,
                ),
            )
            .map_err(Error::unlocated)?;
        let authority = SchemaAuthority {
            report,
            reservation: &self.reservation,
            completion: &self.completion,
        };
        loop {
            self.work.charge(1)?;
            let raw = rows.declarations.get(owner.0 as usize).ok_or_else(|| {
                Error::unlocated("selected declaration owner is outside its actual table")
            })?;
            let status = match rows.header(owner).map_err(Error::unlocated)? {
                Some(s::HeaderRef::Function(_)) => self
                    .selected
                    .method_declaration(facts, &authority, owner, self.work)?,
                Some(s::HeaderRef::Trait(_)) => self
                    .selected
                    .trait_owner(facts, &authority, owner, self.work)?,
                Some(s::HeaderRef::Implementation(_)) => self
                    .selected
                    .implementation_owner(facts, &authority, owner, self.work)?,
                None if raw.definition.kind == s::DefinitionKind::Module => return Ok(true),
                None if matches!(
                    raw.definition.kind,
                    s::DefinitionKind::SelfType | s::DefinitionKind::TypeParameter
                ) =>
                {
                    owner = raw.definition.parent.ok_or_else(|| {
                        Error::unlocated("selected formal lacks its actual declaring owner")
                    })?;
                    continue;
                }
                None => {
                    return Err(Error::unlocated(
                        "selected declaration requires its actual owned-body binding producer",
                    ));
                }
            };
            return match status {
                Demand::Ready(()) => Ok(true),
                Demand::Blocked(_) => Err(Error::unlocated(
                    "selected declaration requires its actual pending producer",
                )),
            };
        }
    }
}
