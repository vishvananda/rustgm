//! Retained inline operands are required independently of runtime branch selection.
use super::{Failure, Frame, Machine, Requirement, code::Inline};
use crate::sema::values::Reader;
impl Machine<'_, '_> {
    pub(super) fn prepare_inline(&mut self, mut frame: Frame) -> Result<Option<Frame>, Failure> {
        let body = self.bodies[frame.body].clone();
        while let Some(&(destination, selected, result)) = body.inline.get(frame.inline_cursor) {
            self.charge(1)?;
            let reader = Reader::new(self.evaluator.facts());
            let (value, target, requirement) = match selected {
                Inline::Closed(source) => (
                    reader.inline(source),
                    self.bodies.expressions.get(&source).copied(),
                    Requirement::InlineBody { source, result },
                ),
                Inline::Instance(instance) => (
                    reader.inline_instance(instance, result),
                    self.bodies
                        .anonymous
                        .as_deref()
                        .and_then(|map| map.get(&(instance, result)).copied()),
                    Requirement::AnonymousBody { instance, result },
                ),
            };
            if let Some(value) = value {
                frame.values[destination.0 as usize] = Some(value);
                frame.inline_cursor += 1;
                continue;
            }
            if let Some(target) = target {
                if let Inline::Closed(source) = selected {
                    self.evaluator.require_inline(source, result)?;
                }
                frame.inline_cursor += 1;
                self.frames.push(frame);
                self.push(target, &[], Some(destination))?;
            } else {
                self.waiting = Some(requirement);
                self.frames.push(frame);
            }
            return Ok(None);
        }
        frame.inline_cursor = usize::MAX;
        Ok(Some(frame))
    }
}
