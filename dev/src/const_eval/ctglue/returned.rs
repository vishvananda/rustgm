//! The owned popped child pays exactly its parent's current cleanup continuation.
use super::{ctsites, drive};
use crate::{
    const_eval::{Failure, Frame, Machine, ReturnTo, code::Code},
    sema::values::{Constant, Reader, View},
    types::TypeId,
};
impl Machine<'_, '_> {
    pub(in crate::const_eval) fn cleanup_returned(
        &mut self,
        child: Frame,
        code: &Code,
        result: Constant,
        result_ty: TypeId,
    ) -> Result<(), Failure> {
        let ReturnTo::Cleanup(expected) = child.return_to else {
            return Err("cleanup return has another destination".into());
        };
        if result_ty != TypeId::UNIT
            || code.result != TypeId::UNIT
            || !matches!(
                Reader::new(self.evaluator.facts()).view(result)?,
                View::Product {
                    length: 0,
                    variant: None
                }
            )
        {
            return Err("cleanup return is not its actual Unit value".into());
        }
        let parent = self
            .frames
            .last()
            .ok_or("cleanup return has no live actual parent")?;
        let expected_parent = child
            .cleanup
            .as_ref()
            .and_then(|state| state.parent)
            .ok_or("cleanup child lost its actual parent identity")?;
        if usize::try_from(expected_parent.get() - 1)
            .map_err(|_| "cleanup parent identity overflow")?
            != parent.instance
            || ctsites::cursor(parent)?
                != usize::try_from(expected).map_err(|_| "cleanup event identity overflow")?
        {
            return Err("cleanup return changed its actual parent or event".into());
        }
        self.bodies.validate_entry(parent.body)?;
        let body = self.bodies[parent.body].clone();
        let bundle = body
            .cleanup()
            .ok_or("cleanup parent lost its owned footer")?;
        let mut work = crate::body::defining_cleanups::Work::new(self.limits.steps);
        let actual = ctsites::current(self.evaluator.facts(), &body, parent, bundle, &mut work);
        self.charge(work.steps)?;
        let (event, method, _) = actual?;
        if self.bodies.functions.get(&method.instance) != Some(&child.body) {
            return Err("cleanup return came from another actual selected body".into());
        }
        self.validate_invoked_entry(child.body, method.instance)?;
        let parent = self
            .frames
            .last()
            .ok_or("cleanup return lost its live parent")?;
        if !parent.cleanup_allowed(&body, event)? {
            return Err("cleanup return changed its actual initialized condition".into());
        }
        drop(child);
        let parent = self
            .frames
            .last_mut()
            .ok_or("cleanup return lost its actual parent")?;
        drive::advance(parent, event)
    }
}
