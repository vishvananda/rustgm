//! Final requirements retain solved inputs and their proof frontier across body dependencies.
use super::{Checker, Error, NodeId, Reservation, Selection, selection};
use crate::{
    sema::bodies::{Dependency, Requirement},
    types::{
        infer::{Instantiation, predicates::Predicate as Goal},
        parameters::clauses::Clause,
    },
};
use std::collections::VecDeque;

#[derive(Default)]
pub(in crate::sema::bodies) struct CanonicalRequirements {
    queued: VecDeque<(NodeId, Clause)>,
    active: Option<Active>,
}
impl CanonicalRequirements {
    pub(in crate::sema::bodies) fn waiting(&self) -> bool {
        self.active.is_some()
    }
}
struct Active {
    node: NodeId,
    goal: Goal,
    query: Box<selection::Pending>,
    reservation: Reservation,
}

impl Checker<'_> {
    pub(in crate::sema::bodies) fn defer_canonical_requirement(
        &mut self,
        node: NodeId,
        clause: Clause,
    ) -> Result<(), Error> {
        let clause = self.alias_clause(node, clause)?;
        if let Clause::UnstableFeature(id) = clause {
            return self.prove_feature_bound(node, id);
        }
        let pending = self
            .state
            .canonical_requirements
            .get_or_insert_with(Default::default);
        if pending.queued.len() >= 1_000_000 {
            return Err(self.error(node, "canonical requirement storage budget exceeded"));
        }
        let ready = pending.active.is_none();
        pending.queued.push_back((node, clause));
        if ready {
            // Preserve immediate proof publication until a real dependency waits.
            // Remaining validators may enqueue work, but never replay this query.
            self.advance_canonical_requirements()?;
        }
        Ok(())
    }

    pub(in crate::sema::bodies) fn advance_canonical_requirements(
        &mut self,
    ) -> Result<Option<Dependency>, Error> {
        let Some(mut pending) = self.state.canonical_requirements.take() else {
            return Ok(None);
        };
        loop {
            let first = self.trait_region_start(self.state.body.node)?;
            let (node, goal, mut reservation, progress) =
                if let Some(mut active) = pending.active.take() {
                    let first = active
                        .reservation
                        .resume(self.facts)
                        .map_err(|e| self.error(active.node, e))?;
                    let context =
                        self.state
                            .canonical_trait_inference
                            .as_mut()
                            .ok_or_else(|| {
                                super::super::Declarations::error(
                                    self.report,
                                    self.state.body,
                                    "missing canonical inference context",
                                )
                            })?;
                    let progress = active
                        .query
                        .resume_reusing(
                            self.facts,
                            context,
                            &active.goal,
                            first,
                            self.state.canonical_trait_proofs.as_deref_mut(),
                        )
                        .map_err(|e| self.error(active.node, e))?;
                    (active.node, active.goal, active.reservation, progress)
                } else if let Some((node, clause)) = pending.queued.pop_front() {
                    let (goal, progress) = self.begin_canonical_requirement(node, clause, first)?;
                    (node, goal, Reservation::default(), progress)
                } else {
                    self.state.canonical_requirements = Some(pending);
                    return Ok(None);
                };
            match progress {
                selection::Progress::Ready(Selection::Proven(evidence)) => {
                    self.retain_reserved_trait_evidence(node, evidence, reservation)?;
                    self.publish_canonical_equalities();
                }
                selection::Progress::Ready(_) => {
                    return Err(self.suspended_trait_failure(
                        node,
                        &goal,
                        "user trait obligation requires a matching assumption or implementation",
                    ));
                }
                selection::Progress::Inferred { .. } => {
                    return Err(
                        self.error(node, "canonical requirement returned incomplete inference")
                    );
                }
                selection::Progress::Pending(dependency, query) => {
                    reservation
                        .reserve(
                            self.facts,
                            self.state.body,
                            self.key(node),
                            first..query.allocation_end(),
                        )
                        .map_err(|e| self.error(node, e))?;
                    pending.active = Some(Active {
                        node,
                        goal,
                        query,
                        reservation,
                    });
                    self.state.canonical_requirements = Some(pending);
                    return Ok(Some(Dependency {
                        requirement: Requirement::from(dependency),
                        source: self.key(node),
                    }));
                }
            }
        }
    }

    fn begin_canonical_requirement(
        &mut self,
        node: NodeId,
        clause: Clause,
        first: u32,
    ) -> Result<(Goal, selection::Progress), Error> {
        let (predicate, count) = match clause {
            Clause::UnstableFeature(_) => {
                return Err(self.error(
                    node,
                    "non-type requirement reached a trait-only continuation",
                ));
            }
            Clause::Predicate(value) => (value, None),
            Clause::ForAll(id) => {
                let value =
                    self.facts.types.quantified(id).ok_or_else(|| {
                        self.error(node, "missing quantified canonical requirement")
                    })?;
                (value.predicate, Some(value.bound_regions))
            }
        };
        let assumptions = self.trait_assumptions(node)?.unwrap_or_default();
        let environment = self.trait_query_environment(node)?;
        let key = self.key(node);
        let context = self
            .state
            .canonical_trait_inference
            .get_or_insert_with(Default::default);
        let error = |message: String| super::super::Declarations::error(self.report, key, message);
        if count.is_none()
            && self.facts.types.nominal_sizes_sealed()
            && matches!(
                self.availability,
                crate::sema::traits::implementations::Availability::Validated
            )
            && selection::Reuse::needed(self.facts, predicate).map_err(|e| error(e.to_owned()))?
        {
            self.state
                .canonical_trait_proofs
                .get_or_insert_with(Default::default);
        }
        let mut substitution = Instantiation::new(context, &self.facts.types, &[])
            .map_err(|e| error(e.to_string()))?;
        let goal = context
            .instantiate_predicate(&mut self.facts.types, predicate, &mut substitution)
            .map_err(|e| error(e.to_string()))?;
        let reuse = self.state.canonical_trait_proofs.as_deref_mut();
        let progress = if let Some(selected) = reuse
            .as_ref()
            .filter(|_| count.is_none())
            .map(|r| r.selection(self.facts, predicate, first))
            .transpose()
            .map_err(error)?
            .flatten()
        {
            context.account_work(1).map_err(|e| error(e.to_string()))?;
            selection::Progress::Ready(selected)
        } else {
            selection::begin_canonical(
                self.facts,
                context,
                &goal,
                selection::OwnedCaller {
                    assumptions,
                    environment,
                    first_region: first,
                    retain_sized: false,
                },
                (self.availability, count),
                reuse,
            )
            .map_err(error)?
        };
        Ok((goal, progress))
    }

    fn publish_canonical_equalities(&mut self) {
        if let Some(context) = &mut self.state.canonical_trait_inference {
            self.facts
                .region_equalities
                .extend(context.region_equalities.drain(..).map(|(left, right)| {
                    crate::sema::model::RegionEquality {
                        body: self.state.body,
                        left,
                        right,
                    }
                }));
        }
    }
}
