//! Code publication owns its snapshot while the exact solved inputs are borrowed.
use super::{Body, Contract, Declarations, Engine, ProofPurpose, const_eval};
impl Engine {
    pub(super) fn finish_code(
        &self,
        facts: &mut Declarations,
        body: &Body,
        contract: &Contract,
        regions: &crate::borrow::regions::Body,
        purpose: ProofPurpose,
    ) -> Result<(Option<const_eval::code::Prepared>, bool), &'static str> {
        if purpose == ProofPurpose::Original {
            return Ok((None, false));
        }
        if contract.key != body.key()
            || contract.source != body.source
            || contract.signature != body.owner.signatures().map(|(_, owned)| owned)
            || regions.key != body.key()
            || regions.source != body.source
            || regions.instance != body.instance
        {
            return Err("prepared code requires its exact proved body, contract and instance");
        }
        let needed = const_eval::use_inputs::Inputs::prepare(facts, body)?;
        let available = (64 * 1024 * 1024usize).saturating_sub(self.catalog.bytes);
        let fixed = const_eval::code::Code::storage_bytes(body)?;
        let extra = available
            .checked_sub(fixed)
            .ok_or("constant execution code exceeds 64 MiB budget")?;
        let mut work = 0;
        let targets =
            crate::borrow::solved_callables(&mut facts.types, body, regions, extra, &mut work);
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
        let code = const_eval::code::Prepared::snapshot(body, targets?, available)?;
        Ok((Some(code), needed))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/proved_snapshots.rs"]
mod tests;
