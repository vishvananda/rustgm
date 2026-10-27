//! Metadata extraction reads one evaluated pointer, independently of pointed-to memory.
use super::{Constant, Failure, Frame, Machine, code::Code, executed_calls::Answer};
use crate::{
    sema::{model::Scalar, values::Reader},
    support::bindings::SourceIntrinsic,
    types::{Kind, Primitive, TypeId, instance::Instance},
};
impl Machine<'_, '_> {
    pub(super) fn source_ptr_metadata(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let facts = self.evaluator.facts();
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(target.ty()) else {
            return Ok(None);
        };
        if catalog.intrinsic_kind(crate::resolve::model::DefId(item.0))?
            != Some(SourceIntrinsic::PtrMetadata)
        {
            return Ok(None);
        }
        self.charge(64)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|actual| !std::ptr::eq(actual, body))
        {
            return Err("source ptr_metadata belongs to another actual caller code".into());
        }
        let selected = self
            .evaluator
            .checked_source_ptr_metadata(target, body.source)?
            .ok_or("source ptr_metadata lost its actual source selection")?;
        let arguments = super::reached_layout::checked_call_arguments(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            (selected.definition(), selected.output()),
        )?;
        let [input] = body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source ptr_metadata lost its actual arguments")?
        else {
            return Err("source ptr_metadata requires one evaluated pointer argument".into());
        };
        let input_ty = selected.input(self.evaluator.facts())?;
        if body
            .values
            .get(input.0 as usize)
            .is_none_or(|v| v.ty != input_ty)
        {
            return Err("source ptr_metadata argument differs from its exact signature".into());
        }
        // Read the actual SSA value even when its result erases to unit.
        let pointer = Self::value(frame, *input)?;
        let pointer_layout = self.footprint(input_ty)?;
        let value = if selected.output() == TypeId::UNIT {
            if pointer_layout.size != 8 {
                return Err(
                    "source ptr_metadata unit result requires an actual thin pointer".into(),
                );
            }
            self.evaluator.fields(&[], None)?
        } else if selected.output() == TypeId::primitive(Primitive::Usize) {
            if pointer_layout.size != 16 {
                return Err("source ptr_metadata length requires an actual wide pointer".into());
            }
            let length = Reader::new(self.evaluator.facts())
                .pointer_text_length(pointer)?
                .ok_or("source ptr_metadata pointer has no proved length metadata")?;
            Constant::scalar(Scalar::Integer(u128::from(length)))
        } else {
            return Err("source ptr_metadata dynamic or unresolved result requires its real metadata storage producer".into());
        };
        self.work.calls += 1;
        Ok(Some(Answer::Ready(value)))
    }
}
