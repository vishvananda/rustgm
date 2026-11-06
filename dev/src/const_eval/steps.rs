//! One actual statement completes before its cleanup timeline cursor advances.
use super::{Action, Failure, Frame, Machine, Progress, ReturnTo};
impl Machine<'_, '_> {
    pub(super) fn enter_successor(
        &mut self,
        mut frame: Frame,
        body: &super::code::Code,
        successor: crate::body::ir::BlockId,
    ) -> Result<(), Failure> {
        frame.block = successor;
        frame.cleanup_cursor = None;
        frame.next = body
            .blocks
            .get(successor.0 as usize)
            .ok_or("invalid constant successor")?
            .start;
        self.frames.push(frame);
        Ok(())
    }

    pub(super) fn run_statement(
        &mut self,
        mut frame: Frame,
        action: Action,
        cleanup: bool,
    ) -> Result<Option<Progress>, Failure> {
        frame.next += 1;
        let call = self.statement(&mut frame, action)?;
        if let Some(requirement) = self.waiting.take() {
            frame.next -= 1;
            self.frames.push(frame);
            if self.initialize_storage(requirement)? {
                return Ok(None);
            }
            return Ok(Some(Progress::Prepare(requirement)));
        }
        if cleanup {
            self.finish_cleanup_statement(&mut frame)?;
        }
        self.frames.push(frame);
        if let Some((target, arguments, destination)) = call {
            self.push(target, &arguments, ReturnTo::Value(destination))?;
        }
        Ok(None)
    }
}
