//! A shared reference exposes its active enum tag without reading payload bytes.
use super::{Constant, Failure, Frame, Machine, code::Code, executed_calls::Answer};
use crate::{
    sema::{model::Scalar, values::Reader},
    support::bindings::SourceIntrinsic,
    types::{Kind, TypeId, aggregate, instance::Instance},
};

impl Machine<'_, '_> {
    pub(super) fn source_discriminant(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let facts = self.evaluator.facts();
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(target.ty()) else {
            return Ok(None);
        };
        if facts
            .source_bindings
            .as_deref()
            .map(|c| c.intrinsic_kind(crate::resolve::model::DefId(item.0)))
            .transpose()?
            .flatten()
            != Some(SourceIntrinsic::DiscriminantValue)
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
            return Err("source discriminant belongs to another reached caller".into());
        }
        let selected = self
            .evaluator
            .checked_source_discriminant(target, body.source)?
            .ok_or("source discriminant lost its checked source declaration")?;
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
            .ok_or("source discriminant argument range changed")?
        else {
            return Err("source discriminant requires one evaluated shared reference".into());
        };
        if body
            .values
            .get(input.0 as usize)
            .is_none_or(|v| !selected.input_matches(&self.evaluator.facts().types, v.ty))
        {
            return Err("source discriminant input differs from its actual reference type".into());
        }
        // Ordinary call proof owns region compatibility; extraction keeps this exact evaluated reference.
        let address = Self::value(frame, *input)?;
        self.path(address)?;
        let enum_ = match self.evaluator.facts().types.kind(selected.referent()) {
            Some(Kind::Adt(item, _)) => self
                .evaluator
                .facts()
                .aggregates
                .get(item)
                .is_some_and(|d| d.kind == aggregate::Kind::Enum),
            _ => false,
        };
        let bits = if enum_ {
            let super::memory::Read::Value(variant) =
                self.read_active_variant(super::memory::Resolved {
                    address,
                    ty: selected.referent(),
                    raw: None,
                    union: None,
                })?
            else {
                return Ok(Some(Answer::Pending));
            };
            let Some((primitive, bits)) =
                Reader::new(self.evaluator.facts()).discriminant(variant)?
            else {
                self.waiting = Some(super::Requirement::DiscriminantValue(variant));
                return Ok(Some(Answer::Pending));
            };
            if TypeId::primitive(primitive) != selected.output() {
                return Err(
                    "source discriminant logical result differs from declared representation"
                        .into(),
                );
            }
            bits
        } else {
            0
        };
        self.work.calls += 1;
        Ok(Some(Answer::Ready(Constant::scalar(Scalar::Integer(bits)))))
    }
}
