//! Reserve only each newly emitted proof segment before yielding to declaration work.
use crate::{
    resolve::unit::NodeKey,
    sema::{model::Declarations, traits::obligations::Reservation},
    types::lifetime_proof::Obligations,
};

pub(in crate::sema::bodies) struct Evidence {
    pub proof: Obligations,
    reservation: Reservation,
}
impl Evidence {
    pub fn new(proof: Obligations) -> Self {
        Self {
            proof,
            reservation: Reservation::default(),
        }
    }
    pub fn resume(&mut self, facts: &Declarations) -> Result<(), &'static str> {
        let first = self.reservation.resume(facts)?;
        self.proof.resume_at(first)
    }
    pub fn reserve(
        &mut self,
        facts: &mut Declarations,
        body: NodeKey,
        source: NodeKey,
    ) -> Result<(), &'static str> {
        self.reservation
            .reserve(facts, body, source, self.proof.regions.clone())
    }
    pub fn publish(
        self,
        facts: &mut Declarations,
        body: NodeKey,
        source: NodeKey,
    ) -> Result<(), &'static str> {
        self.reservation.publish(self.proof, facts, body, source)
    }
}
