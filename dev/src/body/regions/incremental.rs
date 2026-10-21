//! Append-only semantic facts are consumed once, independently of defining-body order.
use super::{Contract, Contracts, Declarations, Kind, NodeKey, Obligation, Proof};
use crate::{
    sema::model::{AdjustmentKind, BodyRegionOrigin, Function},
    types::RegionId,
};
use std::collections::HashMap;

#[derive(Default)]
struct Bucket {
    // Preserve the public contract order even when foreign bodies interleave.
    obligations: [Vec<Obligation>; 6],
    proofs: Vec<Proof>,
}

/// Retained completed contracts and cursors over the shared semantic fact streams.
#[derive(Default)]
pub struct Collector {
    cursor: Cursor,
    pending: HashMap<NodeKey, Bucket>,
    completed: HashMap<NodeKey, Contract>,
}
impl Collector {
    /// Number of semantic facts visited, excluding fixed body-header construction.
    #[must_use]
    pub const fn observations(&self) -> usize {
        self.cursor.observations
    }

    /// Build one completed function's contract without scanning preceding facts again.
    /// # Errors
    /// Rejects missing bodies, inconsistent ownership and late additions to a sealed contract.
    pub fn function(
        &mut self,
        facts: &Declarations,
        function: &Function,
    ) -> Result<&Contract, &'static str> {
        self.update(facts)?;
        let source = function
            .body
            .ok_or("lifetime contract requires a function body")?;
        if !self.completed.contains_key(&source) {
            let mut contract = super::function_contract(facts, function, source)?;
            self.populate(facts, &mut contract);
            super::clause_terms::collect(facts, function.definition, |source, kind| {
                contract.obligations.push(Obligation { source, kind });
                Ok(())
            })?;
            super::opaque_members(facts, function.definition, |source, kind| {
                contract.obligations.push(Obligation { source, kind });
                Ok(())
            })?;
            self.completed.insert(source, contract);
        }
        self.completed
            .get(&source)
            .ok_or("completed lifetime contract disappeared")
    }

    /// Finish in declaration order, reusing every previously built contract.
    /// # Errors
    /// Rejects missing/duplicate owners, late facts and invalid declaration contracts.
    pub fn finish(mut self, facts: &Declarations) -> Result<Contracts<'_>, &'static str> {
        self.update(facts)?;
        let mut bodies = Vec::new();
        let mut index = HashMap::new();
        for function in &facts.functions {
            if let Some(source) = function.body {
                self.function(facts, function)?;
                let contract = self
                    .completed
                    .remove(&source)
                    .ok_or("missing completed function contract")?;
                index.insert(source, bodies.len());
                bodies.push(contract);
            }
        }
        for closure in facts.closures.all() {
            let mut contract = super::closure_contract(facts, closure)?;
            self.populate(facts, &mut contract);
            if index.insert(closure.body, bodies.len()).is_some() {
                return Err("duplicate closure lifetime owner");
            }
            bodies.push(contract);
        }
        for initializer in &facts.initializers {
            let mut contract = super::initializer_contract(facts, initializer)?;
            self.populate(facts, &mut contract);
            index.insert(initializer.value, bodies.len());
            bodies.push(contract);
        }
        for (source, _) in facts.constant_bodies() {
            if index.insert(source, bodies.len()).is_some() {
                return Err("duplicate anonymous constant lifetime owner");
            }
            let mut contract = Contract::constant(source);
            self.populate(facts, &mut contract);
            bodies.push(contract);
        }
        let start = bodies.len();
        super::constructors(facts, &mut bodies, &mut index)?;
        for contract in &mut bodies[start..] {
            self.populate(facts, contract);
        }
        if !self.pending.is_empty() || !self.completed.is_empty() {
            return Err("lifetime obligation has no body owner");
        }
        for source in facts.generics.builtin_requirements.keys() {
            if !index.contains_key(source) {
                return Err("builtin requirement has no body contract");
            }
        }
        Ok(Contracts {
            bodies,
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
        })
    }

    fn populate(&mut self, facts: &Declarations, contract: &mut Contract) {
        if let Some(bucket) = self.pending.remove(&contract.source) {
            contract
                .obligations
                .extend(bucket.obligations.into_iter().flatten());
            contract.proofs = bucket.proofs;
        }
        contract.builtins = facts
            .generics
            .builtin_requirements
            .get(&contract.source)
            .map(|requirements| std::sync::Arc::new(requirements.clone()));
    }

    fn update(&mut self, facts: &Declarations) -> Result<(), &'static str> {
        self.cursor.visit(facts, |body, source, fact| {
            if self.completed.contains_key(&body) {
                return Err("completed lifetime contract received additional facts");
            }
            let bucket = self.pending.entry(body).or_default();
            match fact {
                Fact::Obligation(order, kind) => {
                    bucket.obligations[order].push(Obligation { source, kind });
                }
                Fact::Proof(proof) => bucket.proofs.push(proof.clone()),
            }
            Ok(())
        })
    }
}

pub(super) enum Fact<'a> {
    Obligation(usize, Kind),
    Proof(&'a Proof),
}
#[derive(Default)]
pub(super) struct Cursor {
    cursors: [usize; 7],
    observations: usize,
}
impl Cursor {
    fn frontier(&self, facts: &Declarations) -> Result<[usize; 7], &'static str> {
        let next = [
            facts.body_regions.len(),
            facts.region_equalities.len(),
            facts.outlives.len(),
            facts.coercions.len(),
            facts.well_formed.len(),
            facts.type_outlives.len(),
            facts.quantified_proofs.len(),
        ];
        if self
            .cursors
            .iter()
            .zip(next)
            .any(|(&before, after)| before > after)
        {
            return Err("semantic lifetime facts moved backwards");
        }
        Ok(next)
    }
    pub(super) fn visit(
        &mut self,
        facts: &Declarations,
        mut add: impl FnMut(NodeKey, NodeKey, Fact<'_>) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let next = self.frontier(facts)?;
        for (i, value) in facts.body_regions.iter().enumerate().skip(self.cursors[0]) {
            if !matches!(value.origin, BodyRegionOrigin::QuantifiedProof) {
                add(
                    value.body,
                    value.source,
                    Fact::Obligation(
                        0,
                        Kind::Variable {
                            id: RegionId(
                                u32::try_from(i + 1).map_err(|_| "too many lifetime variables")?,
                            ),
                            needs_provenance: matches!(
                                value.origin,
                                BodyRegionOrigin::ExpressionBorrow
                                    | BodyRegionOrigin::PatternBorrow
                            ),
                        },
                    ),
                )?;
            }
        }
        for value in &facts.region_equalities[self.cursors[1]..] {
            add(
                value.body,
                value.body,
                Fact::Obligation(1, Kind::Equality((value.left, value.right))),
            )?;
        }
        for value in &facts.outlives[self.cursors[2]..] {
            add(
                value.body,
                value.source,
                Fact::Obligation(2, Kind::Outlives((value.longer, value.shorter))),
            )?;
        }
        for value in &facts.coercions[self.cursors[3]..] {
            add(
                value.body,
                value.source,
                Fact::Obligation(
                    3,
                    Kind::Coercion {
                        actual: value.actual,
                        expected: value.expected,
                        unsize: value.kind == AdjustmentKind::ArrayToSlice,
                        reference: value.reference,
                    },
                ),
            )?;
        }
        for value in &facts.well_formed[self.cursors[4]..] {
            add(
                value.body,
                value.source,
                Fact::Obligation(4, Kind::WellFormed(value.ty)),
            )?;
        }
        for value in &facts.type_outlives[self.cursors[5]..] {
            add(
                value.body,
                value.source,
                Fact::Obligation(
                    5,
                    Kind::TypeOutlives {
                        ty: value.ty,
                        shorter: value.shorter,
                    },
                ),
            )?;
        }
        for value in &facts.quantified_proofs[self.cursors[6]..] {
            add(
                value.body,
                value.evidence.source,
                Fact::Proof(&value.evidence),
            )?;
        }
        self.observations += self
            .cursors
            .iter()
            .zip(next)
            .map(|(&before, after)| after - before)
            .sum::<usize>();
        self.cursors = next;
        Ok(())
    }
}
