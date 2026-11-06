//! Immutable execution uses the actual captured source operation and store witness.
use super::{Code, PreparedOperand};
use crate::{
    body::ir::{Place, Rvalue, ValueId},
    sema::model::Declarations,
};
impl Code {
    fn reborrow_records(&self) -> Option<&crate::body::reborrow_inputs::Inputs> {
        let mut index = self.operands.len();
        if matches!(
            index.checked_sub(1).and_then(|i| self.operands.get(i)),
            Some(PreparedOperand::CallerSites(_))
        ) {
            index -= 1;
        }
        if matches!(
            index.checked_sub(1).and_then(|i| self.operands.get(i)),
            Some(PreparedOperand::Cleanup(_))
        ) {
            index -= 1;
        }
        match index.checked_sub(1).and_then(|i| self.operands.get(i))? {
            PreparedOperand::Reborrows(inputs) => Some(inputs),
            _ => None,
        }
    }
    pub(super) fn require_reborrow_context(
        &self,
        types: &crate::types::Store,
    ) -> Result<(), &'static str> {
        if let Some(inputs) = self.reborrow_records() {
            inputs.require_context(self.owner.key(self.source), self.source, self.instance)?;
            inputs.require_store(types)?;
        }
        Ok(())
    }
    pub(super) fn reborrow_bytes(&self) -> usize {
        self.reborrow_records().map_or(0, |p| {
            p.allocation_bytes().saturating_add(2 * size_of::<usize>())
        })
    }
    pub(in crate::const_eval) fn value_reborrow(
        &self,
        facts: &Declarations,
        operation: ValueId,
    ) -> Result<crate::body::reborrow_inputs::ValueReborrowUse, &'static str> {
        let row = self
            .reborrow_records()
            .ok_or("constant value reborrow lacks its actual input snapshot")?
            .selected(
                &facts.types,
                self.owner.key(self.source),
                self.source,
                self.instance,
                operation,
            )?;
        let value = self
            .values
            .get(operation.0 as usize)
            .ok_or("constant value reborrow operation disappeared")?;
        let source = match row.place {
            Place::Local(id) => self.locals.get(id.0 as usize).map(|p| p.ty),
            Place::Temporary(id) => self.values.get(id.0 as usize).map(|p| p.ty),
            Place::Projection(id) => self.projections.get(id.0 as usize).map(|p| p.ty),
            Place::Static(id) => facts.definitions.get(id.0 as usize).copied().flatten(),
        };
        if source != Some(row.source)
            || value.ty != row.target
            || !matches!(value.kind,Rvalue::Reborrow{place,mutable}if place==row.place&&mutable==row.mutable)
        {
            return Err("constant value reborrow changed its actual place/operation/endpoints");
        }
        Ok(row)
    }
}
