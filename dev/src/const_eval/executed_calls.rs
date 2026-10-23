//! Intrinsic result data never bypasses the current checked direct-call witness.
use super::{Constant, Failure, Frame, Machine, Requirement, code::Code};
use crate::{sema::model::Scalar, types::instance::Instance};
pub(super) enum Answer {
    Ordinary,
    Pending,
    Ready(Constant),
}
impl Machine<'_, '_> {
    pub(super) fn source_intrinsic_call(
        &mut self,
        body: &Code,
        frame: &Frame,
        target: Instance,
    ) -> Result<Answer, Failure> {
        let position = frame
            .next
            .checked_sub(1)
            .ok_or("source intrinsic call has no current statement")?;
        let value = if let Some(call) =
            super::reached_layout::validate(self.evaluator.facts(), body, frame, position, target)?
        {
            self.bodies
                .layout_result(call.instance())
                .map(|bits| Scalar::Integer(u128::from(bits)))
        } else if let Some(call) =
            super::reached_drop::validate(self.evaluator.facts(), body, frame, position, target)?
        {
            self.bodies
                .needs_drop_result(call.instance())
                .map(Scalar::Boolean)
        } else {
            return Ok(Answer::Ordinary);
        };
        self.bodies.validate_entry(frame.body)?;
        let Some(value) = value else {
            self.waiting = Some(Requirement::FunctionBody(target));
            return Ok(Answer::Pending);
        };
        self.work.calls += 1;
        Ok(Answer::Ready(Constant::scalar(value)))
    }
}
