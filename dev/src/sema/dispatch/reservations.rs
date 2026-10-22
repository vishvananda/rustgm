//! A suspended pattern dispatcher retains IDs without claiming intervening allocations.
use super::Dispatcher;
use crate::{
    resolve::unit::NodeKey,
    sema::{model::Declarations, traits::obligations::Reservation},
};
impl Dispatcher {
    pub(in crate::sema) fn reserve_pattern(
        &self,
        facts: &mut Declarations,
        body: NodeKey,
        reservation: &mut Reservation,
    ) -> Result<(), &'static str> {
        reservation.reserve(facts, body, body, self.obligations.regions.clone())
    }
    pub(in crate::sema) fn resume_pattern(
        &mut self,
        facts: &Declarations,
        reservation: &mut Reservation,
    ) -> Result<(), &'static str> {
        self.obligations.resume_at(reservation.resume(facts)?)
    }
}
