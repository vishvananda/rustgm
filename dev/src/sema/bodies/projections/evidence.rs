//! Reserve only each newly emitted proof segment before yielding to declaration work.
use crate::{
    resolve::unit::NodeKey,
    sema::model::{BodyRegion, BodyRegionOrigin, Declarations, QuantifiedProof},
    types::lifetime_proof::Obligations,
};

pub(in crate::sema::bodies) struct Evidence {
    pub proof: Obligations,
    reserved: Option<u32>,
}
impl Evidence {
    pub fn new(proof: Obligations) -> Self {
        Self {
            proof,
            reserved: None,
        }
    }
    pub fn resume(&mut self, facts: &Declarations) -> Result<(), &'static str> {
        if self.reserved.is_none() {
            return Err("body projection proof was not reserved before resumption");
        }
        let first = frontier(facts)?;
        self.proof.resume_at(first)?;
        self.reserved = Some(first);
        Ok(())
    }
    pub fn reserve(
        &mut self,
        facts: &mut Declarations,
        body: NodeKey,
        source: NodeKey,
    ) -> Result<(), &'static str> {
        let first = self.reserved.unwrap_or(self.proof.regions.start);
        if frontier(facts)? != first || self.proof.regions.end < first {
            return Err("body projection proof allocation frontier changed");
        }
        let region = BodyRegion {
            body,
            source,
            origin: BodyRegionOrigin::QuantifiedProof,
        };
        facts
            .body_regions
            .extend((first..self.proof.regions.end).map(|_| region));
        self.reserved = Some(self.proof.regions.end);
        Ok(())
    }
    pub fn publish(
        mut self,
        facts: &mut Declarations,
        body: NodeKey,
        source: NodeKey,
    ) -> Result<(), &'static str> {
        if self.reserved.is_none() {
            return self.proof.retain_body(facts, body, source);
        }
        self.reserve(facts, body, source)?;
        if !self.proof.is_empty() {
            facts.quantified_proofs.push(QuantifiedProof {
                body,
                source,
                proof: self.proof,
            });
        }
        Ok(())
    }
}
fn frontier(facts: &Declarations) -> Result<u32, &'static str> {
    facts
        .body_regions
        .len()
        .checked_add(1)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or("body projection region space exhausted")
}
