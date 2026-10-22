//! A suspended pattern dispatcher retains IDs without claiming intervening allocations.
use super::Dispatcher;
use crate::{
    resolve::unit::NodeKey,
    sema::{model::Declarations, traits::obligations::Reservation},
};
impl Dispatcher {
    pub(in crate::sema) fn source_instance(
        &mut self,
        facts: &mut Declarations,
        instance: crate::sema::values::Instance,
    ) -> Result<crate::sema::values::Instance, String> {
        let before = self.normalize.steps;
        let selected = instance.substitute(&mut facts.types, &mut self.normalize)?;
        self.context
            .account_work(self.normalize.steps - before)
            .map_err(|e| e.to_string())?;
        for argument in facts
            .types
            .arguments(selected.arguments)
            .ok_or("missing source arguments")?
        {
            if let crate::types::arguments::Argument::Lifetime(region) = argument
                && region != crate::types::RegionId::STATIC
                && !self.obligations.owns_region(region)
            {
                return Err("source argument is outside the retained caller proof".into());
            }
        }
        Ok(selected)
    }
    pub(in crate::sema) fn source_result(
        &mut self,
        facts: &mut Declarations,
        result: crate::types::TypeId,
    ) -> Result<crate::types::TypeId, String> {
        let before = self.normalize.steps;
        let result = self.normalize.apply(&mut facts.types, result)?;
        self.context
            .account_work(self.normalize.steps - before)
            .map_err(|e| e.to_string())?;
        Ok(result)
    }
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
