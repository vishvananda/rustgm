//! Declaration projection selections resume with their original immutable environment.
use super::{Checker, Error, NodeKey, Request, Selected, Selection, regions};
use crate::{
    sema::{
        nominals::traits::{Projected, normal_form},
        traits::{
            implementations::Availability,
            obligations::{Obligations, Reservation},
            selection::Source,
        },
    },
    types::parameters::predicates::{Bound, Predicate},
};

#[derive(Default)]
struct Delivery {
    deferred: bool,
    template: bool,
    reservation: Option<Reservation>,
}
pub(super) struct Pending {
    query: Box<normal_form::Query>,
    delivery: Delivery,
}
struct Attempt {
    progress: normal_form::Progress,
    delivery: Delivery,
}

impl Checker<'_> {
    pub(super) fn select_projection(
        &mut self,
        key: NodeKey,
        request: Request,
    ) -> Result<Selection, Error> {
        let context = self
            .projections
            .roots
            .get(&key)
            .ok_or_else(|| self.error(key, "missing projection scope"))?
            .context;
        let Some(Attempt {
            progress,
            mut delivery,
        }) = self.projection_attempt(key, request, context)?
        else {
            return Ok(Selection::Deferred);
        };
        let projected = match progress {
            normal_form::Progress::Ready(projected) => projected,
            normal_form::Progress::Pending(owner, query) => {
                if let regions::Position::Local(body) | regions::Position::Constant(body) =
                    context.position
                {
                    delivery
                        .reservation
                        .get_or_insert_with(Default::default)
                        .reserve(
                            &mut self.facts,
                            NodeKey { node: body, ..key },
                            key,
                            query.allocation_start()..query.allocation_end(),
                        )
                        .map_err(|e| self.error(key, e))?;
                } else {
                    delivery.template = true;
                }
                self.projections
                    .roots
                    .get_mut(&key)
                    .ok_or_else(|| Error::at(key.file, 0, "missing suspended projection"))?
                    .pending = Some(Pending { query, delivery });
                return Ok(Selection::Header(owner));
            }
        };
        self.finish_projection_selection(key, request, context, projected, delivery)
    }

    fn projection_attempt(
        &mut self,
        key: NodeKey,
        request: Request,
        context: regions::Context,
    ) -> Result<Option<Attempt>, Error> {
        let pending = self
            .projections
            .roots
            .get_mut(&key)
            .ok_or_else(|| Error::at(key.file, 0, "missing projection query"))?
            .pending
            .take();
        if let Some(Pending {
            query,
            mut delivery,
        }) = pending
        {
            let progress = if let Some(reservation) = &mut delivery.reservation {
                let first = reservation
                    .resume(&self.facts)
                    .map_err(|e| self.error(key, e))?;
                query.resume(&mut self.facts, first)
            } else {
                query.resume_template(&mut self.facts)
            }
            .map_err(|e| self.error(key, e))?;
            return Ok(Some(Attempt { progress, delivery }));
        }
        let predicate = Predicate {
            ty: request.application.subject,
            bound: Bound::Trait(request.application.trait_application),
        };
        let deferred = self.may_defer_projection(key, context, predicate)?;
        if deferred
            && predicate
                .contains_projections(&self.facts.types)
                .map_err(|e| self.error(key, e))?
        {
            return Ok(None);
        }
        let (environment, first) = self.scoped_projection_environment(key, request, context)?;
        let progress = normal_form::begin(
            &mut self.facts,
            environment,
            request,
            first,
            deferred,
            Availability::Declarations,
        )
        .map_err(|e| self.error(key, e))?;
        Ok(Some(Attempt {
            progress,
            delivery: Delivery {
                deferred,
                ..Delivery::default()
            },
        }))
    }

    fn finish_projection_selection(
        &mut self,
        key: NodeKey,
        request: Request,
        context: regions::Context,
        projected: Projected,
        delivery: Delivery,
    ) -> Result<Selection, Error> {
        let Some(source) = projected.source else {
            return if delivery.deferred {
                Ok(Selection::Deferred)
            } else {
                Err(self.error(
                    key,
                    "associated type requires an unambiguous implementation",
                ))
            };
        };
        let proof = projected.proof;
        if let Some(resolution) = projected.resolution {
            self.retain_selection_proof(key, request, context, proof, delivery)?;
            return Ok(Selection::Resolved(resolution));
        }
        let Source::Implementation { owner, arguments } = source else {
            return Err(self.error(key, "missing associated assumption normal form"));
        };
        let mut restore = if proof.regions.is_empty() && proof.equalities.is_empty() {
            None
        } else {
            Some(Box::new(
                proof.restoration().map_err(|e| self.error(key, e))?,
            ))
        };
        self.retain_alias_sizes(
            key,
            request,
            context,
            projected.sized,
            restore.as_deref_mut(),
        )?;
        // Arguments use the private selection's original identities. Keep their
        // restoration before rebasing a canonical declaration template.
        self.retain_selection_proof(key, request, context, proof, delivery)?;
        let definition =
            crate::sema::projections::definition(&self.facts, owner, request.application.item)
                .map_err(|e| self.error(key, e))?;
        Ok(Selection::Definition(Selected {
            request,
            owner,
            definition,
            arguments,
            family: projected.family,
            restore,
        }))
    }

    fn retain_selection_proof(
        &mut self,
        key: NodeKey,
        request: Request,
        context: regions::Context,
        mut proof: Obligations,
        delivery: Delivery,
    ) -> Result<(), Error> {
        if let Some(reservation) = delivery.reservation {
            return self.retain_reserved_projection_proof(
                key,
                request,
                context,
                proof,
                reservation,
            );
        }
        if delivery.template {
            let first = self.projection_region_start(key, context)?;
            if first != proof.regions.start {
                proof
                    .rebase(&mut self.facts.types, first)
                    .map_err(|e| self.error(key, e))?;
            }
        }
        self.retain_scoped_projection_proof(key, request, context, proof)
    }
}
