//! Reached subtraction consumes exact evaluated inputs and the actual source call.
use super::{Constant, Failure, Frame, Machine, code::Code, executed_calls::Answer};
use crate::{
    lex::token::Punctuation,
    sema::{model::Scalar, values::Overflow},
    support::bindings::SourceIntrinsic,
    types::instance::Instance,
};
impl Machine<'_, '_> {
    pub(super) fn source_subtraction(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = self
            .evaluator
            .checked_source_subtraction(target, body.source)?
        else {
            return Ok(None);
        };
        self.charge(64)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|actual| !std::ptr::eq(actual, body))
        {
            return Err("source subtraction belongs to another reached caller".into());
        }
        let arguments = super::reached_layout::checked_call_arguments_with_signature(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            (selected.definition, selected.output),
            |actual, public| {
                public == selected.signature
                    && selected.opened_signature_matches(&self.evaluator.facts().types, actual)
            },
        )?;
        self.evaluator
            .facts()
            .source_bindings
            .as_deref()
            .ok_or("source subtraction catalog disappeared")?
            .check_const_call(selected.definition)?;
        let [left, right] = body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source subtraction arguments changed")?
        else {
            return Err("source subtraction requires two evaluated arguments".into());
        };
        if [(*left, selected.input), (*right, selected.rhs)]
            .into_iter()
            .any(|(id, ty)| body.values.get(id.0 as usize).is_none_or(|v| v.ty != ty))
        {
            return Err("source subtraction changed its actual argument types".into());
        }
        let (left, right) = (Self::value(frame, *left)?, Self::value(frame, *right)?);
        let result = self.evaluate_source_arithmetic(&selected, left, right)?;
        self.work.calls += 1;
        Ok(Some(Answer::Ready(result)))
    }
    fn evaluate_source_arithmetic(
        &mut self,
        selected: &crate::sema::compiler_items::SubtractionSelection,
        left: Constant,
        right: Constant,
    ) -> Result<Constant, Failure> {
        if let Some(operator) = crate::sema::compiler_items::unchecked_shift(selected.kind) {
            return self
                .evaluator
                .binary(operator, selected.input, left, right, Overflow::Checked);
        }
        if let Some(operator) = crate::sema::compiler_items::wrapping_operator(selected.kind) {
            return self.evaluator.binary(
                operator,
                selected.input,
                left,
                right,
                Overflow::Wrapping,
            );
        }
        let operator = match selected.kind {
            SourceIntrinsic::UncheckedSub => Punctuation::Minus,
            kind => crate::sema::compiler_items::overflowing_operator(kind)
                .ok_or("source arithmetic lost its selected overflow operation")?,
        };
        let checked =
            self.evaluator
                .binary(operator, selected.input, left, right, Overflow::Checked);
        if selected.kind == SourceIntrinsic::UncheckedSub {
            checked
        } else {
            let overflow = match checked {
                Ok(_) => false,
                Err(Failure::Fault(_)) => true,
                Err(error) => return Err(error),
            };
            let wrapped =
                self.evaluator
                    .binary(operator, selected.input, left, right, Overflow::Wrapping)?;
            self.reserve_cells(3)?;
            self.evaluator.fields(
                &[wrapped, Constant::scalar(Scalar::Boolean(overflow))],
                None,
            )
        }
    }
}
