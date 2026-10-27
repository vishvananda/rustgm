//! Only a requested inherited result waits for the actual defining-header proof.
use super::{Declarations, Progress, Query, selection};
use crate::sema::{
    projections::Definition,
    specialization::{ProjectionRequirements, ProofInputs},
};

pub(super) enum Continuation {
    Inherited(ProjectionRequirements),
    Metadata(super::data_result::Pending),
    Discriminant(super::tag_result::Pending),
}
impl Continuation {
    pub(super) fn allocation_end(&self, first: u32) -> u32 {
        match self {
            Self::Inherited(value) => value.allocation_end(first),
            Self::Metadata(value) => value.allocation_end(first),
            Self::Discriminant(_) => first,
        }
    }
    fn inherited(&self) -> Result<&ProjectionRequirements, &'static str> {
        match self {
            Self::Inherited(value) => Ok(value),
            Self::Metadata(_) | Self::Discriminant(_) => Err("metadata query used inherited proof"),
        }
    }
    fn inherited_mut(&mut self) -> Result<&mut ProjectionRequirements, &'static str> {
        match self {
            Self::Inherited(value) => Ok(value),
            Self::Metadata(_) | Self::Discriminant(_) => Err("metadata query used inherited proof"),
        }
    }
}
impl Query {
    pub(super) fn prepare_inherited(
        &mut self,
        facts: &mut Declarations,
        source: selection::Source,
    ) -> Result<bool, &'static str> {
        let Some(method) = Definition::method(facts, self.alias, source)? else {
            return Ok(false);
        };
        #[cfg(test)]
        let definition = method.definition();
        self.inherited = match ProjectionRequirements::new(facts, method)? {
            None => None,
            Some(requirements) => {
                requirements.charge(
                    1,
                    size_of::<Continuation>().saturating_sub(size_of::<ProjectionRequirements>()),
                )?;
                Some(Box::new(Continuation::Inherited(requirements)))
            }
        };
        #[cfg(test)]
        if let Some(requirements) = &self.inherited {
            crate::sema::traits::selection::copy_trace::record(|| {
                let Ok(requirements) = requirements.inherited() else {
                    return "invalid inherited continuation".into();
                };
                let (clauses, regions) = requirements.storage_capacities();
                format!(
                    "specialization-rpit-outer defining={definition:?} clauses_capacity={clauses} regions_capacity={regions}"
                )
            });
        }
        Ok(self.inherited.is_some())
    }
    pub(super) fn advance_inherited(
        mut self,
        facts: &mut Declarations,
    ) -> Result<Progress, &'static str> {
        if self.assumptions.is_none() {
            self.inherited
                .as_ref()
                .ok_or("missing inherited result proof")?
                .inherited()?
                .charge(
                    1,
                    size_of::<super::super::Assumptions>() + 2 * size_of::<usize>(),
                )?;
        }
        let caller = self.state.owned_caller(&mut self.assumptions, false);
        let state = &mut self.state;
        let mut inputs = ProofInputs {
            context: &mut state.context,
            proof: &mut state.obligations,
            availability: state.availability,
            frontier: self.frontier,
            caller,
        };
        let requirements = self
            .inherited
            .as_mut()
            .ok_or("missing inherited result proof")?
            .inherited_mut()?;
        if let Some(dependency) = requirements
            .advance(facts, &mut inputs)
            .map_err(|_| "invalid or exhausted inherited result defining-header proof")?
        {
            requirements.charge(1, size_of::<Self>())?;
            return Ok(Progress::Pending(dependency, Box::new(self)));
        }
        self.finish(facts, None)
    }
}
