//! Retain each inference round and its exact suspended selection across header work.
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
    next: Vec<usize>,
    evidence: HashMap<usize, Source>,
    assumptions: Arc<Assumptions>,
    polled: bool,
    progress: bool,
    query: Option<Box<selection::Pending>>,
    reservation: Reservation,
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
                .filter_map(|(index, (_, predicate))| predicate.trait_arguments().map(|_| index))
                .collect();
            if pending.is_empty() && !self.state.projections.active() {
                return Ok(Progress::Ready(HashMap::new()));
            }
            Pending {
                goals: pending,
                cursor: 0,
                next: Vec::new(),
                evidence: HashMap::new(),
                assumptions: self.trait_assumptions(body)?.unwrap_or_default(),
                polled: false,
                progress: false,
                query: None,
                reservation: Reservation::default(),
            }
        };
        while !pending.goals.is_empty() || self.state.projections.active() {
            if !pending.polled {
                match self.poll_projections(body)? {
                    projections::Advance::Ready(progress) => pending.progress |= progress,
                    projections::Advance::Definition(dependency) => {
                        pending.progress = true;
                        self.state.trait_goals = Some(Box::new(pending));
                        return Ok(Progress::Prepare(dependency));
                    }
                }
                pending.polled = true;
            }
            while let Some(&index) = pending.goals.get(pending.cursor) {
                let first = self.trait_region_start(body)?;
                let node = self.state.call_types.obligations[index].0;
                let selected = self.select_pending_goal(&mut pending, index, first)?;
                match selected {
                    selection::Progress::Ready(selected) => {
                        let reservation = std::mem::take(&mut pending.reservation);
                        if let Selection::Proven(proof) = selected {
                            pending.evidence.insert(
                                index,
                                self.retain_reserved_trait_evidence(node, proof, reservation)?,
                            );
                        } else {
                            pending.next.push(index);
                        }
                        pending.cursor += 1;
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
                        self.state.trait_goals = Some(Box::new(pending));
                        return Ok(Progress::Prepare(Dependency {
                            requirement: Requirement::ImplHeader(owner),
                            source,
                        }));
                    }
                }
            }
            if pending.next.len() == pending.goals.len()
                && !pending.progress
                && (!pending.next.is_empty() || self.state.projections.active())
                && !self
                    .state
                    .infer
                    .default_numbers()
                    .map_err(|e| self.error(body, e))?
            {
                let Some(&index) = pending.next.first() else {
                    return Err(self.error(body, "associated projection inputs remain unresolved"));
                };
                let node = self.state.call_types.obligations[index].0;
                return Err(self.error(node, "trait obligation requires an unambiguous matching assumption or implementation"));
            }
            pending.goals.clear();
            std::mem::swap(&mut pending.goals, &mut pending.next);
            pending.cursor = 0;
            pending.polled = false;
            pending.progress = false;
        }
        Ok(Progress::Ready(pending.evidence))
    }
    fn select_pending_goal(
        &mut self,
        pending: &mut Pending,
        index: usize,
        first: u32,
    ) -> Result<selection::Progress, Error> {
        let (node, predicate) = &self.state.call_types.obligations[index];
        let node = *node;
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
            selection::begin(
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
            )
        }
        .map_err(|e| self.error(node, e))?;
        Ok(selected)
    }
}
