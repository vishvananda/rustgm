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

#[cfg(test)]
#[path = "../../../tests/invariants/lifetime_contract_constants.rs"]
mod tests;

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
        let roots = self.storage_roots(solved)?;
        let node = self.state.body.node;
        let first = self
            .facts
            .body_regions
            .frontier(self.state.body)
            .map_err(|_| self.error(node, "storage proof region identity space exhausted"))?;
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

    fn storage_roots(&self, solved: &crate::types::infer::Solved) -> Result<Vec<TypeId>, Error> {
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
        Ok(roots)
    }

    /// Declaration-time typing of a demanded body can reach a nominal whose
    /// declaration has not yet published its field template. Without projected
    /// templates no storage batch waits for it, so publication waits here, once
    /// per nominal: a second miss means its declaration cannot publish one.
    pub(super) fn field_template_dependency(
        &mut self,
        solved: &crate::types::infer::Solved,
        waited: &mut Option<crate::types::ItemId>,
    ) -> Result<Option<Requirement>, Error> {
        if matches!(
            self.availability,
            crate::sema::traits::implementations::Availability::Validated
        ) || self.facts.aggregates.has_projected_templates()
            || !self.state.pointer_casts.is_empty()
        {
            return Ok(None);
        }
        let node = self.state.body.node;
        for root in self.storage_roots(solved)? {
            let missing = self
                .facts
                .aggregates
                .prepare_storage_fields_pending(&mut self.facts.types, root)
                .map_err(|error| self.error(node, error))?;
            if let Some(item) = missing {
                if waited.replace(item) == Some(item) {
                    return Err(self.error(node, "missing aggregate field template"));
                }
                return Ok(Some(Requirement::Nominal(item)));
            }
        }
        Ok(None)
    }

    /// A region proof relates generic nominals by their variance and lifetime
    /// requirements. Before the crate-wide inference, a declaration-time body
    /// completes both reached contracts of the nominals in its types, and
    /// waits once per nominal for a field template its declaration has not
    /// published yet. The later crate-wide inference republishes them.
    pub(super) fn lifetime_contract_dependency(
        &mut self,
        solved: &crate::types::infer::Solved,
        waited: &mut Option<crate::types::ItemId>,
    ) -> Result<Option<Requirement>, Error> {
        if !matches!(
            self.availability,
            crate::sema::traits::implementations::Availability::Declarations
        ) {
            return Ok(None);
        }
        // Only a source library demands these early. An ordinary crate keeps
        // its existing order, so its declared type tables do not change.
        if self.report.package.source_core().is_none() {
            return Ok(None);
        }
        let node = self.state.body.node;
        let payments = crate::sema::header_attributes::source_payments(
            self.facts.must_use_contracts.as_deref(),
        )?;
        let pay = |work, bytes| payments.charge(work, bytes);
        let nominals = self
            .facts
            .lifetimes
            .variances
            .unreached_nominals(&self.facts.types, solved.types.iter().copied(), &pay)
            .map_err(|error| self.error(node, error))?;
        for &item in &nominals {
            let unreduced = &self.facts.unreduced;
            let missing = self
                .facts
                .lifetimes
                .variances
                .append_reached_or_pending(
                    &self.facts.types,
                    &self.facts.aggregates,
                    &self.facts.definitions,
                    item,
                    &mut |owner, ordinal, storage| {
                        unreduced.fields(owner, ordinal).unwrap_or(storage)
                    },
                    &pay,
                )
                .map_err(|error| self.error(node, error))?;
            if let Some(item) = missing {
                if waited.replace(item) == Some(item) {
                    return Err(self.error(node, "missing aggregate field template"));
                }
                return Ok(Some(Requirement::Nominal(item)));
            }
        }
        let predicates = &self.facts.generics.predicates;
        for item in nominals {
            self.facts
                .lifetimes
                .demand_reached_outlives(
                    &mut self.facts.types,
                    &self.facts.aggregates,
                    item,
                    |owner| {
                        predicates
                            .get(&crate::resolve::model::DefId(owner.0))
                            .into_iter()
                            .flatten()
                            .map(|predicate| predicate.value)
                    },
                    &pay,
                )
                .map_err(|error| self.error(node, error))?;
        }
        Ok(None)
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
        let first = self
            .facts
            .body_regions
            .frontier(self.state.body)
            .map_err(|_| self.error(node, "storage proof region identity space exhausted"))?;
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
