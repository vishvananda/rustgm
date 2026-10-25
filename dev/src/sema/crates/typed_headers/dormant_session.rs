//! An unpaid session moves the genuine report and store; no canonical IDs escape.
use crate::{
    resolve::{
        crates::{ReservationWork, unpaid_provided::Inputs},
        expansion::Report,
        package::Error,
    },
    sema::model::Declarations,
    support::crates::Dependencies,
    types::arena::Identity,
};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../../../tests/invariants/held_stores.rs"]
mod tests;

struct Session {
    // Drop the actual type arena before its real resolver definition owner.
    facts: Declarations,
    inputs: Inputs,
    inventory: Arc<Dependencies>,
    provider_index: u32,
    arena: Identity,
    old_definitions: usize,
}
impl Session {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private unpaid reservation entry awaits the reviewed canonical RecipeCursor consumer; Provided import/publication remains closed"
        )
    )]
    fn prepare(
        report: Report,
        mut facts: Declarations,
        inventory: Arc<Dependencies>,
        provider_index: u32,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        work.charge(1)?;
        let old_definitions = report.resolution.definitions.len();
        if !report.names_resolved || facts.definitions.len() != old_definitions {
            return Err(Error::unlocated(
                "unpaid session requires its actual completed report/facts",
            ));
        }
        facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| Error::unlocated("unpaid session requires actual source policy facts"))?
            .require_provider_report(&report)
            .map_err(Error::unlocated)?;
        for function in &facts.functions {
            work.charge(1)?;
            if report
                .resolution
                .definitions
                .get(function.definition.0 as usize)
                .is_none_or(|definition| definition.node != Some(function.node))
            {
                return Err(Error::unlocated(
                    "unpaid session facts changed their actual source declarations",
                ));
            }
        }
        let rows = inventory
            .artifacts()
            .get(provider_index as usize)
            .and_then(crate::support::crates::Interface::source_descriptor)
            .ok_or_else(|| {
                Error::unlocated("unpaid session lacks its selected actual source data")
            })?
            .rows();
        let expected = old_definitions
            .checked_add(rows.declarations.len())
            .ok_or_else(|| Error::unlocated("unpaid semantic definition count overflow"))?;
        work.reserve(
            size_of::<Self>()
                .saturating_add(Identity::allocation_bytes())
                .saturating_add(
                    facts
                        .definitions
                        .capacity()
                        .saturating_add(rows.declarations.len())
                        .saturating_mul(4 * size_of::<Option<crate::types::TypeId>>()),
                ),
        )?;
        let arena = facts.types.arena_identity();
        let inputs = Inputs::reserve(report, Arc::clone(&inventory), provider_index, work)?;
        if inputs.definitions_len() != expected {
            return Err(Error::unlocated(
                "unpaid reservation changed its complete real definition count",
            ));
        }
        work.charge(expected.saturating_sub(old_definitions))?;
        facts.definitions.resize(expected, None);
        let session = Self {
            facts,
            inputs,
            inventory,
            provider_index,
            arena,
            old_definitions,
        };
        session.require(work)?;
        Ok(session)
    }
    fn require(&self, work: &mut ReservationWork) -> Result<(), Error> {
        work.charge(
            self.facts
                .definitions
                .len()
                .saturating_sub(self.old_definitions)
                .saturating_add(1),
        )?;
        self.inputs
            .require(&self.inventory, self.provider_index, work)?;
        if !self.facts.types.has_identity(&self.arena)
            || self.facts.definitions.len() != self.inputs.definitions_len()
            || self
                .facts
                .definitions
                .get(self.old_definitions..)
                .is_none_or(|cells| cells.iter().any(Option::is_some))
        {
            return Err(Error::unlocated(
                "unpaid session changed its actual arena or reserved semantic slots",
            ));
        }
        Ok(())
    }
}
