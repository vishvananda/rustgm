//! Retain suspended candidates and retry body goals only when their inputs change.
use super::{
    Checker, Error, NodeId,
    selection::{self, Caller, Selection, Source},
};
use crate::sema::{
    bodies::{Dependency, Requirement, projections},
    traits::{assumptions::Assumptions, obligations::Reservation},
};
use crate::support::hash::HashMap;
use std::sync::Arc;

pub(in crate::sema::bodies) enum Progress {
    Ready(HashMap<usize, Source>),
    Prepare(Dependency),
}
pub(in crate::sema::bodies) struct Pending {
    goals: Vec<usize>,
    registered: usize,
    pub(in crate::sema::bodies) finishing: bool,
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
        let Some(mut pending) = self.take_trait_goals(body)? else {
            return Ok(Progress::Ready(HashMap::default()));
        };
        pending.finishing = true;
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
                    return Err(self.call_trait_failure(node, index, "trait obligation requires an unambiguous matching assumption or implementation"));
                }
            };
            if pending.evidence.contains_key(&index) {
                continue;
            }
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
    fn take_trait_goals(&mut self, body: NodeId) -> Result<Option<Pending>, Error> {
        let mut pending = if let Some(pending) = self.state.trait_goals.take() {
            *pending
        } else {
            let goals: Vec<_> = (0..self.state.call_types.obligations.len())
                .filter(|&index| self.inferred_trait_goal(index))
                .collect();
            if goals.is_empty() && !self.state.projections.active() {
                return Ok(None);
            }
            Pending {
                goals,
                registered: self.state.call_types.obligations.len(),
                finishing: false,
                cursor: 0,
                current: None,
                evidence: HashMap::default(),
                assumptions: self.trait_assumptions(body)?.unwrap_or_default(),
                query: None,
                reservation: Reservation::default(),
                partial: HashMap::default(),
            }
        };
        for index in pending.registered..self.state.call_types.obligations.len() {
            self.state
                .infer
                .account_work(1)
                .map_err(|e| self.error(body, e))?;
            if self.inferred_trait_goal(index) {
                pending.goals.push(index);
            }
        }
        pending.registered = self.state.call_types.obligations.len();
        Ok(Some(pending))
    }
    fn inferred_trait_goal(&self, index: usize) -> bool {
        let predicate = &self.state.call_types.obligations[index].1;
        predicate.trait_arguments().is_some()
            || matches!(
                predicate.builtin_bound(),
                Some(crate::types::parameters::predicates::Bound::Auto(_))
            )
            || self.state.call_types.bound_regions(index).is_some()
    }
    pub(in crate::sema::bodies) fn refine_call_inputs(
        &mut self,
        call: NodeId,
        expected: super::super::TermId,
    ) -> Result<Option<Dependency>, Error> {
        let callable = if let crate::syntax::ast::Kind::Call { callee, .. } = self.kind(call) {
            Some(self.term(callee)?)
        } else {
            None
        };
        let mut inputs = std::mem::take(&mut self.state.call_types.input_context);
        let owner = inputs.owner(
            &mut self.state.infer,
            &self.facts.types,
            &self.state.call_types,
            &self.state.adjustments,
            callable,
            call,
        );
        self.state.call_types.input_context = inputs;
        let Some((owner, seeds)) = owner.map_err(|e| self.error(call, e))? else {
            return Ok(None);
        };
        let seeds: Vec<_> = seeds
            .into_iter()
            .filter(|&index| self.inferred_trait_goal(index))
            .collect();
        self.state
            .notifications
            .own(owner, &seeds)
            .map_err(|e| self.error(call, e))?;
        let body = self.state.body.node;
        let Some(mut pending) = self.take_trait_goals(body)? else {
            return Ok(None);
        };
        loop {
            if let projections::Advance::Definition(dependency) = self.poll_projections(body)? {
                self.state.trait_goals = Some(Box::new(pending));
                return Ok(Some(dependency));
            }
            if pending.current.is_none() && self.head(call, expected)?.is_some() {
                break;
            }
            let index = if let Some(index) = pending.current {
                index
            } else {
                let Some(index) = self
                    .state
                    .notifications
                    .next_owned(&mut self.state.infer, owner)
                    .map_err(|e| self.error(call, e))?
                else {
                    break;
                };
                index
            };
            if self
                .state
                .call_types
                .obligations
                .get(index)
                .is_none_or(|(source, _)| *source != owner)
            {
                return Err(self.error(
                    call,
                    "call input requirement belongs to another instantiation",
                ));
            }
            if pending.evidence.contains_key(&index) {
                continue;
            }
            pending.current = Some(index);
            let first = self.trait_region_start(body)?;
            if let Some(dependency) = self.advance_trait_goal(&mut pending, index, first)? {
                self.state.trait_goals = Some(Box::new(pending));
                return Ok(Some(dependency));
            }
            pending.current = None;
        }
        self.state.trait_goals = Some(Box::new(pending));
        Ok(None)
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
            selection::Progress::Inferred { mut query } => {
                let semantic_debt = !query
                    .materialization_only(&mut self.state.infer)
                    .map_err(|reason| self.error(node, reason))?;
                let hints = query
                    .float_hints(self.facts, &mut self.state.infer)
                    .map_err(|reason| self.error(node, reason))?;
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
                self.observe_trait_goal(node, index, semantic_debt)?;
                self.state
                    .notifications
                    .prefer_floats(&mut self.state.infer, &self.facts.types, index, &hints)
                    .map_err(|reason| self.error(node, reason))?;
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
                    self.observe_trait_goal(node, index, true)?;
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
    fn observe_trait_goal(
        &mut self,
        node: NodeId,
        index: usize,
        semantic_debt: bool,
    ) -> Result<(), Error> {
        let float_default = crate::sema::compiler_items::float_default(
            self.facts,
            &mut self.state.infer,
            &self.state.call_types.obligations[index].1,
        )
        .map_err(|reason| self.error(node, reason))?;
        self.state
            .notifications
            .observe(
                &mut self.state.infer,
                &self.facts.types,
                index,
                &self.state.call_types.obligations[index].1,
                float_default.filter(|_| semantic_debt),
            )
            .map_err(|e| self.error(node, e))
    }
    fn select_pending_goal(
        &mut self,
        pending: &mut Pending,
        index: usize,
        first: u32,
    ) -> Result<selection::Progress, Error> {
        self.state
            .notifications
            .clear_hints(&mut self.state.infer, index)
            .map_err(|reason| self.error(self.state.body.node, reason))?;
        let environment =
            self.trait_query_environment(self.state.call_types.obligations[index].0)?;
        if let Some((query, reservation)) = pending.partial.remove(&index) {
            pending.query = Some(query);
            pending.reservation = reservation;
        }
        if pending.query.is_none()
            && !self.state.call_types.assumed_obligations.contains(&index)
            && self.memoizable(pending, index, environment.as_ref())?
        {
            return self.select_memoized(pending, index, first, environment);
        }
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
                    environment: environment.as_ref(),
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
                    environment,
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
    /// A fresh closed goal in a caller without premises: prepares its memo key.
    fn memoizable(
        &mut self,
        pending: &Pending,
        index: usize,
        environment: Option<&Arc<crate::types::parameters::environment::Environment>>,
    ) -> Result<bool, Error> {
        if !matches!(
            self.availability,
            crate::sema::traits::implementations::Availability::Validated
        ) || !self.facts.types.nominal_sizes_sealed()
            || self.state.call_types.bound_regions(index).is_some()
            || !pending.assumptions.is_empty()
            || !environment.is_none_or(|environment| environment.is_neutral())
        {
            return Ok(false);
        }
        let (node, predicate) = &self.state.call_types.obligations[index];
        let node = *node;
        self.facts
            .generics
            .selection_memo
            .prepare(&self.facts.types, &mut self.state.infer, predicate)
            .map_err(|e| self.error(node, e))
    }
    /// Reuse a verified answer, or search while observing its footprint.
    fn select_memoized(
        &mut self,
        pending: &Pending,
        index: usize,
        first: u32,
        environment: Option<Arc<crate::types::parameters::environment::Environment>>,
    ) -> Result<selection::Progress, Error> {
        let (node, predicate) = &self.state.call_types.obligations[index];
        let node = *node;
        if let Some(selected) = self.facts.generics.selection_memo.reuse(first) {
            self.state
                .infer
                .account_work(1)
                .map_err(|e| self.error(node, e))?;
            return Ok(selection::Progress::Ready(selected));
        }
        let mut watch = self.facts.generics.selection_memo.watch(&self.facts.types);
        let mut untouched = false;
        let selected = selection::begin_refining_untouched(
            self.facts,
            &mut self.state.infer,
            predicate,
            selection::OwnedCaller {
                assumptions: Arc::clone(&pending.assumptions),
                environment,
                first_region: first,
                retain_sized: false,
            },
            self.availability,
            &mut untouched,
        )
        .map_err(|e| self.error(node, e))?;
        if let (Some(watch), selection::Progress::Ready(result)) = (&mut watch, &selected) {
            watch.untouched = untouched;
            self.facts
                .generics
                .selection_memo
                .record(&self.facts.types, watch, first, result);
        }
        Ok(selected)
    }
}
