//! Conservative knowledge queries retain the same checked direct-call contract.
use super::{Constant, Failure, Frame, Machine, code::Code, executed_calls::Answer};
use crate::types::instance::Instance;

impl Machine<'_, '_> {
    pub(super) fn source_knowledge_call(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = self
            .evaluator
            .checked_static_knowledge(target, body.source)?
        else {
            return self.source_guaranteed_compare(body, frame, position, target);
        };
        self.charge(64)?;
        let arguments = self.checked_knowledge_call(
            body,
            frame,
            position,
            target,
            (
                selected.definition(),
                crate::types::TypeId::primitive(crate::types::Primitive::Bool),
            ),
            |actual, public| {
                public == selected.signature()
                    && selected.opened_matches(self.evaluator.facts(), actual)
            },
        )?;
        let Some(&[input]) = body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
        else {
            return Err("source value knowledge requires its one evaluated argument".into());
        };
        if body
            .values
            .get(input.0 as usize)
            .is_none_or(|value| value.ty != selected.input())
        {
            return Err("source value knowledge changed its actual argument type".into());
        }
        let _value = Self::value(frame, input)?;
        self.work.calls += 1;
        // Neither true nor pointee knowledge is promised. This valid conservative
        // choice never reads through a pointer or executes the attributed body.
        Ok(Some(Answer::Ready(Constant::scalar(
            crate::sema::model::Scalar::Boolean(false),
        ))))
    }
    fn checked_knowledge_call(
        &self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
        contract: (crate::resolve::model::DefId, crate::types::TypeId),
        matches_signature: impl FnOnce(crate::types::TypeId, crate::types::TypeId) -> bool,
    ) -> Result<crate::body::ir::ValueList, Failure> {
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|current| !std::ptr::eq(current, body))
        {
            return Err("source knowledge query belongs to another current Code".into());
        }
        let arguments = super::reached_layout::checked_call_arguments_with_signature(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            contract,
            matches_signature,
        )?;
        self.evaluator
            .facts()
            .source_bindings
            .as_deref()
            .ok_or("source knowledge query catalog disappeared")?
            .check_const_call(contract.0)?;
        Ok(arguments)
    }

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
        let arguments = self.checked_knowledge_call(
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
