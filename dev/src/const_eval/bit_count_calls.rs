//! A reached bit count consumes one exact evaluated integer and the actual source call.
use super::{Constant, Failure, Frame, Machine, code::Code, executed_calls::Answer};
use crate::{
    sema::model::Scalar,
    types::{Primitive, TypeId, instance::Instance},
};
impl Machine<'_, '_> {
    pub(super) fn source_bit_count(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = self.evaluator.checked_bit_count(target, body.source)? else {
            return Ok(None);
        };
        self.charge(64)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|current| !std::ptr::eq(current, body))
        {
            return Err("source bit count belongs to another reached caller".into());
        }
        let arguments = super::reached_layout::checked_call_arguments_with_signature(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            (selected.definition(), TypeId::primitive(Primitive::U32)),
            |actual, public| {
                public == selected.signature()
                    && selected.opened_matches(self.evaluator.facts(), actual)
            },
        )?;
        self.evaluator
            .facts()
            .source_bindings
            .as_deref()
            .ok_or("source bit count catalog disappeared")?
            .check_const_call(selected.definition())?;
        let Some(&[input]) = body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
        else {
            return Err("source bit count requires its one evaluated argument".into());
        };
        if body
            .values
            .get(input.0 as usize)
            .is_none_or(|value| value.ty != selected.input())
        {
            return Err("source bit count changed its actual argument type".into());
        }
        let bits = super::values::integer(Self::value(frame, input)?)?;
        let count = selected
            .count(bits)
            .ok_or(Failure::Fault("a nonzero bit count was executed on zero"))?;
        self.work.calls += 1;
        Ok(Some(Answer::Ready(Constant::scalar(Scalar::Integer(
            u128::from(count),
        )))))
    }
}
