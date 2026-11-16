//! A reached type name is its closed subject's name, as text of the actual source call.
use super::{Failure, Frame, Machine, code::Code, executed_calls::Answer};
use crate::types::instance::Instance;
impl Machine<'_, '_> {
    pub(super) fn source_type_name(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = self.evaluator.checked_type_name(target, body.source)? else {
            return Ok(None);
        };
        self.charge(64)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|current| !std::ptr::eq(current, body))
        {
            return Err("source type name belongs to another reached caller".into());
        }
        let arguments = super::reached_layout::checked_call_arguments_with_signature(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            (selected.definition(), selected.output()),
            |actual, public| {
                public == selected.signature()
                    && selected.opened_matches(self.evaluator.facts(), actual)
            },
        )?;
        self.evaluator
            .facts()
            .source_bindings
            .as_deref()
            .ok_or("source type name catalog disappeared")?
            .check_const_call(selected.definition())?;
        if arguments.start != arguments.end {
            return Err("source type name changed its actual arguments".into());
        }
        let (value, spelled) = self.evaluator.type_name_text(&selected)?;
        self.charge(spelled)?;
        self.work.calls += 1;
        Ok(Some(Answer::Ready(value)))
    }
}
