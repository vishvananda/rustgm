//! Reached scalar comparisons return genuine source Ordering variant constants.
use super::{Constant, Failure, Frame, Machine, code::Code, executed_calls::Answer};
use crate::{lex::token::Punctuation, sema::values::Overflow, types::instance::Instance};
impl Machine<'_, '_> {
    pub(super) fn source_three_way(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = self
            .evaluator
            .checked_source_three_way(target, body.source)?
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
            return Err("three_way_compare belongs to another reached caller".into());
        }
        let arguments = super::reached_layout::checked_call_arguments(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            (selected.definition, selected.output),
        )?;
        let [left, right] = body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("three_way_compare argument range changed")?
        else {
            return Err("three_way_compare requires two evaluated scalar arguments".into());
        };
        if [left, right].into_iter().any(|id| {
            body.values
                .get(id.0 as usize)
                .is_none_or(|v| v.ty != selected.input)
        }) {
            return Err("three_way_compare input differs from its actual scalar type".into());
        }
        if let Some(variant) = selected.pending_tag(self.evaluator.facts())? {
            self.waiting = Some(super::Requirement::DiscriminantValue(variant));
            return Ok(Some(Answer::Pending));
        }
        // Values are snapshots of already evaluated arguments, including identical operands.
        let (left, right) = (Self::value(frame, *left)?, Self::value(frame, *right)?);
        let less = super::values::boolean(self.evaluator.binary(
            Punctuation::Less,
            selected.input,
            left,
            right,
            Overflow::Checked,
        )?)?;
        let greater = super::values::boolean(self.evaluator.binary(
            Punctuation::Greater,
            selected.input,
            left,
            right,
            Overflow::Checked,
        )?)?;
        let ordinal = if less {
            0
        } else if greater {
            2
        } else {
            1
        };
        let result: Constant = self
            .evaluator
            .fields(&[], Some(selected.variants[ordinal]))?;
        self.work.calls += 1;
        Ok(Some(Answer::Ready(result)))
    }
}
