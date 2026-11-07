//! A reached `assume` consumes one exact evaluated condition and the actual source call.
use super::{Failure, Frame, Machine, code::Code, executed_calls::Answer};
use crate::types::{TypeId, instance::Instance};
impl Machine<'_, '_> {
    /// A true condition returns unit. A false one is undefined behavior, so
    /// constant evaluation fails, as rustc's "`assume` called with `false`".
    /// The declaration's fallback body is never entered.
    pub(super) fn source_assume(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = self.evaluator.checked_assumption(target, body.source)? else {
            return Ok(None);
        };
        self.charge(32)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|current| !std::ptr::eq(current, body))
        {
            return Err("source assume belongs to another reached caller".into());
        }
        let arguments = super::reached_layout::checked_call_arguments_with_signature(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            (selected.definition(), TypeId::UNIT),
            |actual, public| public == selected.signature() && actual == public,
        )?;
        self.evaluator
            .facts()
            .source_bindings
            .as_deref()
            .ok_or("source assume catalog disappeared")?
            .check_const_call(selected.definition())?;
        let Some(&[condition]) = body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
        else {
            return Err("source assume requires its one evaluated condition".into());
        };
        if body
            .values
            .get(condition.0 as usize)
            .is_none_or(|value| value.ty != TypeId::primitive(crate::types::Primitive::Bool))
        {
            return Err("source assume changed its actual condition type".into());
        }
        if !super::values::boolean(Self::value(frame, condition)?)? {
            return Err(Failure::Fault(
                "`assume` was executed with a false condition",
            ));
        }
        self.work.calls += 1;
        Ok(Some(Answer::Ready(self.evaluator.fields(&[], None)?)))
    }
}
