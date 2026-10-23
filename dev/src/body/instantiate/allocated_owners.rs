//! A normalization operation reserves only its own completed caller-proof allocations.
use super::{Declarations, projections};
use crate::{
    body::{
        ir::{Body, BodyKey, ProofSite},
        regions::{Contract, Kind, Proof},
    },
    resolve::unit::NodeKey,
    sema::values::execution::CallerInstance,
    types::TypeId,
};
use std::ops::Range;

struct Allocation {
    proof: usize,
    source: ProofSite,
    regions: Range<u32>,
}
pub(super) struct Operation {
    source: NodeKey,
    key: BodyKey,
    caller: CallerInstance,
    first: u32,
    next: u32,
    originals: Vec<(ProofSite, u32)>,
    rows: Vec<Allocation>,
    bytes: usize,
    work: usize,
}

pub(super) fn normalize(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
    comparison: Option<(TypeId, TypeId)>,
) -> Result<(), &'static str> {
    let Some(mut owner) = Operation::begin(facts, body, contract)? else {
        return projections::normalize(facts, body, contract, comparison);
    };
    let result = (|| {
        let next =
            projections::normalize_with_owner(facts, body, contract, comparison, Some(&mut owner))?;
        owner.finish(facts, body, contract, next)
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(owner.work);
    result
}

impl Operation {
    fn begin(
        facts: &mut Declarations,
        body: &Body,
        contract: &Contract,
    ) -> Result<Option<Self>, &'static str> {
        use crate::sema::values::execution::{caller_published, with_registered_namespace};
        let Some(source) = body.source.source() else {
            return Ok(None);
        };
        if !caller_published(facts, source) {
            return Ok(None);
        }
        if body.source != contract.source || body.key() != contract.key {
            return Err("normalization caller contract belongs to another body");
        }
        let caller = body
            .owner
            .source_caller(facts, source.into(), body.instance)?;
        with_registered_namespace(facts, source, caller, |_, _| Ok(()))?;
        let first = frontier(facts, source)?;
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(contract.obligations.len())
            .saturating_add(contract.proofs.len())
            .saturating_add(1);
        if projections::first_region(contract)? > first {
            return Err("normalization caller has unregistered original proof identities");
        }
        let bytes =
            crate::sema::values::execution::caller_namespace_available(facts).min(64 * 1024 * 1024);
        let original_bytes = contract
            .proofs
            .len()
            .saturating_mul(size_of::<(ProofSite, u32)>());
        if contract.proofs.len() > 1_000_000
            || original_bytes.saturating_add(size_of::<Self>()) > bytes
        {
            return Err("normalization original recorder storage budget exceeded");
        }
        let mut owner = Self {
            source,
            key: body.key(),
            caller,
            first,
            next: first,
            originals: Vec::with_capacity(contract.proofs.len()),
            rows: Vec::new(),
            bytes: bytes
                .saturating_sub(original_bytes)
                .saturating_sub(size_of::<Self>()),
            work: 0,
        };
        let valid = owner.originals(facts, contract);
        facts.body_work.canonical_steps =
            facts.body_work.canonical_steps.saturating_add(owner.work);
        owner.work = 0;
        valid?;
        Ok(Some(owner))
    }
    pub(super) const fn first(&self) -> u32 {
        self.first
    }

    fn originals(&mut self, facts: &Declarations, contract: &Contract) -> Result<(), &'static str> {
        for obligation in &contract.obligations {
            self.charge(1)?;
            if let Kind::Variable { id, .. } = obligation.kind {
                self.original_region(facts, id.0)?;
            }
        }
        for evidence in &contract.proofs {
            self.charge(1)?;
            self.originals
                .push((evidence.source, evidence.proof.regions.end));
            for range in evidence.proof.owned_regions() {
                self.charge(1)?;
                for id in range {
                    self.original_region(facts, id)?;
                }
            }
        }
        Ok(())
    }
    fn original_region(&mut self, facts: &Declarations, id: u32) -> Result<(), &'static str> {
        self.charge(1)?;
        if id == 0
            || !facts
                .body_regions
                .get(self.source, crate::types::RegionId(id))
                .is_some_and(|r| r.body == self.source && r.source == self.source)
        {
            return Err("normalization original proof has foreign caller ownership");
        }
        Ok(())
    }
    pub(super) fn record(
        &mut self,
        proof: usize,
        evidence: &Proof,
        regions: Range<u32>,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        if regions.start != self.next || regions.end < regions.start {
            return Err("normalization allocation record has a gap or overlap");
        }
        if let Some(&(source, end)) = self.originals.get(proof) {
            if source != evidence.source || regions.start < end {
                return Err("normalization resumed proof identity changed");
            }
        } else if evidence.source != self.source.into() {
            return Err("normalization new proof has a foreign source");
        }
        self.coverage(evidence, &regions)?;
        if regions.is_empty() {
            return Ok(());
        }
        if self.rows.len() == self.rows.capacity() {
            let capacity = self.rows.capacity().saturating_mul(2).max(4);
            if capacity > 1_000_000 || capacity.saturating_mul(size_of::<Allocation>()) > self.bytes
            {
                return Err("normalization allocation recorder storage budget exceeded");
            }
            self.rows.reserve_exact(capacity - self.rows.len());
        }
        self.next = regions.end;
        self.rows.push(Allocation {
            proof,
            source: evidence.source,
            regions,
        });
        Ok(())
    }
    fn coverage(&mut self, evidence: &Proof, regions: &Range<u32>) -> Result<(), &'static str> {
        let mut next = regions.start;
        for owned in evidence.proof.owned_regions() {
            self.charge(1)?;
            if next == regions.end {
                return Ok(());
            }
            if owned.end <= next {
                continue;
            }
            if owned.start > next {
                break;
            }
            next = owned.end.min(regions.end);
        }
        if next != regions.end {
            return Err("normalization allocation record is not retained by its proof");
        }
        Ok(())
    }
    fn finish(
        &mut self,
        facts: &mut Declarations,
        body: &Body,
        contract: &Contract,
        next: u32,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        if body.source != self.source.into()
            || body.key() != self.key
            || contract.source != self.source.into()
            || contract.key != self.key
            || body
                .owner
                .source_caller(facts, self.source.into(), body.instance)?
                != self.caller
            || frontier(facts, self.source)? != self.first
            || next != self.next
            || contract.proofs.len() < self.originals.len()
        {
            return Err("normalization allocation owner or frontier changed");
        }
        // Revalidate the exact retained proof positions before publishing any row.
        let rows = std::mem::take(&mut self.rows);
        let result = (|| {
            let mut first = self.first;
            for row in &rows {
                self.charge(1)?;
                let proof = contract
                    .proofs
                    .get(row.proof)
                    .ok_or("normalization allocation proof is stale")?;
                if proof.source != row.source || row.regions.start != first {
                    return Err("normalization allocation proof or source changed");
                }
                self.coverage(proof, &row.regions)?;
                first = row.regions.end;
            }
            if first != next {
                return Err("normalization allocation coverage is incomplete");
            }
            self.charge((next - self.first) as usize)?;
            crate::sema::values::execution::reserve_normalization(
                facts,
                self.source,
                self.caller,
                self.first..next,
            )
        })();
        self.rows = rows;
        result
    }
    fn charge(&mut self, amount: usize) -> Result<(), &'static str> {
        self.work = self.work.saturating_add(amount);
        if self.work > 1_000_000 {
            return Err("normalization caller allocation work budget exceeded");
        }
        Ok(())
    }
}
/// The source body's next region identity: published normalization regions are
/// reserved in that body's numbering unit.
fn frontier(facts: &Declarations, body: NodeKey) -> Result<u32, &'static str> {
    facts
        .body_regions
        .frontier(body)
        .ok()
        .filter(|&first| first < 0x8000_0000)
        .ok_or("normalization allocation identity space exhausted")
}

#[cfg(test)]
#[path = "../../../tests/invariants/allocated_proofs.rs"]
mod tests;
