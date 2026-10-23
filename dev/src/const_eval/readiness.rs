//! Constant operand formation is distinct from reached value execution.
use super::{
    Failure, Frame, Machine, Requirement,
    code::{Inline, PreparedOperand},
};
use crate::sema::values::Reader;
impl Machine<'_, '_> {
    /// Only actual registered source inline bodies enter the source value cache.
    pub(super) fn start_source_inline(&mut self, body: &super::code::Code) -> Result<(), Failure> {
        if !matches!(body.owner, crate::body::ir::Owner::AnonymousConstant) {
            return Ok(());
        }
        let Some(source) = body.source.source() else {
            return Ok(());
        };
        if self
            .evaluator
            .facts()
            .inline_constants
            .as_deref()
            .is_none_or(|registry| registry.get(source).is_none())
        {
            return Ok(());
        }
        self.evaluator.require_inline(source, body.result)?;
        if !self.active_inline.insert(source) {
            return Err(Failure::Fault("cycle in inline constant values"));
        }
        Ok(())
    }
    pub(super) fn cast(
        &mut self,
        kind: crate::sema::model::Cast,
        source: crate::types::TypeId,
        target: crate::types::TypeId,
        value: crate::sema::values::Constant,
    ) -> Result<Option<crate::sema::values::Constant>, Failure> {
        if kind == crate::sema::model::Cast::EnumToInteger {
            let reader = Reader::new(self.evaluator.facts());
            if let crate::sema::values::View::Product {
                variant: Some(id), ..
            } = reader.view(value)?
                && reader.discriminant(id)?.is_none()
            {
                self.waiting = Some(Requirement::DiscriminantValue(id));
                return Ok(None);
            }
        }
        self.evaluator.cast(kind, source, target, value).map(Some)
    }
    pub(super) fn prepare_operands(&mut self, mut frame: Frame) -> Result<Option<Frame>, Failure> {
        let body = self.bodies[frame.body].clone();
        while let Some(&operand) = body.operands.get(frame.operand_cursor) {
            self.charge(1)?;
            let reader = Reader::new(self.evaluator.facts());
            if let PreparedOperand::Discriminant(variant) = operand {
                if reader.discriminant(variant)?.is_none() {
                    self.waiting = Some(Requirement::DiscriminantValue(variant));
                    self.frames.push(frame);
                    return Ok(None);
                }
                frame.operand_cursor += 1;
                continue;
            }
            let PreparedOperand::Inline {
                destination,
                selected,
                result,
            } = operand
            else {
                return Err("invalid constant operand preparation".into());
            };
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
                frame.operand_cursor += 1;
                continue;
            }
            if let Some(target) = target {
                if let Inline::Closed(source) = selected {
                    self.evaluator.require_inline(source, result)?;
                }
                frame.operand_cursor += 1;
                self.frames.push(frame);
                self.push(target, &[], Some(destination))?;
            } else {
                self.waiting = Some(requirement);
                self.frames.push(frame);
            }
            return Ok(None);
        }
        frame.operand_cursor = usize::MAX;
        Ok(Some(frame))
    }
}
