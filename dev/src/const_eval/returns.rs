//! Returning frames publish completed storage before their suspended parent resumes.
use super::{Failure, Frame, Machine, Owner, ValueId, code::Code};
use crate::sema::values::Constant;

impl Machine<'_, '_> {
    pub(super) fn returned(
        &mut self,
        frame: &Frame,
        body: &Code,
        value: ValueId,
    ) -> Result<Option<Constant>, Failure> {
        let result_ty = body
            .values
            .get(value.0 as usize)
            .ok_or("constant return lacks a typed value")?
            .ty;
        let result = Self::value(frame, value)?;
        self.slots -= frame.locals.len() + frame.values.len();
        self.memory.expire(&frame.allocations);
        if let Owner::Initializer { definition, .. } = body.owner {
            self.active_initializers.remove(&definition);
            self.validate_result(result, result_ty)?;
            self.evaluator.complete_definition(definition, result);
        }
        if let Owner::InitializerInstance { instance, .. } = body.owner {
            if !self.active_associated.remove(&(instance, body.result)) {
                return Err("selected initializer return lost active identity".into());
            }
            self.validate_result(result, result_ty)?;
            if body.source_only {
                self.evaluator
                    .complete_source_associated(instance, result)?;
            } else {
                self.evaluator
                    .complete_associated(instance, body.result, result)?;
            }
        }
        if self.active_inline.remove(&body.source) {
            self.validate_result(result, result_ty)?;
            self.evaluator.complete_inline(body.source, result)?;
        }
        if let Owner::AnonymousInstance(instance) = body.owner {
            if !self.active_anonymous.remove(&(instance, body.result)) {
                return Err("anonymous return lost its active identity".into());
            }
            self.validate_result(result, result_ty)?;
            self.evaluator
                .complete_inline_instance(instance, body.result, result)?;
        }
        if let Some(parent) = self.frames.last_mut() {
            // Storage-read preparation publishes its initializer without a value destination.
            if frame.return_to.is_none() && matches!(body.owner, Owner::Initializer { .. }) {
                return Ok(None);
            }
            let target = frame
                .return_to
                .ok_or("constant return has no destination")?;
            parent.values[target.0 as usize] = Some(result);
            return Ok(None);
        }
        self.validate_result(result, result_ty)?;
        Ok(Some(result))
    }
}
