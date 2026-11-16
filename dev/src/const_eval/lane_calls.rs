//! A reached `simd_splat` fills every lane of its closed vector with its one
//! evaluated value; the other SIMD intrinsics keep the unimplemented error.
use super::{Failure, Frame, Machine, code::Code, executed_calls::Answer};
use crate::{
    sema::compiler_items::SimdOperation,
    types::{aggregate::SimdShape, instance::Instance},
};
impl Machine<'_, '_> {
    pub(super) fn source_splat(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = self.evaluator.checked_source_simd(target, body.source)? else {
            return Ok(None);
        };
        if selected.operation != SimdOperation::Splat {
            return Err("source intrinsic constant execution is not implemented".into());
        }
        self.charge(64)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|current| !std::ptr::eq(current, body))
        {
            return Err("source simd_splat belongs to another reached caller".into());
        }
        let facts = self.evaluator.facts();
        let arguments = super::reached_layout::checked_call_arguments_with_signature(
            facts,
            body,
            frame,
            position,
            target,
            (selected.definition, selected.output),
            |actual, public| {
                selected.opened_signature_matches(&facts.types, actual)
                    && selected.opened_signature_matches(&facts.types, public)
            },
        )?;
        facts
            .source_bindings
            .as_deref()
            .ok_or("source simd_splat catalog disappeared")?
            .check_const_call(selected.definition)?;
        let (Some(&[input]), &[lane_type]) = (
            body.arguments
                .get(arguments.start as usize..arguments.end as usize),
            selected.inputs(),
        ) else {
            return Err("source simd_splat requires its one evaluated argument".into());
        };
        if body
            .values
            .get(input.0 as usize)
            .is_none_or(|value| value.ty != lane_type)
        {
            return Err("source simd_splat changed its actual argument type".into());
        }
        let SimdShape::Ready { lane, count } =
            facts.aggregates.simd_shape(&facts.types, selected.output)?
        else {
            return Err("source simd_splat requires a closed SIMD vector type".into());
        };
        if lane != lane_type {
            return Err(
                "invalid monomorphization of simd_splat: the value's type is not the vector's element type"
                    .into(),
            );
        }
        self.charge(usize::try_from(count).map_err(|_| "simd_splat lane count overflow")?)?;
        let value = Self::value(frame, input)?;
        let lanes = self.evaluator.repeat(value, count)?;
        let vector = self.evaluator.fields(&[lanes], None)?;
        self.work.calls += 1;
        Ok(Some(Answer::Ready(vector)))
    }
}
