//! Prepare this body's concrete field graphs before immutable ownership queries.
use super::{
    Checker, Error, NodeId, NodeKey, Requirement, TypeId, projections::evidence::Evidence,
};
use crate::sema::{
    model::Declarations,
    projections::{Cursor, Progress as Normalization},
    stored_fields::{Preparation, Progress, Query},
};
use crate::types::lifetime_proof::Obligations;

impl Checker<'_> {
    /// Inference retains provisional templates that are not body value types.
    /// Demand only published roots; expanding every term creates symbolic field
    /// instances for every prefix of a nested constructor before substitution.
    pub(super) fn body_storage(
        &mut self,
        solved: &crate::types::infer::Solved,
    ) -> Result<Option<Batch>, Error> {
        if !self.facts.aggregates.has_projected_templates() && self.state.pointer_casts.is_empty() {
            return Ok(None);
        }
        let mut roots = vec![self.state.result];
        roots.extend(self.pointer_storage(solved));
        roots.extend(self.pointer_cast_storage(solved)?);
        for &node in &self.state.touched {
            if !self.state.type_annotations.contains(&node) {
                roots.push(solved.types[self.term(node)?.0 as usize]);
            }
        }
        roots.extend(
            self.state
                .call_types
                .field_receivers
                .iter()
                .map(|term| solved.types[term.0 as usize]),
        );
        for (_, method) in &self.state.call_types.methods {
            roots.extend(method.value_types(solved));
        }
        let node = self.state.body.node;
        let first = u32::try_from(self.facts.body_regions.len())
            .ok()
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| self.error(node, "storage proof region identity space exhausted"))?;
        let proof = Obligations::new(first);
        let cursor = Cursor::new(
            None,
            &proof,
            crate::types::lifetime_proof::ScopeId::ROOT,
            self.availability,
        )
        .map_err(|e| self.error(node, e))?;
        Ok(Some(Batch {
            roots: roots.into_iter(),
            query: None,
            cursor,
            evidence: Evidence::new(proof),
            normalizing: false,
        }))
    }

    pub(super) fn advance_body_storage(
        &mut self,
        batch: &mut Batch,
    ) -> Result<Option<Requirement>, Error> {
        let body = self.state.body;
        let result = batch.advance(
            self.facts,
            self.state.storage.get_or_insert_with(Default::default),
        );
        let requirement = result.map_err(|e| self.error(body.node, e))?;
        if requirement.is_some() {
            self.facts.body_work.storage_waits += 1;
            if matches!(requirement, Some(Requirement::Definition(_))) {
                self.facts.body_work.storage_definition_waits += 1;
            }
            batch
                .evidence
                .reserve_normalization(self.facts, body, body, &batch.cursor)
                .map_err(|e| self.error(body.node, e))?;
        }
        Ok(requirement)
    }

    pub(super) fn prepare_storage(&mut self, node: NodeId, ty: TypeId) -> Result<(), Error> {
        if !self.facts.aggregates.has_projected_templates() {
            return self
                .facts
                .aggregates
                .prepare(&mut self.facts.types, ty)
                .map_err(|error| self.error(node, error));
        }
        if self
            .state
            .storage
            .get_or_insert_with(Default::default)
            .completed(self.facts, ty)
            .map_err(|error| self.error(node, error))?
        {
            return Ok(());
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
        let mut session = crate::sema::projections::Concrete::new(
            None,
            &mut proof,
            crate::types::lifetime_proof::ScopeId::ROOT,
            self.availability,
            |_, _| Ok(None),
        )
        .map_err(|error| self.error(node, error))?;
        for &ty in types {
            self.state
                .storage
                .get_or_insert_with(Default::default)
                .prepare(self.facts, ty, &mut session)
                .map_err(|error| self.error(node, error))?;
        }
        proof
            .retain_body(self.facts, self.state.body, self.key(node))
            .map_err(|error| self.error(node, error))
    }
}

/// One shared normalizer and proof traverse all published value roots exactly once.
pub(super) struct Batch {
    roots: std::vec::IntoIter<TypeId>,
    query: Option<Query>,
    cursor: Cursor,
    evidence: Evidence,
    normalizing: bool,
}
impl Batch {
    pub fn resume(&mut self, facts: &Declarations) -> Result<(), &'static str> {
        self.evidence.resume_normalization(facts, &mut self.cursor)
    }
    pub fn publish(self, facts: &mut Declarations, body: NodeKey) -> Result<(), &'static str> {
        self.evidence.publish(facts, body, body)
    }
    fn advance(
        &mut self,
        facts: &mut Declarations,
        preparation: &mut Preparation,
    ) -> Result<Option<Requirement>, &'static str> {
        loop {
            let query = if let Some(query) = &mut self.query {
                query
            } else if let Some(root) = self.roots.next() {
                // A published root needing no proof completes immediately.
                if preparation.completed(facts, root)? {
                    continue;
                }
                self.query.insert(Query::new(root))
            } else {
                return Ok(None);
            };
            if self.normalizing {
                let mut selections = 0;
                let progress = self
                    .cursor
                    .advance(facts, &mut self.evidence.proof, &mut |_, _| {
                        selections += 1;
                        Ok(None)
                    });
                facts.body_work.storage_selections += selections;
                match progress? {
                    Normalization::Definition(definition) => {
                        return Ok(Some(Requirement::Definition(definition)));
                    }
                    Normalization::Dependency(owner) => {
                        return Ok(Some(Requirement::from(owner)));
                    }
                    Normalization::Complete { ty, steps } => {
                        facts.body_work.storage_steps += steps;
                        preparation.normalized(facts, query, ty, steps)?;
                        self.normalizing = false;
                    }
                }
            }
            match preparation.advance(facts, query)? {
                Progress::Complete => {
                    self.query = None;
                }
                Progress::Nominal(item) => return Ok(Some(Requirement::Nominal(item))),
                Progress::Normalize(raw) => {
                    self.cursor.start(facts, raw)?;
                    self.normalizing = true;
                }
            }
        }
    }
}
