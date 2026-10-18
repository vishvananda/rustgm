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
                    source,
                    proof,
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
        if self.has_quantified() {
            facts
                .quantified_proofs
                .push(crate::sema::model::QuantifiedProof {
                    body,
                    source,
                    proof: self,
                });
            return Ok(());
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
        Ok(())
    }
}
