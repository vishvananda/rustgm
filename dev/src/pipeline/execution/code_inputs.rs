//! Code publication owns its snapshot while the exact solved inputs are borrowed.
use super::{Body, Contract, Declarations, Engine, ProofPurpose, const_eval};
impl Engine {
    pub(super) fn publish_code(
        &mut self,
        facts: &mut Declarations,
        mut checked: super::CheckedInput,
        selected: Option<crate::types::instance::Instance>,
    ) -> Result<usize, &'static str> {
        let registration = self.catalog.stage_ordinary(checked.take_code()?)?;
        let index = registration.publish_source(
            facts,
            &mut self.const_calls,
            checked.retained,
            selected,
        )?;
        self.refresh_code_bytes(facts);
        Ok(index)
    }
    pub(super) fn refresh_code_bytes(&self, facts: &mut Declarations) {
        facts.constant_work.code_bytes = self.catalog.bytes.saturating_add(
            self.const_calls
                .as_deref()
                .map_or(0, const_eval::use_inputs::Inputs::retained_bytes),
        );
    }
    pub(super) fn retained_source(
        &self,
        code: Option<&const_eval::code::Prepared>,
        contract: std::sync::Arc<Contract>,
        uses: const_eval::use_inputs::Uses,
    ) -> Result<const_eval::use_inputs::Source, &'static str> {
        let code = code.ok_or("retained caller has no owned executable snapshot")?;
        let available = self.const_calls.as_deref().map_or(
            64 * 1024 * 1024,
            const_eval::use_inputs::Inputs::remaining_bytes,
        );
        const_eval::use_inputs::Source::prepare(code.code(), contract, uses, available)
    }
    pub(super) fn finish_code(
        &self,
        facts: &mut Declarations,
        body: &Body,
        contract: &Contract,
        regions: &crate::borrow::regions::Body,
        purpose: ProofPurpose,
    ) -> Result<
        (
            Option<const_eval::code::Prepared>,
            const_eval::use_inputs::Uses,
        ),
        &'static str,
    > {
        if purpose == ProofPurpose::Original {
            return Ok((None, const_eval::use_inputs::Uses::default()));
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
        let row_bytes = self
            .const_calls
            .as_deref()
            .map_or(
                64 * 1024 * 1024,
                const_eval::use_inputs::Inputs::remaining_bytes,
            )
            .saturating_sub(size_of::<const_eval::use_inputs::Row>())
            .saturating_sub(contract.retained_bytes());
        let uses = const_eval::use_inputs::Inputs::prepare_uses(facts, body, row_bytes)?;
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
        Ok((Some(code), uses))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/proved_snapshots.rs"]
mod tests;
