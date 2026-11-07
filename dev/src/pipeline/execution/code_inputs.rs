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
        checked: crate::pipeline::cdrop::Checked<'_>,
    ) -> Result<
        (
            Option<const_eval::code::Prepared>,
            const_eval::use_inputs::Uses,
        ),
        &'static str,
    > {
        let cleanup = checked.consume(body, purpose, contract, regions)?;
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
        let mut uses = const_eval::use_inputs::Inputs::prepare_uses(facts, body, row_bytes)?;
        let available = (64 * 1024 * 1024usize).saturating_sub(self.catalog.bytes);
        let fixed = const_eval::code::Code::storage_bytes(body)?;
        let mut extra = available
            .checked_sub(fixed)
            .ok_or("constant execution code exceeds 64 MiB budget")?;
        let mut work = 0usize;
        let code = (|| {
            let cleanup = if let Some(cleanup) = cleanup {
                facts.charge_caller_inputs(body.statements.len(), 0)?;
                work = work.saturating_add(body.statements.len());
                let growth = const_eval::code::Code::cleanup_growth_bound(body)?;
                extra = extra
                    .checked_sub(growth.bytes)
                    .ok_or("constant cleanup footer exceeds storage budget")?;
                let view = crate::pipeline::cdrop::execution_view(
                    facts, body, contract, regions, *cleanup, extra, &mut work,
                )?;
                let bundle =
                    const_eval::ctglue::prepare(facts, body, contract, view, extra, &mut work)?;
                extra = extra
                    .checked_sub(bundle.allocation_bytes())
                    .ok_or("constant cleanup bundle exceeds storage budget")?;
                Some(bundle)
            } else {
                None
            };
            crate::borrow::solved_callables(&mut facts.types, body, regions, extra, &mut work)
                .and_then(|solved| {
                    let plan = const_eval::code::ClosurePlan::collect(
                        &facts.types,
                        solved.body,
                        &mut work,
                    )?;
                    let mut code = if plan.needed() && facts.source_bindings.is_some() {
                        let (types, payments) = facts.caller_input_store()?;
                        const_eval::code::Prepared::from_proved(
                            types,
                            solved.body,
                            solved.targets,
                            solved.storage,
                            available,
                            &mut work,
                            const_eval::code::ClosurePreparation {
                                plan,
                                pay: Some(&|w, b| payments.charge(w, b)),
                            },
                        )?
                    } else {
                        const_eval::code::Prepared::from_proved(
                            &mut facts.types,
                            solved.body,
                            solved.targets,
                            solved.storage,
                            available,
                            &mut work,
                            const_eval::code::ClosurePreparation { plan, pay: None },
                        )?
                    };
                    if let Some(cleanup) = cleanup {
                        code.attach_cleanup(cleanup, available, &mut work)?;
                    }
                    uses.finalize_code(&mut facts.types, code.code(), &mut work)?;
                    Ok(code)
                })
        })();
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
        Ok((Some(code?), uses))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/proved_snapshots.rs"]
mod tests;
