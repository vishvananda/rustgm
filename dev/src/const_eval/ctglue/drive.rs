//! Real timeline events suspend on selected Drop bodies before local storage ends.
use super::{Bundle, ctsites};
use crate::{
    body::drops::schedule::Effect,
    body::{
        ir::{BlockId, Place},
        storage::Location,
    },
    const_eval::{Failure, Frame, Machine, Requirement, code::Code},
    sema::values::Constant,
};
use std::{num::NonZeroU32, ops::Range};

#[derive(Clone, Copy)]
pub(in crate::const_eval) enum Step {
    Ready,
    Enter { code: usize, event: u32 },
    Pending,
}
fn set_cursor(frame: &mut Frame, event: usize) -> Result<(), Failure> {
    frame
        .cleanup
        .as_mut()
        .ok_or("cleanup cursor has no actual frame state")?
        .event = Some(
        u32::try_from(event)
            .ok()
            .and_then(|n| n.checked_add(1))
            .and_then(NonZeroU32::new)
            .ok_or("cleanup cursor identity overflow")?,
    );
    Ok(())
}
impl Machine<'_, '_> {
    pub(in crate::const_eval) fn cleanup_parent(
        &self,
        return_to: crate::const_eval::ReturnTo,
    ) -> Result<Option<NonZeroU32>, Failure> {
        if !matches!(return_to, crate::const_eval::ReturnTo::Cleanup(_)) {
            return Ok(None);
        }
        let parent = self
            .frames
            .last()
            .ok_or("cleanup child has no actual parent frame")?;
        let id = parent
            .instance
            .checked_add(1)
            .and_then(|id| u32::try_from(id).ok())
            .and_then(NonZeroU32::new)
            .ok_or("cleanup parent identity budget exceeded")?;
        Ok(Some(id))
    }
    pub(in crate::const_eval) fn suspend_cleanup(
        &mut self,
        mut frame: Frame,
        step: Step,
    ) -> Result<Option<crate::sema::values::execution::Progress>, Failure> {
        match step {
            Step::Ready => Err("ready cleanup cannot suspend its caller".into()),
            Step::Enter { code, event } => {
                let receiver = self.cleanup_receiver(&mut frame, code, event)?;
                self.frames.push(frame);
                self.push(
                    code,
                    &[receiver],
                    crate::const_eval::ReturnTo::Cleanup(event),
                )?;
                Ok(None)
            }
            Step::Pending => {
                self.frames.push(frame);
                self.waiting
                    .take()
                    .map(crate::sema::values::execution::Progress::Prepare)
                    .map(Some)
                    .ok_or_else(|| "cleanup preparation lost its actual requirement".into())
            }
        }
    }
    pub(in crate::const_eval) fn cleanup_block(
        &mut self,
        frame: &mut Frame,
    ) -> Result<Step, Failure> {
        let code = self.bodies[frame.body].clone();
        let Some(bundle) = code.cleanup() else {
            return Ok(Step::Ready);
        };
        let range = bundle
            .view
            .blocks
            .get(frame.block.0 as usize)
            .cloned()
            .ok_or("cleanup timeline lacks its actual block")?;
        let state = frame
            .cleanup
            .as_ref()
            .ok_or("cleanup block has no actual frame state")?;
        state.require_frame(&code, frame.instance)?;
        if state.event.is_none() {
            set_cursor(frame, range.start)?;
        }
        let current = ctsites::cursor(frame)?;
        if current < range.start || current > range.end {
            let event = bundle
                .view
                .events
                .get(current)
                .ok_or("cleanup cursor escaped its actual block")?;
            ctsites::location(&code, frame, event.location)?;
            if !matches!(event.location, Location::Edge { .. }) {
                return Err("cleanup cursor skipped its actual block events".into());
            }
            return Ok(Step::Ready);
        }
        self.cleanup_range(frame, &code, bundle, range, None)
    }
    pub(in crate::const_eval) fn cleanup_edge(
        &mut self,
        frame: &mut Frame,
        target: BlockId,
    ) -> Result<Step, Failure> {
        let code = self.bodies[frame.body].clone();
        let Some(bundle) = code.cleanup() else {
            return Ok(Step::Ready);
        };
        let block = bundle
            .view
            .blocks
            .get(frame.block.0 as usize)
            .ok_or("cleanup edge lacks its actual source block")?;
        let range = bundle
            .view
            .edges
            .get(&(frame.block, target))
            .cloned()
            .unwrap_or(0..0);
        frame
            .cleanup
            .as_ref()
            .ok_or("cleanup edge has no actual frame state")?
            .require_frame(&code, frame.instance)?;
        let current = ctsites::cursor(frame)?;
        if current == block.end {
            set_cursor(frame, range.start)?;
        } else if current < range.start || current > range.end {
            return Err("cleanup edge changed its actual continuation".into());
        }
        self.cleanup_range(frame, &code, bundle, range, Some(target))
    }
    fn cleanup_range(
        &mut self,
        frame: &mut Frame,
        code: &Code,
        bundle: &Bundle,
        range: Range<usize>,
        edge: Option<BlockId>,
    ) -> Result<Step, Failure> {
        let mut current = ctsites::cursor(frame)?;
        while current < range.end {
            self.charge(1)?;
            let event = bundle
                .view
                .events
                .get(current)
                .ok_or("missing actual cleanup event")?;
            match event.location {
                Location::At { block, before }
                    if edge.is_none() && block == frame.block && before <= frame.next => {}
                Location::Edge { block, target }
                    if edge == Some(target) && block == frame.block => {}
                _ => return Err("cleanup timeline changed its exact CFG location".into()),
            }
            match event.effect {
                Effect::Statement => {
                    ctsites::location(code, frame, event.location)?;
                    return Ok(Step::Ready);
                }
                Effect::Cleanup(index) => {
                    if bundle.view.call(index)?.is_some() {
                        let step = self.enter_cleanup(frame, code, bundle)?;
                        if !matches!(step, Step::Ready) {
                            return Ok(step);
                        }
                    }
                }
                Effect::Point | Effect::Deinitialize { .. } | Effect::StorageEnd(_) => {}
                Effect::Birth(_) => {
                    return Err(
                        "constant materialized-temporary cleanup execution is not implemented"
                            .into(),
                    );
                }
                Effect::Flags { range, .. } => {
                    let width = range
                        .end
                        .checked_sub(range.start)
                        .ok_or("invalid cleanup flag write extent")?;
                    self.charge(width as usize + 32)?;
                    frame
                        .cleanup
                        .as_mut()
                        .ok_or("flag write has no actual frame state")?
                        .write(code, frame.instance, current)?;
                }
            }
            current += 1;
            set_cursor(frame, current)?;
        }
        if edge.is_none()
            && frame.next
                != code
                    .blocks
                    .get(frame.block.0 as usize)
                    .ok_or("invalid cleanup block")?
                    .end
        {
            return Err("cleanup timeline omitted its actual statement event".into());
        }
        Ok(Step::Ready)
    }
    fn enter_cleanup(
        &mut self,
        frame: &mut Frame,
        code: &Code,
        bundle: &Bundle,
    ) -> Result<Step, Failure> {
        self.evaluator.check_source_call_policy()?;
        let mut work = crate::body::defining_cleanups::Work::new(self.limits.steps);
        let found = ctsites::current(self.evaluator.facts(), code, frame, bundle, &mut work);
        self.charge(work.steps)?;
        let (event, method, _) = found?;
        if !frame.cleanup_allowed(code, event)? {
            return Ok(Step::Ready);
        }
        let Some(&target) = self.bodies.functions.get(&method.instance) else {
            self.waiting = Some(Requirement::CleanupBody(method.instance));
            return Ok(Step::Pending);
        };
        self.validate_invoked_entry(target, method.instance)?;
        let selected = self.bodies.code(target).ok_or("cleanup body disappeared")?;
        if selected.result != crate::types::TypeId::UNIT {
            return Err("cleanup body changed its actual Unit result".into());
        }
        Ok(Step::Enter {
            code: target,
            event: u32::try_from(event).map_err(|_| "cleanup event overflow")?,
        })
    }
    fn cleanup_receiver(
        &mut self,
        frame: &mut Frame,
        target: usize,
        expected: u32,
    ) -> Result<Constant, Failure> {
        let code = self.bodies[frame.body].clone();
        let bundle = code
            .cleanup()
            .ok_or("cleanup entry lost its actual footer")?;
        let mut work = crate::body::defining_cleanups::Work::new(self.limits.steps);
        let found = ctsites::current(self.evaluator.facts(), &code, frame, bundle, &mut work);
        self.charge(work.steps)?;
        let (event, method, place) = found?;
        if !frame.cleanup_allowed(&code, event)? {
            return Err("cleanup entry changed its actual initialized condition".into());
        }
        if u32::try_from(event).map_err(|_| "cleanup event overflow")? != expected
            || self.bodies.functions.get(&method.instance) != Some(&target)
        {
            return Err("cleanup entry changed its actual event or selected body".into());
        }
        self.validate_invoked_entry(target, method.instance)?;
        let receiver = receiver(self.evaluator.facts(), method.instance)?;
        let address = self
            .address_place(frame, place)?
            .ok_or("cleanup receiver lost its live storage")?;
        if address.ty != method.target
            || address.raw.is_some()
            || address.union.is_some()
            || !matches!(place, Place::Local(_))
        {
            return Err("cleanup receiver differs from its proved owned local".into());
        }
        self.path(address.address)?;
        self.evaluator
            .adjust_reference(address.address, address.ty, receiver)
    }
    pub(in crate::const_eval) fn finish_cleanup_statement(
        &mut self,
        frame: &mut Frame,
    ) -> Result<(), Failure> {
        let code = self.bodies[frame.body].clone();
        let Some(bundle) = code.cleanup() else {
            return Ok(());
        };
        let current = ctsites::cursor(frame)?;
        let event = bundle
            .view
            .events
            .get(current)
            .ok_or("cleanup statement cursor is invalid")?;
        if !matches!(event.effect, Effect::Statement)
            || !matches!(event.location, Location::At { block, before } if block == frame.block && before.checked_add(1) == Some(frame.next))
        {
            return Err("cleanup statement completion changed its actual event".into());
        }
        set_cursor(
            frame,
            current.checked_add(1).ok_or("cleanup cursor overflow")?,
        )
    }
}
pub(super) fn advance(frame: &mut Frame, event: usize) -> Result<(), Failure> {
    set_cursor(
        frame,
        event
            .checked_add(1)
            .ok_or("cleanup return cursor overflow")?,
    )
}
fn receiver(
    facts: &crate::sema::model::Declarations,
    method: crate::types::instance::Instance,
) -> Result<crate::types::TypeId, &'static str> {
    let Some(crate::types::Kind::FunctionItem { signature, .. }) = facts.types.kind(method.ty())
    else {
        return Err("cleanup lost its actual function item");
    };
    let Some(crate::types::Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
        return Err("cleanup lost its actual signature");
    };
    let Some([receiver]) = facts.types.list(parameters) else {
        return Err("cleanup lost its actual single receiver");
    };
    Ok(*receiver)
}
