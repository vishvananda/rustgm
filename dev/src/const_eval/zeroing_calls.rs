//! Reached zeroing intrinsics check the actual assertion or update the proved byte span.
use super::{Failure, Frame, Machine, code::Code, executed_calls::Answer};
use crate::{
    sema::compiler_items::zeroing_contract::Operation,
    types::{Kind, TypeId, instance::Instance},
};
impl Machine<'_, '_> {
    pub(super) fn source_zeroing(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = self.evaluator.checked_zeroing(target, body.source)? else {
            return Ok(None);
        };
        self.charge(64)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|current| !std::ptr::eq(current, body))
        {
            return Err("source zeroing belongs to another reached caller".into());
        }
        let facts = self.evaluator.facts();
        let arguments = super::reached_layout::checked_call_arguments_with_signature(
            facts,
            body,
            frame,
            position,
            target,
            (selected.definition(), TypeId::UNIT),
            |actual, public| {
                public == selected.signature() && selected.opened_matches(facts, actual)
            },
        )?;
        facts
            .source_bindings
            .as_deref()
            .ok_or("source zeroing catalog disappeared")?
            .check_const_call(selected.definition())?;
        let inputs = body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source zeroing lost its evaluated arguments")?;
        if inputs
            .iter()
            .zip(selected.inputs())
            .any(|(id, ty)| body.values.get(id.0 as usize).is_none_or(|v| v.ty != ty))
        {
            return Err("source zeroing changed its actual argument types".into());
        }
        let answer = match (selected.operation(), inputs) {
            (Operation::Assertion(subject), []) => {
                if !facts.aggregates.zero_valid(&facts.types, subject, |_| {
                    Ok(crate::types::aggregate::ZeroFields::Unproven)
                })? {
                    return Err("source intrinsic constant execution is not implemented".into());
                }
                self.evaluator.fields(&[], None)?
            }
            (Operation::Fill(destination), &[address, byte, count]) => {
                let Some(Kind::Pointer { pointee, .. }) = facts.types.kind(destination) else {
                    return Err("source write_bytes lost its destination type".into());
                };
                let values = [
                    Self::value(frame, address)?,
                    Self::value(frame, byte)?,
                    Self::value(frame, count)?,
                ];
                self.counted_fill(pointee, &values)?
                    .ok_or("source write_bytes has no unit result")?
            }
            _ => return Err("source zeroing changed its actual arguments".into()),
        };
        self.work.calls += 1;
        Ok(Some(Answer::Ready(answer)))
    }
}
