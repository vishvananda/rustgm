//! A reached pointer query consumes the same checked direct-call contract as other intrinsics.
use super::{Failure, Frame, Machine, code::Code, executed_calls::Answer};
use crate::{sema::model::Declarations, types::instance::Instance};

impl Machine<'_, '_> {
    pub(super) fn source_guaranteed_compare(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = self
            .evaluator
            .checked_guaranteed_compare(target, body.source)?
        else {
            return Ok(None);
        };
        self.charge(64)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|current| !std::ptr::eq(current, body))
        {
            return Err("source pointer comparison belongs to another current Code".into());
        }
        let arguments = super::reached_layout::checked_call_arguments_with_signature(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            (
                selected.definition(),
                crate::types::TypeId::primitive(crate::types::Primitive::U8),
            ),
            |actual, public| {
                public == selected.signature()
                    && selected.opened_matches(self.evaluator.facts(), actual)
            },
        )?;
        let facts: &Declarations = self.evaluator.facts();
        facts
            .source_bindings
            .as_deref()
            .ok_or("source pointer comparison catalog disappeared")?
            .check_const_call(selected.definition())?;
        let Some(&[left, right]) = body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
        else {
            return Err("source pointer comparison requires its two actual values".into());
        };
        if [left, right].into_iter().any(|id| {
            body.values
                .get(id.0 as usize)
                .is_none_or(|value| value.ty != selected.pointer())
        }) {
            return Err("source pointer comparison changed its actual input type".into());
        }
        let left = Self::value(frame, left)?;
        let right = Self::value(frame, right)?;
        let mut work = 0;
        let answer = self.evaluator.guaranteed_pointer_compare(
            left,
            right,
            &mut work,
            self.limits.steps.saturating_sub(self.work.steps),
        );
        self.charge(work)?;
        self.work.calls += 1;
        Ok(Some(Answer::Ready(answer?)))
    }
}
