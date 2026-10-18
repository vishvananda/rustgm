//! Solved bodies retain their finalization frontier while field declarations complete.
use super::{
    AdjustmentKind, Checker, Declarations, Dependency, Error, NodeId, NodeKey, Operation,
    RegionEquality, field_storage::Batch,
};
use crate::{
    sema::traits::selection::Source,
    types::{GenericArgs, infer::Solved, parameters::predicates::Predicate},
};
use std::collections::HashMap;

pub(super) struct Finalization {
    solved: Solved,
    predicates: Vec<Predicate>,
    arguments: Vec<GenericArgs>,
    evidence: HashMap<usize, Source>,
    storage: Option<Batch>,
}
impl Checker<'_> {
    pub(super) fn finish(&mut self, body: NodeId) -> Result<Option<Dependency>, Error> {
        if self.state.trait_goals.is_none() {
            for adjustment in &self.state.adjustments {
                if adjustment.kind == AdjustmentKind::Never {
                    self.state
                        .infer
                        .default_never(adjustment.expected)
                        .map_err(|e| self.error(body, e))?;
                }
            }
        }
        let evidence = match self.infer_trait_obligations(body)? {
            super::trait_goals::Progress::Ready(evidence) => evidence,
            super::trait_goals::Progress::Prepare(dependency) => return Ok(Some(dependency)),
        };
        let (solved, predicates, arguments) = std::mem::take(&mut self.state.infer)
            .finish_with_arguments(
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
            )
            .map_err(|e| self.error(body, e))?;
        let storage = self.body_storage(&solved)?;
        self.advance_finalization(Finalization {
            solved,
            predicates,
            arguments,
            evidence,
            storage,
        })
    }
    pub(super) fn resume_finalization(
        &mut self,
        mut finalization: Finalization,
    ) -> Result<Option<Dependency>, Error> {
        if let Some(storage) = &mut finalization.storage {
            storage
                .resume(self.facts)
                .map_err(|e| self.error(self.state.body.node, e))?;
        }
        self.advance_finalization(finalization)
    }
    fn advance_finalization(
        &mut self,
        mut finalization: Finalization,
    ) -> Result<Option<Dependency>, Error> {
        let body = self.state.body.node;
        if let Some(storage) = &mut finalization.storage
            && let Some(requirement) = self.advance_body_storage(storage)?
        {
            self.state.finalization = Some(Box::new(finalization));
            return Ok(Some(Dependency {
                requirement,
                source: self.key(body),
            }));
        }
        if let Some(storage) = finalization.storage.take() {
            storage
                .publish(self.facts, self.state.body)
                .map_err(|e| self.error(body, e))?;
        }
        let Finalization {
            solved,
            predicates,
            arguments,
            evidence,
            ..
        } = finalization;
        self.validate_generic_uses(&solved, &predicates, &evidence)?;
        self.publish_constant_uses(arguments)?;
        self.publish_methods(&solved);
        let body_key = self.key(body);
        self.publish_pattern_types(&solved)?;
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
        self.publish_body_types(body, &solved)?;
        self.publish_constant_callables()?;
        self.publish_index_bounds(&solved)?;
        self.publish_adjustments(&solved, body_key)?;
        for (node, term) in &self.state.calls {
            let key = self.key(*node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid call node"))?
                .operation = Some(Operation::Call {
                signature: solved.types[term.0 as usize],
            });
        }
        for node in &self.state.annotation_checks {
            crate::types::bound_uses::check_pointer(&self.facts.types, self.ty(*node)?)
                .map_err(|e| self.error(*node, e))?;
        }
        self.state.annotation_checks.clear();
        self.state.annotation_children.clear();
        self.state.type_annotations.clear();
        self.validate_operations()?;
        self.validate_assignments()?;
        self.validate_values()?;
        self.publish_reference_alignment()?;
        self.finish_patterns()?;
        // Canonical nominal checks may allocate universal aliases after the main
        // inference graph has finished. Publish them under this body's identity.
        self.facts.region_equalities.extend(
            std::mem::take(&mut self.state.infer.region_equalities)
                .into_iter()
                .map(|(left, right)| RegionEquality {
                    body: body_key,
                    left,
                    right,
                }),
        );
        for node in self.state.touched.drain(..) {
            self.state.terms.remove(NodeKey {
                file: self.state.file,
                node,
            });
            self.state.diverges.remove(node.0 as usize);
        }
        self.state.bindings.clear();
        self.state.checks.clear();
        self.state.negated.clear();
        self.state.adjustments.clear();
        self.state.reference_adjustments.clear();
        self.state.reference_expected.clear();
        self.state.reference_work.clear();
        self.state.reference_steps = 0;
        self.state.break_roots.clear();
        self.state.discarded_adjustments.clear();
        self.state.calls.clear();
        self.state.assignments.clear();
        self.state.breaks.clear();
        self.complete_body();
        Ok(None)
    }
}
