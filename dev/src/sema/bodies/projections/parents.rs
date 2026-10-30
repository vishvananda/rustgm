//! Partial parent proofs retain their exact frontier while a member result infers inputs.
use super::{Checker, Completion, Continuation, Error, Query, Task, WatchId};
use crate::sema::traits::{
    assumptions::Assumptions,
    obligations::Reservation,
    selection::{self, OwnedCaller, Selection},
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
            if let Some(owner) = self.advance_projection_parent(&mut query, watch, false)? {
                return Ok(Completion::Dependency(
                    super::super::Requirement::from(owner),
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

    pub(super) fn range_parent_readiness(&mut self, query: &Query) -> Result<bool, Error> {
        if self.report.resolution.builtin_traits.ranges().is_none()
            || self
                .facts
                .iteration
                .as_deref()
                .and_then(|r| r.ranges.as_deref())
                .is_some_and(|r| r.complete)
        {
            return Ok(false);
        }
        let input = match query.task {
            Task::Normalize { input, .. } | Task::Symbolic { input, .. } => input,
            Task::Contract(_) => return Ok(false),
        };
        let application = self
            .state
            .infer
            .projection_application(&self.facts.types, input)
            .map_err(|e| self.error(query.node, e))?;
        let canonical = self
            .facts
            .iteration
            .as_deref()
            .ok_or_else(|| self.error(query.node, "missing iteration contracts"))?
            .canonical;
        if !application
            .predicate
            .trait_arguments()
            .is_some_and(|(item, _)| {
                canonical
                    .declarations
                    .iter()
                    .any(|d| d.definition.0 == item.0)
            })
        {
            return Ok(false);
        }
        crate::sema::iteration::range_originals::requested_owner(
            self.report,
            &self.facts.types,
            &mut self.state.infer,
            application.predicate.subject(),
        )
        .map(|owner| owner.is_some())
        .map_err(|e| self.error(query.node, e))
    }

    fn parent_application(
        &mut self,
        query: &Query,
        resolved: bool,
    ) -> Result<Option<crate::types::infer::projections::Application>, Error> {
        let proven = matches!(query.parent.as_deref(), Some(Parent::Proven));
        let latent_ranges = self.report.resolution.builtin_traits.ranges().is_some()
            && !self
                .facts
                .iteration
                .as_deref()
                .and_then(|r| r.ranges.as_deref())
                .is_some_and(|r| r.complete);
        if !latent_ranges && (proven || (resolved && query.parent.is_none())) {
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
        if (proven || (resolved && query.parent.is_none()))
            && self
                .facts
                .generics
                .implementations
                .as_ref()
                .map(|registry| {
                    registry.selection_goal_ready(
                        &self.facts.types,
                        &mut self.state.infer,
                        &application.predicate,
                    )
                })
                .transpose()
                .map_err(|e| self.error(node, e))?
                .unwrap_or(true)
        {
            return Ok(None);
        }
        Ok(Some(application))
    }

    /// A parent proof may determine inputs, but never discharges member formation.
    /// The original projection remains registered until its whole normal form finishes.
    pub(super) fn advance_projection_parent(
        &mut self,
        query: &mut Query,
        watch: WatchId,
        resolved: bool,
    ) -> Result<Option<selection::Dependency>, Error> {
        self.state
            .notifications
            .clear_projection_hints(&mut self.state.infer, watch)
            .map_err(|reason| self.error(query.node, reason))?;
        let Some(application) = self.parent_application(query, resolved)? else {
            return Ok(None);
        };
        let proven = matches!(query.parent.as_deref(), Some(Parent::Proven));
        let node = query.node;
        if proven {
            // Input refinement can reveal a latent matching canonical header
            // after an earlier parent proof. Keep that proof, but recheck the
            // current complete goal through the retained readiness protocol.
            query.parent = None;
        }
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
            selection::Progress::Ready(Selection::Refuted(refutation)) => {
                refutation.reject().map_err(|e| self.error(node, e))?;
                Err(self.error(node, "projection parent is explicitly refuted"))
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
            selection::Progress::Inferred { query: mut pending } => {
                self.observe_parent_hints(node, watch, &mut pending)?;
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
    fn observe_parent_hints(
        &mut self,
        node: super::NodeId,
        watch: WatchId,
        pending: &mut selection::Pending,
    ) -> Result<(), Error> {
        let hints = pending
            .float_hints(self.facts, &mut self.state.infer)
            .map_err(|reason| self.error(node, reason))?;
        self.state
            .notifications
            .prefer_projection_floats(&mut self.state.infer, &self.facts.types, watch, &hints)
            .map_err(|reason| self.error(node, reason))
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
