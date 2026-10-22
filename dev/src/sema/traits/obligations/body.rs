//! Body proofs share the body's allocator and later directed lifetime validation.
use super::Obligations;
use crate::{
    resolve::unit::NodeKey,
    sema::model::{
        BodyRegion, BodyRegionOrigin, Declarations, Outlives, RegionEquality, TypeOutlives,
    },
};

/// A body reservation tracks only its next allocation boundary. The query owns
/// the lifetime proof and records foreign gaps when resuming at this boundary.
#[derive(Default)]
pub(in crate::sema) struct Reservation {
    next: Option<u32>,
}
impl Reservation {
    /// Publish already reserved shared evidence together with its diagnostic table.
    pub fn publish_sourced(
        mut self,
        evidence: crate::sema::model::LifetimeProof,
        facts: &mut Declarations,
        body: NodeKey,
    ) -> Result<(), &'static str> {
        if self.next.is_none() {
            return Err("shared body evidence was not reserved before publication");
        }
        self.reserve(facts, body, evidence.source, evidence.proof.regions.clone())?;
        if !evidence.proof.is_empty() {
            facts
                .quantified_proofs
                .push(crate::sema::model::QuantifiedProof { body, evidence });
        }
        Ok(())
    }
    pub fn resume(&mut self, facts: &Declarations) -> Result<u32, &'static str> {
        if self.next.is_none() {
            return Err("body proof was not reserved before resumption");
        }
        let first = frontier(facts)?;
        if self.next.is_some_and(|next| next > first) {
            return Err("body proof allocation frontier moved backwards");
        }
        self.next = Some(first);
        Ok(first)
    }
    pub fn reserve(
        &mut self,
        facts: &mut Declarations,
        body: NodeKey,
        source: NodeKey,
        allocated: std::ops::Range<u32>,
    ) -> Result<(), &'static str> {
        let first = self.next.unwrap_or(allocated.start);
        if frontier(facts)? != first || allocated.end < first || allocated.end >= 0x8000_0000 {
            return Err("body proof allocation frontier changed");
        }
        let region = BodyRegion {
            body,
            source,
            origin: BodyRegionOrigin::QuantifiedProof,
        };
        facts
            .body_regions
            .extend((first..allocated.end).map(|_| region));
        self.next = Some(allocated.end);
        Ok(())
    }
    /// Preserve ordinary flat source facts after reserving their exact interval.
    pub fn publish_pattern(
        mut self,
        proof: Obligations,
        facts: &mut Declarations,
        body: NodeKey,
        source: NodeKey,
    ) -> Result<(), &'static str> {
        if self.next.is_none() {
            return proof.retain_body(facts, body, source);
        }
        self.reserve(facts, body, source, proof.regions.clone())?;
        if !proof.has_quantified() {
            for id in proof.regions.clone() {
                let region = facts
                    .body_regions
                    .get_mut(id as usize - 1)
                    .ok_or("missing reserved pattern region")?;
                if region.body != body || region.source != source {
                    return Err("reserved pattern region belongs to another owner");
                }
                region.origin = BodyRegionOrigin::Inference;
            }
        }
        proof.publish_body(facts, body, source);
        Ok(())
    }
    pub fn publish(
        mut self,
        proof: Obligations,
        facts: &mut Declarations,
        body: NodeKey,
        source: NodeKey,
    ) -> Result<(), &'static str> {
        if self.next.is_none() {
            return proof.retain_body(facts, body, source);
        }
        self.reserve(facts, body, source, proof.regions.clone())?;
        if !proof.is_empty() {
            facts
                .quantified_proofs
                .push(crate::sema::model::QuantifiedProof {
                    body,
                    evidence: crate::sema::model::LifetimeProof::new(source, proof),
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
        .filter(|n| *n < 0x8000_0000)
        .ok_or("body proof region space exhausted")
}

impl Obligations {
    pub(crate) fn retain_body(
        self,
        facts: &mut Declarations,
        body: NodeKey,
        source: NodeKey,
    ) -> Result<(), &'static str> {
        if self.has_gaps()
            || usize::try_from(self.regions.start).ok() != facts.body_regions.len().checked_add(1)
        {
            return Err("trait proof has foreign body region identities");
        }
        let region = BodyRegion {
            body,
            source,
            origin: if self.has_quantified() {
                BodyRegionOrigin::QuantifiedProof
            } else {
                BodyRegionOrigin::Inference
            },
        };
        facts
            .body_regions
            .extend(self.regions.clone().map(|_| region));
        self.publish_body(facts, body, source);
        Ok(())
    }
    fn publish_body(self, facts: &mut Declarations, body: NodeKey, source: NodeKey) {
        if self.has_quantified() {
            facts
                .quantified_proofs
                .push(crate::sema::model::QuantifiedProof {
                    body,
                    evidence: crate::sema::model::LifetimeProof::new(source, self),
                });
            return;
        }
        facts.region_equalities.extend(
            self.equalities
                .into_iter()
                .map(|(left, right)| RegionEquality { body, left, right }),
        );
        facts
            .outlives
            .extend(self.outlives.into_iter().map(|(longer, shorter)| Outlives {
                body,
                source,
                longer,
                shorter,
            }));
        facts
            .type_outlives
            .extend(self.types.into_iter().map(|(ty, shorter)| TypeOutlives {
                body,
                source,
                ty,
                shorter,
            }));
    }
}
