//! Reached closure calls consume their checked caller record and actual proved environment body.
use super::{Constant, Failure, Frame, Machine, Requirement, code::Code};
use crate::{
    body::ir::{Action, Rvalue, ValueId},
    types::{Kind, TypeId},
};

pub(super) enum Outcome {
    Ordinary,
    Pending,
    Enter(usize, Vec<Constant>),
}
impl Machine<'_, '_> {
    pub(super) fn invoke(
        &mut self,
        body: &Code,
        frame: &Frame,
        id: ValueId,
    ) -> Result<Option<(usize, Vec<Constant>, ValueId)>, Failure> {
        match self.captured_invoke(body, frame, id)? {
            Outcome::Enter(index, arguments) => return Ok(Some((index, arguments, id))),
            Outcome::Pending => return Ok(None),
            Outcome::Ordinary => {}
        }
        let Rvalue::Invoke {
            kind,
            signature,
            arguments,
        } = body.values[id.0 as usize].kind
        else {
            return Err("const Invoke changed its actual operation".into());
        };
        let Some((target, arguments)) =
            self.invoked_item(body, frame, id, kind, signature, arguments)?
        else {
            return Ok(None);
        };
        let arguments = self.arguments(frame, arguments)?;
        match self.function_call(body, frame, target, super::function_calls::Form::Invoke)? {
            super::function_calls::Outcome::Enter(index) => Ok(Some((index, arguments, id))),
            super::function_calls::Outcome::Pending => Ok(None),
            super::function_calls::Outcome::Constant(_) => {
                Err("const Invoke cannot substitute an intrinsic result".into())
            }
        }
    }
    fn captured_invoke(
        &mut self,
        code: &Code,
        frame: &Frame,
        id: ValueId,
    ) -> Result<Outcome, Failure> {
        self.charge(12)?;
        self.bodies.validate_entry(frame.body)?;
        let position = frame
            .next
            .checked_sub(1)
            .ok_or("closure call has no reached statement")?;
        let value = code
            .values
            .get(id.0 as usize)
            .ok_or("closure call operation is absent")?;
        let Rvalue::Invoke {
            kind, arguments, ..
        } = value.kind
        else {
            return Err("closure call no longer selects Invoke".into());
        };
        let mut work = 0;
        let call = code.closure_call(&self.evaluator.facts().types, position, id, &mut work);
        self.charge(work)?;
        let Some(call) = call? else {
            if super::code::closure_subject(
                &self.evaluator.facts().types,
                &code.values,
                &code.arguments,
                id,
            )?
            .is_some()
            {
                return Err("closure call requires its actual proved caller record".into());
            }
            return Ok(Outcome::Ordinary);
        };
        let block = code
            .blocks
            .get(frame.block.0 as usize)
            .ok_or("closure call block is absent")?;
        if position < block.start
            || position >= block.end
            || !matches!(code.statements.get(position as usize).map(|s| s.action), Some(Action::Evaluate(actual)) if actual == id)
        {
            return Err("closure call differs from its reached checked statement".into());
        }
        self.evaluator.check_source_call_policy()?;
        crate::sema::compiler_items::callable_role(self.evaluator.facts(), kind)?;
        let target = call.instance();
        if call.entry().is_some_and(|entry| entry.adapter().is_some()) {
            return Err(
                "const closure receiver adapter requires its actual proved adapter body".into(),
            );
        }
        let Some(&index) = self.bodies.functions.get(&target) else {
            self.waiting = Some(Requirement::FunctionBody(target));
            return Ok(Outcome::Pending);
        };
        self.bodies.validate_entry(index)?;
        let callee = self
            .bodies
            .code(index)
            .ok_or("const closure body disappeared")?;
        callee.validate_closure_family(self.evaluator.facts(), target)?;
        let Some(Kind::Closure { environment, .. }) =
            self.evaluator.facts().types.kind(target.ty())
        else {
            return Err("const closure call lost its actual Self".into());
        };
        let mut arguments = arguments;
        if environment == TypeId::UNIT {
            if call.entry().is_some() {
                return Err(
                    "empty closure unexpectedly selected a captured environment entry".into(),
                );
            }
            arguments.start = arguments
                .start
                .checked_add(1)
                .ok_or("closure argument range overflow")?;
        } else if call.entry().is_none()
            || code.arguments.get(arguments.start as usize).copied() != Some(call.receiver())
        {
            return Err("captured closure lost its selected environment receiver".into());
        }
        Ok(Outcome::Enter(index, self.arguments(frame, arguments)?))
    }
}
