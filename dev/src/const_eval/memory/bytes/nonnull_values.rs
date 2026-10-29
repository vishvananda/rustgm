//! Refinement validity never converts a non-null address into dereference rights.
use super::{Constant, Failure, Machine, Reader};
use crate::sema::values::{View, WordTarget};
impl Machine<'_, '_> {
    pub(super) fn require_pattern(
        &mut self,
        constraint: crate::types::Constraint,
        value: Constant,
    ) -> Result<(), Failure> {
        let crate::types::Constraint::ScalarRange(id) = constraint else {
            return self.require_nonnull_pattern(value);
        };
        self.charge(1)?;
        let View::Scalar(value) = Reader::new(self.evaluator.facts()).view(value)? else {
            return Err("scalar pattern requires a genuine integer value".into());
        };
        let bits = match value {
            crate::sema::model::Scalar::Integer(bits) => bits,
            crate::sema::model::Scalar::Character(value) => u128::from(value),
            _ => return Err("scalar pattern requires an integer or char value".into()),
        };
        if !self.evaluator.facts().types.range_allows(id, bits) {
            return Err("value is outside its scalar pattern range".into());
        }
        Ok(())
    }
    pub(super) fn require_nonnull_pattern(&mut self, value: Constant) -> Result<(), Failure> {
        self.charge(1)?;
        if let View::Address { bits } = Reader::new(self.evaluator.facts()).view(value)? {
            return if bits == 0 {
                Err(Failure::Fault(
                    "null pointer is not a valid non-null pattern value",
                ))
            } else {
                Ok(())
            };
        }
        let (target, _) = self.evaluator.pointer_words(value)?;
        let WordTarget::Value(pointer) = target else {
            return Err("non-null pattern requires a genuine data address".into());
        };
        if matches!(
            Reader::new(self.evaluator.facts()).view(pointer)?,
            View::Function(_)
        ) {
            // A canonical function word is a non-null address, not a permission to call.
            return Ok(());
        }
        let (address, work) = self
            .evaluator
            .word_address(pointer, self.limits.steps.saturating_sub(self.work.steps))?;
        self.charge(work)?;
        let root = Reader::new(self.evaluator.facts())
            .location(address.root())?
            .ty;
        let (layout, work) = self
            .evaluator
            .storage_layout(root, self.limits.steps.saturating_sub(self.work.steps))?;
        self.charge(work)?;
        if address.offset() > layout.size {
            return Err("non-null validity of an out-of-allocation raw offset requires its explicit profile".into());
        }
        Ok(())
    }
}
