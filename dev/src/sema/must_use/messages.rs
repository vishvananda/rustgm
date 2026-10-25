//! Charge source decoding and selected-contract message formation before allocation.
use super::{
    Declarations, Error, NodeKey, Report, Span, WarningCode, discarded_types::Requirement,
};
const STEPS: usize = 10_000_000;
const BYTES: usize = 64 * 1024 * 1024;
const MESSAGE: usize = 64 * 1024;
#[cfg(test)]
#[path = "../../../tests/invariants/discard_message_budget.rs"]
mod tests;
impl Declarations {
    pub(in crate::sema) fn reserve_must_use_source(
        &mut self,
        key: NodeKey,
        span: Span,
    ) -> Result<(), Error> {
        let length = span
            .end
            .checked_sub(span.start)
            .ok_or_else(|| Error::at(key.file, span.start, "must_use source span is invalid"))?
            as usize;
        let work = &mut self.must_use_contracts.get_or_insert_default().work;
        work.contract_input_steps = work.contract_input_steps.saturating_add(1 + length);
        work.contract_input_bytes = work
            .contract_input_bytes
            .saturating_add(length.saturating_mul(2))
            .saturating_add(
                4 * size_of::<super::Contract>()
                    + crate::support::retained::new_map::<NodeKey, usize>(1),
            );
        if work.contract_input_steps > STEPS || work.contract_input_bytes > BYTES {
            return Err(Error::at(
                key.file,
                span.start,
                "must_use source contract work or storage budget exceeded",
            ));
        }
        Ok(())
    }
    pub(in crate::sema) fn must_use_warning_recorded(
        &self,
        key: NodeKey,
        offset: u32,
        issue: u32,
    ) -> bool {
        self.must_use_contracts.as_deref().is_some_and(|table| {
            table
                .diagnosed
                .contains(&(WarningCode::UnusedMustUse, key, offset, issue))
        })
    }
    pub(in crate::sema) fn must_use_message(
        &mut self,
        report: &Report,
        key: NodeKey,
        requirement: Requirement,
    ) -> Result<String, Error> {
        let offset = report
            .package
            .unit(key.file)
            .configured
            .ast
            .node(key.node)
            .span
            .start;
        let reason = match requirement {
            Requirement::Declaration(definition) => {
                self.must_use_contract(report, definition)?
                    .ok_or_else(|| {
                        Error::at(key.file, offset, "must_use result lost its actual contract")
                    })?
                    .reason
            }
            Requirement::Closure => None,
        };
        let prefix = match requirement {
            Requirement::Closure => "unused closure that must be used",
            Requirement::Declaration(_) if reason.is_some() => "unused must_use result: ",
            Requirement::Declaration(_) => "unused must_use result",
        };
        let length = prefix.len().saturating_add(reason.map_or(0, str::len));
        let work = &mut self.must_use_contracts.get_or_insert_default().work;
        work.message_steps = work.message_steps.saturating_add(1 + length);
        work.message_bytes = work.message_bytes.saturating_add(length);
        if length > MESSAGE || work.message_steps > STEPS || work.message_bytes > BYTES {
            return Err(Error::at(
                key.file,
                offset,
                "must_use diagnostic message work or storage budget exceeded",
            ));
        }
        // Charging changes only cold work counters. Reauthenticate the borrowed
        // reason after that mutation; no reason is cloned to cross the borrow.
        let reason = match requirement {
            Requirement::Declaration(definition) => {
                self.must_use_contract(report, definition)?
                    .ok_or_else(|| {
                        Error::at(key.file, offset, "must_use result lost its actual contract")
                    })?
                    .reason
            }
            Requirement::Closure => None,
        };
        let mut message = String::with_capacity(length);
        message.push_str(prefix);
        if let Some(reason) = reason {
            message.push_str(reason);
        }
        Ok(message)
    }
}
