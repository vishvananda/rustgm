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

#[path = "contract_completion.rs"]
mod conformance;

pub(in crate::sema) struct SelectedConsumer<'a> {
    reservation: ConsumerReservation,
    completion: Box<CollectedReceipt>,
    selected: SelectedHeaders,
    arena: Identity,
    work: &'a mut ReservationWork,
}
struct RuntimePreparation<'a, 'work> {
    report: &'a Report,
    selected: &'a mut SelectedConsumer<'work>,
}
impl crate::sema::dispatch::Preparation for RuntimePreparation<'_, '_> {
    fn prepay(&mut self, facts: &mut Declarations, work: usize, bytes: usize) -> Result<(), Error> {
        self.selected.require(self.report, facts)?;
        super::synchronize_headers(facts, self.selected.work)?;
        self.selected.work.charge(work)?;
        let result = self.selected.work.reserve(bytes);
        let synchronized = super::synchronize_headers(facts, self.selected.work);
        result.and(synchronized)
    }
    fn prepare(
        &mut self,
        facts: &mut Declarations,
        dependency: crate::sema::dispatch::PreparationDependency,
    ) -> Result<(), Error> {
        use crate::sema::dispatch::PreparationDependency;
        let result = match dependency {
            PreparationDependency::Header(owner) => self
                .selected
                .definition(self.report, facts, owner)
                .and_then(|imported| {
                    if !imported {
                        return Err(Error::unlocated(
                            "selected header is not an actual imported owner",
                        ));
                    }
                    self.selected
                        .register_implementation(self.report, facts, owner)
                }),
            PreparationDependency::ImplementationValidated(owner) => self
                .selected
                .validate_implementation(self.report, facts, owner),
            _ => Err(Error::unlocated(
                "selected dispatch requires its actual remaining declaration producer",
            )),
        };
        let synchronized = super::synchronize_headers(facts, self.selected.work);
        result.and(synchronized)
    }
}
impl HeaderDraft {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the genuine private consumer awaits demanded original/instance execution and encoded Provided admission"
        )
    )]
    pub(super) fn execute<R>(
        mut self,
        work: &mut ReservationWork,
        execute: impl FnOnce(
            &Report,
            &mut Declarations,
            DefId,
            &mut dyn crate::sema::dispatch::Preparation,
        ) -> Result<R, Error>,
    ) -> Result<(Box<Self>, R), Error> {
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
            mut report,
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
        let entry_name = report
            .package
            .names
            .intern("main")
            .map_err(Error::unlocated)?;
        let (mut facts, service) =
            crate::sema::declarations::analyze_reserved(&report, facts, service)?;
        let SelectedConsumer {
            reservation,
            completion,
            selected,
            arena,
            work,
        } = *service;
        let selection = crate::sema::entry::select_named(&report, &facts, entry_name)?;
        let mut service = SelectedConsumer {
            reservation,
            completion,
            selected,
            arena,
            work,
        };
        let result = execute(
            &report,
            &mut facts,
            selection,
            &mut RuntimePreparation {
                report: &report,
                selected: &mut service,
            },
        )?;
        let SelectedConsumer {
            reservation,
            completion,
            selected,
            arena,
            work,
        } = service;
        work.charge(2)?;
        work.reserve(size_of::<Self>().saturating_add(size_of::<ProofSession>()))?;
        Ok((
            Box::new(Self {
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
            }),
            result,
        ))
    }
}
impl SelectedConsumer<'_> {
    fn register_implementation(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        definition: DefId,
    ) -> Result<(), Error> {
        self.require_imported_implementation(report, facts, definition)?;
        self.work.charge(4)?;
        let owner = facts
            .associated
            .owner(definition)
            .and_then(|id| facts.associated.get(id))
            .copied()
            .ok_or_else(|| Error::unlocated("selected registration lost its actual owner"))?;
        let predicate = facts
            .associated
            .imported_impl_request(&facts.types, owner)
            .map_err(Error::unlocated)?;
        let (application, _) = predicate
            .bound
            .trait_parts()
            .ok_or_else(|| Error::unlocated("selected implementation lost its full application"))?;
        let identity = facts
            .types
            .trait_application(application)
            .ok_or_else(|| Error::unlocated("selected implementation lost its actual trait"))?
            .item
            .into();
        let registry = facts
            .generics
            .implementations
            .as_deref_mut()
            .ok_or_else(|| {
                Error::unlocated("selected implementation lacks its reserved registry")
            })?;
        if let Some(header) = registry.header(definition) {
            if header.source.is_some() || header.predicate != predicate {
                return Err(Error::unlocated(
                    "selected implementation replaced its actual header",
                ));
            }
            return Ok(());
        }
        let before = self.work.retained;
        let (retained, pairs) = {
            let work = std::cell::RefCell::new(&mut *self.work);
            registry
                .prepare_imported_charged(
                    &mut facts.types,
                    &facts.associated,
                    identity,
                    owner,
                    crate::sema::traits::implementations::Implementation {
                        owner: definition,
                        source: None,
                        predicate,
                        methods: Vec::new(),
                        types: Vec::new(),
                        constants: Vec::new(),
                    },
                    &|steps, bytes| {
                        let mut work = work
                            .try_borrow_mut()
                            .map_err(|_| "reentrant selected implementation accounting")?;
                        work.charge(steps)
                            .map_err(|_| "selected implementation work budget exceeded")?;
                        work.reserve_raw(bytes)
                    },
                )
                .map_err(Error::unlocated)?
        };
        self.work.retained = before.saturating_add(retained);
        self.validate_appended_pairs(report, facts, definition, pairs)
    }

    pub(in crate::sema) fn require_imported_implementation(
        &mut self,
        report: &Report,
        facts: &Declarations,
        owner: DefId,
    ) -> Result<(), Error> {
        self.require(report, facts)?;
        self.work.charge(3)?;
        let actual = facts
            .associated
            .owner(owner)
            .and_then(|id| facts.associated.get(id))
            .filter(|actual| {
                matches!(
                    actual.kind,
                    crate::sema::associated::OwnerKind::ImportedImpl(_)
                ) && actual.source.is_none()
                    && actual.self_definition == owner
            })
            .ok_or_else(|| {
                Error::unlocated("selected conformance lost its actual imported impl")
            })?;
        let catalog = self.reservation.catalog();
        catalog
            .validate_declaration(
                catalog
                    .declaration(actual.self_definition)
                    .map_err(Error::unlocated)?,
                owner,
                crate::resolve::model::DefKind::SelfType,
            )
            .map_err(Error::unlocated)?;
        Ok(())
    }

    pub(in crate::sema) fn require_imported_pending(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), Error> {
        self.require(report, facts)?;
        if let Some(registry) = facts.generics.implementations.as_deref() {
            for owner in registry.pending_owners() {
                self.work.charge(2)?;
                let catalog = self.reservation.catalog();
                let id = catalog.declaration(owner).map_err(Error::unlocated)?;
                catalog
                    .validate_declaration(id, owner, crate::resolve::model::DefKind::SelfType)
                    .map_err(Error::unlocated)?;
            }
        }
        // This intentionally changes no registry completion flag. Every
        // unfinished imported candidate remains an ordinary selection dependency.
        Ok(())
    }

    pub(in crate::sema) fn implementation_candidates(
        &mut self,
        report: &Report,
        facts: &Declarations,
        headers: &mut Vec<crate::sema::traits::implementations::PendingHeader>,
    ) -> Result<(), Error> {
        use crate::{
            sema::{associated::OwnerKind, traits::implementations::Implementations},
            support::retained::Growth,
        };
        self.require(report, facts)?;
        let view = self.reservation.recipe_view(report, self.work)?;
        let rows = view.rows()?;
        self.work
            .charge(rows.implementations.len().saturating_add(1))?;
        let growth = Growth::vector::<crate::sema::traits::implementations::PendingHeader>(
            headers.len(),
            headers.capacity(),
            rows.implementations.len(),
        );
        self.work.charge(growth.work)?;
        let count = headers.len().saturating_add(rows.implementations.len());
        if count != 0 {
            self.work.reserve(
                growth
                    .bytes
                    .saturating_add(if facts.generics.implementations.is_none() {
                        size_of::<Implementations>()
                    } else {
                        0
                    })
                    .saturating_add(Implementations::reservation_allocation_bound(count)),
            )?;
        }
        for raw in &rows.implementations {
            let Some(reference) = raw.trait_application else {
                continue;
            };
            self.work.charge(4)?;
            let owner = view.definition(raw.self_definition, self.work)?;
            if let Some(actual) = facts
                .associated
                .owner(owner)
                .and_then(|id| facts.associated.get(id))
            {
                if !matches!(actual.kind, OwnerKind::ImportedImpl(_)) {
                    return Err(Error::unlocated(
                        "selected implementation candidate changed its actual owner kind",
                    ));
                }
                // The ordinary owner walk already included this exact identity.
                continue;
            }
            let application = rows
                .trait_applications
                .get(reference.0 as usize)
                .ok_or_else(|| Error::unlocated("selected candidate lost its trait application"))?;
            if application.declaration.artifact != rows.policy.provider {
                return Err(Error::unlocated(
                    "selected implementation candidate requires its actual dependency trait",
                ));
            }
            let definition = view.definition(application.declaration.declaration, self.work)?;
            let catalog = self.reservation.catalog();
            catalog
                .validate_declaration(
                    catalog.declaration(definition).map_err(Error::unlocated)?,
                    definition,
                    crate::resolve::model::DefKind::Trait,
                )
                .map_err(Error::unlocated)?;
            headers.push(crate::sema::traits::implementations::PendingHeader {
                owner,
                item: crate::types::ItemId(definition.0).into(),
                head: None,
            });
        }
        Ok(())
    }

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
