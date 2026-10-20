//! Retain suspended candidates and retry body goals only when their inputs change.
use super::{
    Checker, Error, NodeId,
    selection::{self, Caller, Selection, Source},
};
use crate::sema::{
    bodies::{Dependency, Requirement, projections},
    traits::{assumptions::Assumptions, obligations::Reservation},
};
use std::{collections::HashMap, sync::Arc};

pub(in crate::sema::bodies) enum Progress {
    Ready(HashMap<usize, Source>),
    Prepare(Dependency),
}
pub(in crate::sema::bodies) struct Pending {
    goals: Vec<usize>,
    cursor: usize,
    current: Option<usize>,
    evidence: HashMap<usize, Source>,
    assumptions: Arc<Assumptions>,
    query: Option<Box<selection::Pending>>,
    reservation: Reservation,
    partial: HashMap<usize, (Box<selection::Pending>, Reservation)>,
}
impl Checker<'_> {
    pub(in crate::sema::bodies) fn infer_trait_obligations(
        &mut self,
        body: NodeId,
    ) -> Result<Progress, Error> {
        let mut pending = if let Some(pending) = self.state.trait_goals.take() {
            *pending
        } else {
            let pending: Vec<_> = self
                .state
                .call_types
                .obligations
                .iter()
                .enumerate()
                .filter_map(|(index, (_, predicate))| {
                    (predicate.trait_arguments().is_some()
                        || matches!(
                            predicate.builtin_bound(),
                            Some(crate::types::parameters::predicates::Bound::Auto(_))
                        )
                        || self.state.call_types.bound_regions(index).is_some())
                    .then_some(index)
                })
                .collect();
            if pending.is_empty() && !self.state.projections.active() {
                return Ok(Progress::Ready(HashMap::new()));
            }
            Pending {
                goals: pending,
                cursor: 0,
                current: None,
                evidence: HashMap::new(),
                assumptions: self.trait_assumptions(body)?.unwrap_or_default(),
                query: None,
                reservation: Reservation::default(),
                partial: HashMap::new(),
            }
        };
        loop {
            let index = if let Some(index) = pending.current {
                index
            } else {
                match self.poll_projections(body)? {
                    projections::Advance::Ready(_) => {}
                    projections::Advance::Definition(dependency) => {
                        self.state.trait_goals = Some(Box::new(pending));
                        return Ok(Progress::Prepare(dependency));
                    }
                }
                if let Some(&index) = pending.goals.get(pending.cursor) {
                    pending.cursor += 1;
                    index
                } else if let Some(index) = self
                    .state
                    .notifications
                    .next_goal(&mut self.state.infer)
                    .map_err(|e| self.error(body, e))?
                {
                    index
                } else {
                    if !self.state.notifications.waiting() && !self.state.projections.active() {
                        break;
                    }
                    if self
                        .state
                        .infer
                        .default_numbers()
                        .map_err(|e| self.error(body, e))?
                    {
                        continue;
                    }
                    let Some(index) = self.state.notifications.first_pending() else {
                        return Err(
                            self.error(body, "associated projection inputs remain unresolved")
                        );
                    };
                    let node = self.state.call_types.obligations[index].0;
                    return Err(self.error(node, "trait obligation requires an unambiguous matching assumption or implementation"));
                }
            };
            pending.current = Some(index);
            let first = self.trait_region_start(body)?;
            if let Some(dependency) = self.advance_trait_goal(&mut pending, index, first)? {
                self.state.trait_goals = Some(Box::new(pending));
                return Ok(Progress::Prepare(dependency));
            }
            pending.current = None;
        }
        Ok(Progress::Ready(pending.evidence))
    }
    fn advance_trait_goal(
        &mut self,
        pending: &mut Pending,
        index: usize,
        first: u32,
    ) -> Result<Option<Dependency>, Error> {
        let node = self.state.call_types.obligations[index].0;
        self.facts.body_work.trait_selections += 1;
        let selected = self.select_pending_goal(pending, index, first)?;
        match selected {
            selection::Progress::Inferred { query } => {
                pending
                    .reservation
                    .reserve(
                        self.facts,
                        self.state.body,
                        self.key(node),
                        first..query.allocation_end(),
                    )
                    .map_err(|e| self.error(node, e))?;
                let reservation = std::mem::take(&mut pending.reservation);
                pending.partial.insert(index, (query, reservation));
                self.observe_trait_goal(node, index)?;
            }
            selection::Progress::Ready(selected) => {
                let reservation = std::mem::take(&mut pending.reservation);
                if let Selection::Proven(proof) = selected {
                    if self.state.call_types.intrinsic(index)
                        && !matches!(
                            proof.source,
                            Source::ResultBound(_) | Source::QuantifiedResultBound { .. }
                        )
                    {
                        return Err(self.error(
                            node,
                            "trait object upcast requires an intrinsic supertrait bound",
                        ));
                    }
                    pending.evidence.insert(
                        index,
                        self.retain_reserved_trait_evidence(node, proof, reservation)?,
                    );
                    self.state
                        .notifications
                        .complete(&mut self.state.infer, index)
                        .map_err(|e| self.error(node, e))?;
                } else {
                    self.observe_trait_goal(node, index)?;
                }
            }
            selection::Progress::Pending(owner, query) => {
                let source = self.key(node);
                pending
                    .reservation
                    .reserve(
                        self.facts,
                        self.state.body,
                        source,
                        first..query.allocation_end(),
                    )
                    .map_err(|e| self.error(node, e))?;
                pending.query = Some(query);
                return Ok(Some(Dependency {
                    requirement: Requirement::from(owner),
                    source,
                }));
            }
        }
        Ok(None)
    }
    fn observe_trait_goal(&mut self, node: NodeId, index: usize) -> Result<(), Error> {
        self.state
            .notifications
            .observe(
                &mut self.state.infer,
                &self.facts.types,
                index,
                &self.state.call_types.obligations[index].1,
            )
            .map_err(|e| self.error(node, e))
    }
    fn select_pending_goal(
        &mut self,
        pending: &mut Pending,
        index: usize,
        first: u32,
    ) -> Result<selection::Progress, Error> {
        let (node, predicate) = &self.state.call_types.obligations[index];
        let node = *node;
        if let Some((query, reservation)) = pending.partial.remove(&index) {
            pending.query = Some(query);
            pending.reservation = reservation;
        }
        let selected = if let Some(query) = pending.query.take() {
            let first = pending
                .reservation
                .resume(self.facts)
                .map_err(|e| self.error(node, e))?;
            query.resume(self.facts, &mut self.state.infer, predicate, first)
        } else if self.state.call_types.assumed_obligations.contains(&index) {
            selection::select_assumption(
                &mut self.facts.types,
                &mut self.state.infer,
                predicate,
                &Caller {
                    assumptions: &pending.assumptions,
                    environment: self.state.method_environment.as_ref(),
                    first_region: first,
                    retain_sized: false,
                },
            )
            .map(selection::Progress::Ready)
            .map_err(|e| e.to_string())
        } else {
            selection::begin_refining(
                self.facts,
                &mut self.state.infer,
                predicate,
                selection::OwnedCaller {
                    assumptions: Arc::clone(&pending.assumptions),
                    environment: self.state.method_environment.clone(),
                    first_region: first,
                    retain_sized: false,
                },
                self.availability,
                self.state.call_types.bound_regions(index),
            )
        }
        .map_err(|e| self.error(node, e))?;
        Ok(selected)
    }
}
