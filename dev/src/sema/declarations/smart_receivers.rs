//! Receiver validity consumes actual source traits and retains their lifetime debts.
use super::{Checker, Error, NodeKey, Type, TypeId};
use crate::{
    sema::{compiler_items::arbitrary_self, model::Function},
    support::{hash::HashSet, retained},
};
impl Checker<'_> {
    pub(super) fn validate_smart_receiver(
        &mut self,
        function: &Function,
        key: NodeKey,
        mut current: TypeId,
        target: TypeId,
    ) -> Result<TypeId, Error> {
        let report = self.report;
        let level = arbitrary_self::level(report, key)?;
        let error = |message| crate::sema::model::Declarations::error(report, key, message);
        let environment = self
            .facts
            .generics
            .environments
            .get(&function.definition)
            .cloned();
        let mut proof = self.declaration_proof(function.definition, environment);
        let mut visited = HashSet::default();
        let mut shapes = crate::types::structural::Shapes::default();
        let mut correspondence = crate::types::structural::Memo::default();
        let costs = ScratchCosts::default();
        let result = loop {
            let matches = if current == target {
                true
            } else {
                let before = costs.work.get();
                let result = self.receiver_correspondence(
                    key,
                    current,
                    target,
                    &mut shapes,
                    &mut correspondence,
                    &costs,
                );
                self.facts.body_work.canonical_steps = self
                    .facts
                    .body_work
                    .canonical_steps
                    .saturating_add(costs.work.get().saturating_sub(before));
                result?
            };
            if matches {
                break current;
            }
            if let Some(Type::Parameter(parameter)) = self.facts.types.kind(current)
                && self
                    .facts
                    .types
                    .parameter(parameter)
                    .is_some_and(|p| p.owner.0 == function.definition.0)
            {
                return Err(error("receiver cannot be the method's own type parameter"));
            }
            if visited.len() >= 1_000_000 || visited.contains(&current) {
                return Err(error(
                    "receiver Target chain is cyclic or exceeds its work budget",
                ));
            }
            if visited.len() == visited.capacity() {
                let count = visited.len().saturating_mul(2).max(4);
                crate::sema::header_attributes::reserve_source_contracts(
                    &mut self.facts,
                    1,
                    retained::new_map::<TypeId, ()>(count),
                )?;
                visited
                    .try_reserve(count - visited.len())
                    .map_err(|_| error("receiver visit allocation failed"))?;
            }
            visited.insert(current);
            current = arbitrary_self::next(report, &mut self.facts, &mut proof, current, level)
                .map_err(error)?
                .ok_or_else(|| {
                    error("receiver requires its actual implementation and Target chain to Self")
                })?;
        };
        self.retain_trait_proof(function.definition, key, &mut proof);
        Ok(result)
    }
    /// Fingerprints only choose full rigid comparison; final region proof stays
    /// in `validate_distinct_receiver`. One paid memo serves this actual query.
    fn receiver_correspondence(
        &self,
        key: NodeKey,
        current: TypeId,
        target: TypeId,
        shapes: &mut crate::types::structural::Shapes,
        correspondence: &mut crate::types::structural::Memo,
        costs: &ScratchCosts,
    ) -> Result<bool, Error> {
        if !matches!((self.facts.types.kind(current), self.facts.types.kind(target)),
            (Some(Type::Adt(a,_)), Some(Type::Adt(b,_))) if a == b)
        {
            return Ok(false);
        }
        let has_owner = self.facts.source_bindings.is_some()
            || self
                .facts
                .must_use_contracts
                .as_deref()
                .is_some_and(|owner| owner.headers.is_some());
        let payments = if has_owner {
            Some(crate::sema::header_attributes::caller_payments(
                self.facts.must_use_contracts.as_deref(),
                self.facts.source_bindings.is_some(),
            )?)
        } else {
            None
        };
        let producer = self.report.lint_producer();
        let pay = |work, bytes| {
            costs.charge(work, bytes)?;
            if let Some(payments) = &payments {
                payments.charge(work, bytes)
            } else {
                // True legacy absence keeps the existing bounded matcher path;
                // it creates no source/header or lint authority. Available actual
                // frontend totals remain cumulative across receiver queries.
                producer.as_ref().map_or(Ok(()), |producer| {
                    producer
                        .diagnostic_payload(key, work, bytes)
                        .map_err(|error| error.message)
                })
            }
        };
        let a = shapes
            .get_charged(&self.facts.types, current, &pay)
            .map_err(|e| self.error(key, e))?;
        let b = shapes
            .get_charged(&self.facts.types, target, &pay)
            .map_err(|e| self.error(key, e))?;
        if a != b {
            return Ok(false);
        }
        let mut shape = crate::types::structural::Normalized::<false>::new_charged(
            &self.facts.types,
            &pay,
            std::mem::take(correspondence),
        );
        let found = shape
            .matches_charged(current, target)
            .map_err(|e| self.error(key, e))?;
        *correspondence = shape.into_memo();
        Ok(found)
    }
}

/// Query-local scratch limits supplement the actual cumulative provider payer.
/// They grant no attribute, caller, trait, lifetime or execution permission.
#[derive(Default)]
struct ScratchCosts {
    work: std::cell::Cell<usize>,
    bytes: std::cell::Cell<usize>,
}
impl ScratchCosts {
    fn charge(&self, work: usize, bytes: usize) -> Result<(), &'static str> {
        self.work.set(self.work.get().saturating_add(work));
        self.bytes.set(self.bytes.get().saturating_add(bytes));
        if self.work.get() > crate::support::crates::MAX_COMPONENTS
            || self.bytes.get() > crate::support::crates::MAX_BYTES
        {
            return Err("receiver scratch work or storage budget exceeded");
        }
        Ok(())
    }
}
const _: [(); 16] = [(); size_of::<ScratchCosts>()];
