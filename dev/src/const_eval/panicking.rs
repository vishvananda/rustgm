//! A reached genuine source panic is a CTFE fault, without a fabricated helper body.
use super::{Failure, Frame, Machine, code::Code};
use crate::types::{TypeId, instance::Instance};
impl Machine<'_, '_> {
    pub(super) fn source_panic_call(
        &mut self,
        body: &Code,
        frame: &Frame,
        target: Instance,
    ) -> Result<(), Failure> {
        let Some(selected) = self
            .evaluator
            .checked_source_panic_display(target, body.source)?
        else {
            return Ok(());
        };
        self.charge(32)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|actual| !std::ptr::eq(actual, body))
        {
            return Err("source const panic belongs to another actual caller Code".into());
        }
        let position = frame
            .next
            .checked_sub(1)
            .ok_or("source const panic has no current Call")?;
        let arguments = super::reached_layout::checked_call_arguments_with_signature(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            (selected.definition, TypeId::NEVER),
            |actual, public| {
                public == selected.signature
                    && selected.matches_call(&self.evaluator.facts().types, actual)
            },
        )?;
        let [input] = body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source const panic lost its checked input range")?
        else {
            return Err("source const panic requires one evaluated input".into());
        };
        if body
            .values
            .get(input.0 as usize)
            .is_none_or(|value| !selected.matches_input(&self.evaluator.facts().types, value.ty))
        {
            return Err("source const panic changed its actual shared input".into());
        }
        // A reached Call has evaluated its real input even though it never returns.
        let _message = Self::value(frame, *input)?;
        Err(Failure::Fault(
            "constant execution reached source panic_display",
        ))
    }
}
