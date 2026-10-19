//! Partial parent proofs retain their exact frontier while a member result infers inputs.
use super::{Checker, Completion, Continuation, Error, Query, Task, WatchId};
use crate::{
    resolve::model::DefId,
    sema::traits::{
        assumptions::Assumptions,
        obligations::Reservation,
        selection::{self, OwnedCaller, Selection},
    },
};
use std::sync::Arc;

pub(super) enum Parent {
    Waiting {
        query: Box<selection::Pending>,
        reservation: Reservation,
        refined: bool,
    },
    Proven,
}
impl Checker<'_> {
    pub(super) fn refine_projection_parent(
        &mut self,
        mut query: Query,
        watch: WatchId,
    ) -> Result<Completion, Error> {
        if query.parent.is_none() {
            if let Some(owner) = self.advance_projection_parent(&mut query, false)? {
                return Ok(Completion::Dependency(
                    super::super::Requirement::ImplHeader(owner),
                    Continuation::Parent {
                        query: Box::new(query),
                        watch,
                    },
                ));
            }
            if query.parent.is_some() {
                self.state
                    .infer
                    .retry_watch(watch)
                    .map_err(|e| self.error(query.node, e))?;
                return Ok(Completion::Waiting(query, true));
            }
        }
        Ok(Completion::Waiting(query, false))
    }

    /// A parent proof may determine inputs, but never discharges member formation.
    /// The original projection remains registered until its whole normal form finishes.
    pub(super) fn advance_projection_parent(
        &mut self,
        query: &mut Query,
        resolved: bool,
    ) -> Result<Option<DefId>, Error> {
        if matches!(query.parent.as_deref(), Some(Parent::Proven))
            || (resolved && query.parent.is_none())
        {
            return Ok(None);
        }
        let input = match query.task {
            Task::Normalize { input, .. } | Task::Symbolic { input, .. } => input,
            Task::Contract(_) => return Ok(None),
        };
        let node = query.node;
        let application = self
            .state
            .infer
            .projection_application(&self.facts.types, input)
            .map_err(|e| self.error(node, e))?;
        let mut reservation = Reservation::default();
        let mut refined = false;
        let first = self.trait_region_start(node)?;
        let selected = if let Some(parent) = query.parent.take() {
            let Parent::Waiting {
                query: pending,
                reservation: retained,
                refined: previous,
            } = *parent
            else {
                return Err(self.error(node, "completed projection parent was resumed"));
            };
            reservation = retained;
            refined = previous;
            let first = reservation
                .resume(self.facts)
                .map_err(|e| self.error(node, e))?;
            pending
                .resume(
                    self.facts,
                    &mut self.state.infer,
                    &application.predicate,
                    first,
                )
                .map_err(|e| self.error(node, e))?
        } else {
            self.start_projection_parent(query, &application.predicate, first)?
        };
        match selected {
            selection::Progress::Ready(Selection::Proven(evidence)) => {
                reservation
                    .publish(evidence.proof, self.facts, self.state.body, self.key(node))
                    .map_err(|e| self.error(node, e))?;
                query.parent = Some(Box::new(Parent::Proven));
                Ok(None)
            }
            selection::Progress::Ready(Selection::Absent | Selection::Deferred) => {
                if refined {
                    return Err(self.error(
                        node,
                        "selected projection parent requirements are not satisfied",
                    ));
                }
                Ok(None)
            }
            selection::Progress::Pending(owner, pending) => {
                reservation
                    .reserve(
                        self.facts,
                        self.state.body,
                        self.key(node),
                        first..pending.allocation_end(),
                    )
                    .map_err(|e| self.error(node, e))?;
                query.parent = Some(Box::new(Parent::Waiting {
                    query: pending,
                    reservation,
                    refined,
                }));
                Ok(Some(owner))
            }
            selection::Progress::Inferred { query: pending } => {
                reservation
                    .reserve(
                        self.facts,
                        self.state.body,
                        self.key(node),
                        first..pending.allocation_end(),
                    )
                    .map_err(|e| self.error(node, e))?;
                query.parent = Some(Box::new(Parent::Waiting {
                    query: pending,
                    reservation,
                    refined: true,
                }));
                Ok(None)
            }
        }
    }
    fn start_projection_parent(
        &mut self,
        query: &Query,
        goal: &crate::types::infer::predicates::Predicate,
        first: u32,
    ) -> Result<selection::Progress, Error> {
        let node = query.node;
        let environment = self.projection_environment(node)?;
        let (assumptions, work) =
            Assumptions::from_environment(&self.facts.types, environment.as_deref())
                .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .account_work(work)
            .map_err(|e| self.error(node, e))?;
        let ambient = self
            .state
            .projections
            .data
            .as_ref()
            .ok_or_else(|| self.error(node, "missing projection scopes"))?
            .scopes
            .scope_plan()
            .counts(query.scope)
            .map_err(|e| self.error(node, e))?;
        selection::begin_scoped_refining(
            self.facts,
            &mut self.state.infer,
            goal,
            OwnedCaller {
                assumptions: Arc::new(assumptions),
                environment,
                first_region: first,
                retain_sized: false,
            },
            self.availability,
            &ambient,
        )
        .map_err(|e| self.error(node, e))
    }
}
