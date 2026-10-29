//! Intrinsic result data never bypasses the current checked direct-call witness.
use super::{Constant, Failure, Frame, Machine, Requirement, code::Code};
use crate::{sema::model::Scalar, types::instance::Instance};
pub(super) enum Answer {
    Ordinary,
    Pending,
    Ready(Constant),
}
pub(super) enum Selected {
    Unrelated,
    Pending,
    Enter(usize, Vec<Constant>),
}
impl Machine<'_, '_> {
    /// The intrinsic dispatch owner selects F before ordinary body lookup.
    pub(super) fn source_selector_call(
        &mut self,
        body: &Code,
        frame: &Frame,
        id: crate::body::ir::ValueId,
    ) -> Result<Selected, Failure> {
        let Some(catalog) = self.evaluator.facts().source_bindings.as_deref() else {
            return Ok(Selected::Unrelated);
        };
        let crate::body::ir::Rvalue::Call { callee, .. } = body
            .values
            .get(id.0 as usize)
            .ok_or("source callback dispatch has no actual Call")?
            .kind
        else {
            return Err("source callback dispatch requires its actual Call".into());
        };
        let ty = body
            .values
            .get(callee.0 as usize)
            .ok_or("source callback dispatch lost its typed callee")?
            .ty;
        let Some(crate::types::Kind::FunctionItem { item, .. }) =
            self.evaluator.facts().types.kind(ty)
        else {
            return Ok(Selected::Unrelated);
        };
        if catalog.intrinsic_kind(crate::resolve::model::DefId(item.0))?
            != Some(crate::support::bindings::SourceIntrinsic::ConstEvalSelect)
        {
            return Ok(Selected::Unrelated);
        }
        // Validation has a fixed number of graph/header/list lookups. Charge
        // the attempt before inspecting a possibly invalid current use.
        self.charge(128)?;
        self.evaluator.check_source_call_policy()?;
        let call = super::selector_sites::validate_source(
            self.evaluator.facts(),
            &self.bodies,
            self.bodies.inputs(),
            body,
            frame,
            super::selector_sites::Phase::Executing,
        )?
        .ok_or("source selector lost its actual registered Call")?;
        if call.value() != id {
            return Err("source selector changed its actual reached destination/callee".into());
        }
        let form = super::function_calls::Form::Selected {
            selector: call.selector(),
        };
        let index = match self.function_call(body, call.frame(), call.callback(), form)? {
            super::function_calls::Outcome::Pending => return Ok(Selected::Pending),
            super::function_calls::Outcome::Enter(index) => index,
            super::function_calls::Outcome::Constant(_) => {
                return Err(
                    "selected source callback cannot substitute an intrinsic result".into(),
                );
            }
        };
        self.spread_selector(&call, index)
    }

    fn spread_selector(
        &mut self,
        call: &super::selector_sites::Call<'_>,
        index: usize,
    ) -> Result<Selected, Failure> {
        let tuple = Self::value(call.frame(), call.tuple()?)?;
        let width = call.width(self.evaluator.facts())?;
        self.charge(width)?;
        let child = self
            .bodies
            .code(index)
            .ok_or("source selector callback code disappeared")?;
        let fixed = self
            .slots
            .saturating_add(self.memory.retained_cells())
            .saturating_add(child.locals.len())
            .saturating_add(child.values.len());
        if fixed.saturating_add(width) > self.limits.slots {
            return Err("source selector tuple exceeds constant slot budget".into());
        }
        let mut arguments = Vec::new();
        arguments
            .try_reserve_exact(width)
            .map_err(|_| "source selector tuple allocation failed")?;
        if fixed.saturating_add(arguments.capacity()) > self.limits.slots {
            return Err("source selector tuple payload exceeds constant slot budget".into());
        }
        self.work.peak_slots = self
            .work
            .peak_slots
            .max(fixed.saturating_add(arguments.capacity()));
        for ordinal in 0..width {
            arguments.push(self.evaluator.field(tuple, ordinal as u64)?);
        }
        Ok(Selected::Enter(index, arguments))
    }

    pub(super) fn source_intrinsic_call(
        &mut self,
        body: &Code,
        frame: &Frame,
        target: Instance,
    ) -> Result<Answer, Failure> {
        crate::sema::compiler_items::check_const_drop_glue(self.evaluator.facts(), target)?;
        crate::sema::compiler_items::check_const_select(self.evaluator.facts(), target)?;
        let position = frame
            .next
            .checked_sub(1)
            .ok_or("source intrinsic call has no current statement")?;
        if let Some(answer) = self.source_caller_location(body, frame, position, target)? {
            return Ok(answer);
        }
        if let Some(answer) = self.source_wrapping_offset(body, frame, position, target)? {
            return Ok(answer);
        }
        if let Some(answer) = self.source_transmute(body, frame, position, target)? {
            return Ok(answer);
        }
        if let Some(answer) = self.source_three_way(body, frame, position, target)? {
            return Ok(answer);
        }
        if let Some(answer) = self.source_discriminant(body, frame, position, target)? {
            return Ok(answer);
        }
        if let Some(answer) = self.source_offset(body, frame, position, target)? {
            return Ok(answer);
        }
        if let Some(answer) = self.source_raw_parts(body, frame, position, target)? {
            return Ok(answer);
        }
        if let Some(answer) = self.source_ptr_metadata(body, frame, position, target)? {
            return Ok(answer);
        }
        if let Some(answer) = self.source_runtime_check(body, frame, position, target)? {
            return Ok(answer);
        }
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
