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
    pub fn resume_normalization(
        &mut self,
        facts: &Declarations,
        cursor: &mut crate::sema::projections::Cursor,
    ) -> Result<(), &'static str> {
        let first = self.reservation.resume(facts)?;
        cursor.resume_at(&mut self.proof, first)
    }

    pub fn reserve_normalization(
        &mut self,
        facts: &mut Declarations,
        body: NodeKey,
        source: NodeKey,
        cursor: &crate::sema::projections::Cursor,
    ) -> Result<(), &'static str> {
        self.reservation.reserve(
            facts,
            body,
            source,
            self.proof.regions.start..cursor.allocation_end(&self.proof),
        )
    }

    pub fn new(proof: Obligations) -> Self {
        Self {
            proof,
            reservation: Reservation::default(),
        }
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
