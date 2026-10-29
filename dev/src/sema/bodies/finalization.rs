//! Solved bodies retain their finalization frontier while field declarations complete.
use super::{
    AdjustmentKind, Checker, Declarations, Dependency, Error, NodeId, NodeKey, RegionEquality,
    field_storage::Batch, hidden_results::HiddenResults,
};
use crate::support::hash::HashMap;
use crate::{
    sema::traits::selection::Source,
    types::{GenericArgs, infer::Solved, parameters::predicates::Predicate},
};

pub(super) enum Finalization {
    Closures(HashMap<usize, Source>),
    Storage(Box<Publication>),
    Caller(Box<Publication>),
    Requirements(HiddenResults),
}
pub(super) struct Publication {
    hidden: HiddenResults,
    solved: Solved,
    predicates: Vec<Predicate>,
    arguments: Vec<GenericArgs>,
    evidence: HashMap<usize, Source>,
    storage: Option<Batch>,
    implementation_cursor: usize,
}
impl Checker<'_> {
    pub(super) fn finish(&mut self, body: NodeId) -> Result<Option<Dependency>, Error> {
        if self
            .state
            .trait_goals
            .as_ref()
            .is_none_or(|pending| !pending.finishing)
        {
            for adjustment in &self.state.adjustments {
                if adjustment.kind == AdjustmentKind::Never {
                    self.state
                        .infer
                        .default_never(
                            adjustment.expected,
                            self.report.package.unit(self.state.file).edition(),
                        )
                        .map_err(|e| self.error(body, e))?;
                }
            }
        }
        self.state
            .infer
            .default_defining_results(&self.facts.types)
            .map_err(|e| self.error(body, e))?;
        let evidence = match self.infer_trait_obligations(body)? {
            super::trait_goals::Progress::Ready(evidence) => evidence,
            super::trait_goals::Progress::Prepare(dependency) => return Ok(Some(dependency)),
        };
        self.finish_nested(evidence)
    }
    pub(super) fn finish_inference(
        &mut self,
        evidence: HashMap<usize, Source>,
    ) -> Result<Option<Dependency>, Error> {
        let body = self.state.body.node;
        self.publish_nested_progress()?;
        let mut infer = std::mem::take(&mut self.state.infer);
        let finished = infer.finish_with_arguments(
            &mut self.facts.types,
            self.state
                .call_types
                .obligations
                .iter()
                .map(|(_, predicate)| predicate),
            self.state
                .call_types
                .constants
                .iter()
                .map(|pending| pending.arguments.as_slice()),
        );
        self.state.infer = infer.recycled();
        let (solved, predicates, arguments) = finished.map_err(|e| self.error(body, e))?;
        self.publish_nested_body(&solved)?;
        let hidden = self.check_opaque_results(&solved)?;
        let storage = self.body_storage(&solved)?;
        self.prepare_publication(Publication {
            hidden,
            solved,
            predicates,
            arguments,
            evidence,
            storage,
            implementation_cursor: 0,
        })
    }
    pub(super) fn resume_finalization(
        &mut self,
        mut finalization: Finalization,
    ) -> Result<Option<Dependency>, Error> {
        if let Finalization::Storage(publication) = &mut finalization
            && let Some(storage) = &mut publication.storage
        {
            storage
                .resume(self.facts)
                .map_err(|e| self.error(self.state.body.node, e))?;
        }
        self.advance_finalization(finalization)
    }
    fn advance_finalization(
        &mut self,
        finalization: Finalization,
    ) -> Result<Option<Dependency>, Error> {
        match finalization {
            Finalization::Closures(evidence) => self.finish_nested(evidence),
            Finalization::Storage(publication) => self.advance_publication(*publication),
            Finalization::Caller(publication) => self.prepare_publication(*publication),
            Finalization::Requirements(hidden) => self.complete_requirements(hidden),
        }
    }
    fn prepare_publication(
        &mut self,
        mut publication: Publication,
    ) -> Result<Option<Dependency>, Error> {
        if let Some(dependency) = self.advance_implementation_proofs(&mut publication)? {
            self.state.finalization = Some(Box::new(Finalization::Caller(Box::new(publication))));
            return Ok(Some(dependency));
        }
        if let Some(dependency) = self.caller_location_dependency(&publication.solved)? {
            self.state.finalization = Some(Box::new(Finalization::Caller(Box::new(publication))));
            return Ok(Some(dependency));
        }
        // Solved calls do not change while storage dependencies are prepared.
        // Keep this completed phase out of every later storage resumption.
        self.advance_publication(publication)
    }
    fn advance_implementation_proofs(
        &mut self,
        publication: &mut Publication,
    ) -> Result<Option<Dependency>, Error> {
        // Declarations also types a genuinely demanded const-function body.
        // The ordinary whole-body pass uses Validated and stays outside here.
        if !matches!(
            self.availability,
            crate::sema::traits::implementations::Availability::Declarations
        ) || self.facts.source_bindings.is_none()
        {
            return Ok(None);
        }
        while publication.implementation_cursor < self.state.call_types.obligations.len() {
            let index = publication.implementation_cursor;
            let node = self.state.call_types.obligations[index].0;
            self.state.steps = self.state.steps.saturating_add(1);
            if self.state.steps > 1_000_000 {
                return Err(self.error(node, "body type work budget exceeded"));
            }
            if let Some(Source::Implementation { owner, .. }) = publication.evidence.get(&index)
                && !self.erased_solved_call(node, &publication.solved)?
                && !self
                    .facts
                    .generics
                    .implementations
                    .as_ref()
                    .is_some_and(|registry| registry.is_validated(*owner))
            {
                return Ok(Some(Dependency {
                    requirement: super::Requirement::ImplementationValidated(*owner),
                    source: self.key(node),
                }));
            }
            publication.implementation_cursor += 1;
        }
        Ok(None)
    }
    fn erased_solved_call(&self, node: NodeId, solved: &Solved) -> Result<bool, Error> {
        match self.kind(node) {
            crate::syntax::ast::Kind::Unary { operand, .. } => {
                let actual = solved.types[self.term(operand)?.0 as usize];
                Ok(self.source_scalar_unary_type(node, actual))
            }
            crate::syntax::ast::Kind::Binary { left, .. } => {
                let actual = solved.types[self.term(left)?.0 as usize];
                Ok(self.source_scalar_binary_solved(node, actual, solved)?
                    || self.source_scalar_comparison_solved(node, solved)?)
            }
            _ => Ok(false),
        }
    }
    fn advance_publication(
        &mut self,
        mut publication: Publication,
    ) -> Result<Option<Dependency>, Error> {
        let body = self.state.body.node;
        if let Some(storage) = &mut publication.storage
            && let Some(requirement) = self.advance_body_storage(storage)?
        {
            self.state.finalization = Some(Box::new(Finalization::Storage(Box::new(publication))));
            return Ok(Some(Dependency {
                requirement,
                source: self.key(body),
            }));
        }
        if let Some(storage) = publication.storage.take() {
            storage
                .publish(self.facts, self.state.body)
                .map_err(|e| self.error(body, e))?;
        }
        let hidden = self.publish_solved_body(publication)?;
        self.complete_requirements(hidden)
    }
    fn complete_requirements(
        &mut self,
        hidden: HiddenResults,
    ) -> Result<Option<Dependency>, Error> {
        if let Some(dependency) = self.advance_canonical_requirements()? {
            self.state.finalization = Some(Box::new(Finalization::Requirements(hidden)));
            return Ok(Some(dependency));
        }
        self.publish_hidden_results(hidden)?;
        if let Some(context) = self.state.canonical_trait_inference.take() {
            self.facts.body_work.canonical_steps += context.steps;
        }
        self.complete_body()?;
        self.clear_body_inference();
        Ok(None)
    }
    fn publish_solved_body(&mut self, publication: Publication) -> Result<HiddenResults, Error> {
        let body = self.state.body.node;
        let Publication {
            solved,
            predicates,
            arguments,
            evidence,
            hidden,
            ..
        } = publication;
        self.facts.body_work.inference_steps += solved.steps;
        self.validate_generic_uses(&solved, &predicates, &evidence)?;
        let constant_dependencies = self.dependency_frontier();
        self.publish_constant_uses(arguments)?;
        let constant_dependencies = self.dependency_append(&constant_dependencies)?;
        self.publish_methods(&solved)?;
        self.publish_for_plans(&solved)?;
        self.publish_invocations(&solved);
        let body_key = self.key(body);
        self.publish_pattern_types(&solved)?;
        let main_start = self.facts.region_equalities.len();
        self.facts
            .region_equalities
            .extend(
                solved
                    .region_equalities
                    .iter()
                    .map(|(left, right)| RegionEquality {
                        body: body_key,
                        left: *left,
                        right: *right,
                    }),
            );
        let main_equalities = main_start..self.facts.region_equalities.len();
        self.publish_body_types(body, &solved)?;
        let callable_dependencies = self.dependency_frontier();
        self.publish_constant_callables()?;
        let callable_dependencies = self.dependency_append(&callable_dependencies)?;
        self.publish_index_bounds(&solved)?;
        self.publish_adjustments(&solved, body_key)?;
        self.publish_calls(&solved)?;
        self.validate_type_privacy()?;
        for node in &self.state.annotation_checks {
            crate::types::bound_uses::check_pointer(&self.facts.types, self.ty(*node)?)
                .map_err(|e| self.error(*node, e))?;
        }
        self.state.annotation_checks.clear();
        self.state.annotation_children.clear();
        self.state.type_annotations.clear();
        self.validate_atomic_orderings()?;
        self.validate_operations()?;
        self.validate_assignments()?;
        self.validate_values()?;
        self.publish_reference_alignment()?;
        self.finish_patterns()?;
        // Canonical nominal checks may allocate universal aliases after the main
        // inference graph has finished. Publish them under this body's identity.
        let late_start = self.facts.region_equalities.len();
        self.facts.region_equalities.extend(
            std::mem::take(&mut self.state.infer.region_equalities)
                .into_iter()
                .map(|(left, right)| RegionEquality {
                    body: body_key,
                    left,
                    right,
                }),
        );
        let late_equalities = late_start..self.facts.region_equalities.len();
        self.canonicalize_dependencies(
            [constant_dependencies, callable_dependencies],
            [main_equalities.clone(), late_equalities.clone()],
        )?;
        if !self.state.call_types.constants.is_empty() {
            self.facts
                .retain_source_caller(
                    body_key,
                    self.state.environment_owner,
                    self.state.body_binders,
                    [main_equalities.clone(), late_equalities.clone()],
                )
                .map_err(|e| Declarations::error(self.report, body_key, e))?;
        }
        self.seal_pattern_equalities(main_equalities, late_equalities)?;
        self.retain_inline_root()?;
        Ok(hidden)
    }
    fn clear_body_inference(&mut self) {
        for node in self.state.touched.drain(..) {
            self.state.terms.remove(NodeKey {
                file: self.state.file,
                node,
            });
            self.state.diverges.remove(node.get() as usize);
        }
        self.state.bindings.clear();
        self.state.checks.clear();
        self.state.negated.clear();
        self.state.adjustments.clear();
        self.state.reference_adjustments.clear();
        self.state.unsizing_adjustments.clear();
        self.state.pointer_adjustments.clear();
        self.state.callable_adjustments.clear();
        self.state.pointer_casts.clear();
        self.state.reference_expected.clear();
        self.state.address_dereferences.clear();
        self.state.raw_address_places.clear();
        self.state.union_write_places.clear();
        self.state.unsizing_contexts.clear();
        self.state.reference_work.clear();
        self.state.reference_steps = 0;
        self.state.break_roots.clear();
        self.state.discarded_adjustments.clear();
        self.state.calls.clear();
        self.state.assignments.clear();
        self.state.breaks.clear();
    }
}
