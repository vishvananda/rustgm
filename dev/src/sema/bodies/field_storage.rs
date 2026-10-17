//! Prepare this body's concrete field graphs before immutable ownership queries.
use super::{Checker, Error, NodeId, TypeId};
use crate::types::lifetime_proof::Obligations;

impl Checker<'_> {
    /// Inference retains provisional templates that are not body value types.
    /// Demand only published roots; expanding every term creates symbolic field
    /// instances for every prefix of a nested constructor before substitution.
    pub(super) fn prepare_body_storage(
        &mut self,
        node: NodeId,
        solved: &crate::types::infer::Solved,
    ) -> Result<(), Error> {
        if !self.facts.aggregates.has_projected_templates() {
            return Ok(());
        }
        let mut roots = vec![self.result];
        for &node in &self.touched {
            roots.push(solved.types[self.term(node)?.0 as usize]);
        }
        roots.extend(
            self.call_types
                .nominal_order
                .iter()
                .map(|term| solved.types[term.0 as usize]),
        );
        for (_, method) in &self.call_types.methods {
            roots.extend(method.value_types(solved));
        }
        self.prepare_storage_batch(node, &roots)
    }

    pub(super) fn prepare_storage(&mut self, node: NodeId, ty: TypeId) -> Result<(), Error> {
        if !self.facts.aggregates.has_projected_templates() {
            return self
                .facts
                .aggregates
                .prepare(&mut self.facts.types, ty)
                .map_err(|error| self.error(node, error));
        }
        self.prepare_storage_batch(node, &[ty])
    }

    /// Solved roots share one fold, including overlapping projected field inputs
    /// belonging to different nominal instances in the same body.
    pub(super) fn prepare_storage_batch(
        &mut self,
        node: NodeId,
        types: &[TypeId],
    ) -> Result<(), Error> {
        if !self.facts.aggregates.has_projected_templates() {
            return Ok(());
        }
        let first = u32::try_from(self.facts.body_regions.len())
            .ok()
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| self.error(node, "storage proof region identity space exhausted"))?;
        let mut proof = Obligations::new(first);
        let mut session = crate::sema::projections::concrete(
            &mut proof,
            crate::types::lifetime_proof::ScopeId::ROOT,
        )
        .map_err(|error| self.error(node, error))?;
        for &ty in types {
            self.storage
                .get_or_insert_with(Default::default)
                .prepare(self.facts, ty, &mut session)
                .map_err(|error| self.error(node, error))?;
        }
        proof
            .retain_body(self.facts, self.body, self.key(node))
            .map_err(|error| self.error(node, error))
    }
}
