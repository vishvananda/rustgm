//! Const callers reuse actual frames and their sealed call-site data.
use super::{Frame, Machine, code::Code, executed_calls::Answer};
use crate::{
    body::ir::{Action, Owner, ValueId},
    resolve::model::DefId,
    support::bindings::SourceIntrinsic,
    types::{Kind, instance::Instance},
};

impl Machine<'_, '_> {
    pub(super) fn source_caller_location(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, super::Failure> {
        let Some(Kind::FunctionItem { item, .. }) = self.evaluator.facts().types.kind(target.ty())
        else {
            return Ok(None);
        };
        if self
            .evaluator
            .facts()
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.intrinsic_kind(DefId(item.0)))
            .transpose()?
            .flatten()
            != Some(SourceIntrinsic::CallerLocation)
        {
            return Ok(None);
        }
        self.charge(32)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|actual| !std::ptr::eq(actual, body))
        {
            return Err("caller intrinsic belongs to another current Code".into());
        }
        let selected = self
            .evaluator
            .checked_source_caller_location(target, body.source)?
            .ok_or("caller intrinsic lost its current selected declaration")?;
        super::reached_layout::checked_call(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            (selected.definition, selected.result),
        )?;
        let Some(Action::Evaluate(destination)) = body
            .statements
            .get(position as usize)
            .map(|statement| statement.action)
        else {
            return Err("caller intrinsic lost its reached destination".into());
        };
        let location = self.inherited_caller_site(body, frame, destination)?;
        let value = crate::sema::values::Reader::new(self.evaluator.facts())
            .caller_location_value(location)?;
        self.work.calls += 1;
        Ok(Some(Answer::Ready(value)))
    }
    fn inherited_caller_site(
        &mut self,
        body: &Code,
        frame: &Frame,
        destination: ValueId,
    ) -> Result<crate::sema::values::LocationId, super::Failure> {
        let mut next = Some(destination);
        for index in (0..=self.frames.len()).rev() {
            self.charge(4)?;
            let (code, actual) = if index == self.frames.len() {
                (body, frame)
            } else {
                let actual = &self.frames[index];
                self.bodies.validate_entry(actual.body)?;
                (
                    self.bodies
                        .code(actual.body)
                        .ok_or("caller frame code disappeared")?,
                    actual,
                )
            };
            let tracked = if let Owner::Function { definition, .. } = code.owner {
                self.evaluator.checked_caller_tracking(definition)?
            } else {
                false
            };
            let id = next.ok_or("tracked const entry lacks a genuine caller destination")?;
            let site = code
                .caller_site(id)?
                .ok_or("const call lacks its sealed actual caller location")?;
            let crate::body::ir::Rvalue::Call { callee, .. } = code
                .values
                .get(id.0 as usize)
                .ok_or("caller frame lost its actual call")?
                .kind
            else {
                return Err("caller frame destination is not a Call".into());
            };
            if code
                .values
                .get(callee.0 as usize)
                .is_none_or(|value| value.ty != site.callable)
            {
                return Err("caller frame changed its actual call-site callee".into());
            }
            crate::sema::values::Reader::new(self.evaluator.facts())
                .caller_location_value(site.location)?;
            if !tracked {
                return Ok(site.location);
            }
            next = actual.return_to;
        }
        Err("tracked const entry lacks a genuine caller frame".into())
    }
}
