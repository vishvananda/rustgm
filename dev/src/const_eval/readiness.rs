//! Retained inline operands are required independently of runtime branch selection.
use super::{Failure, Frame, Machine, Requirement};
use crate::sema::values::Reader;

impl Machine<'_, '_> {
    pub(super) fn prepare_inline(&mut self, mut frame: Frame) -> Result<Option<Frame>, Failure> {
        let body = self.bodies[frame.body].clone();
        while let Some(&(destination, source, result)) = body.inline.get(frame.inline_cursor) {
            self.charge(1)?;
            if let Some(value) = Reader::new(self.evaluator.facts()).inline(source) {
                frame.values[destination.0 as usize] = Some(value);
                frame.inline_cursor += 1;
                continue;
            }
            if let Some(&target) = self.bodies.expressions.get(&source) {
                self.evaluator.require_inline(source, result)?;
                frame.inline_cursor += 1;
                self.frames.push(frame);
                self.push(target, &[], Some(destination))?;
            } else {
                self.waiting = Some(Requirement::InlineBody { source, result });
                self.frames.push(frame);
            }
            return Ok(None);
        }
        frame.inline_cursor = usize::MAX;
        Ok(Some(frame))
    }
}
