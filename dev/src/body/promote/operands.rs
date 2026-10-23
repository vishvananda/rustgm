//! Promotion reads retained typed lexical facts without indexing source syntax.
use super::{Engine, Error};
use crate::body::ir::{Rvalue, ValueId};
impl Engine<'_, '_> {
    pub(super) fn immediate_operand(&self, value: ValueId, left: bool) -> Result<bool, Error> {
        let Rvalue::Binary { operands, .. } = self.value(value)?.kind else {
            return Err("promotion operand facts require a binary operation".into());
        };
        if operands == crate::body::ir::BinaryOperands::NotRequired {
            return Err("division lacks retained operand source facts".into());
        }
        Ok(operands.immediate(left))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/operand_origins.rs"]
mod tests;
