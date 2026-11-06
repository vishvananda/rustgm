//! Literal references retain their real logical view over immutable sized storage.
use super::{Constant, Evaluator, Failure, Metadata, Queries, Reader, TailStorage, Value};
use crate::{
    sema::text,
    types::{Kind, Primitive, TypeId},
};

impl Queries<'_> {
    pub(in crate::sema::constants) fn text_value(
        &mut self,
        text: text::Id,
        ty: TypeId,
    ) -> Result<Value, &'static str> {
        let facts = &mut *self.typing.facts;
        let Some(Kind::Reference {
            referent,
            mutable: false,
            ..
        }) = facts.types.kind(ty)
        else {
            return Err("text literal requires an immutable reference type");
        };
        if referent == TypeId::primitive(Primitive::Str) {
            return Ok(Value::Text(text));
        }
        if matches!(facts.types.kind(referent), Some(Kind::Array { element, .. })
            if element == TypeId::primitive(Primitive::U8))
        {
            return Ok(Value::Bytes(text));
        }
        let profile = crate::sema::compiler_items::c_strings::literal_profile(facts, ty)?
            .ok_or("text literal lacks its actual byte-array, str or CStr type")?;
        let bytes = facts.texts.get(text)?;
        if bytes.last() != Some(&0) {
            return Err("CStr literal backing lacks its trailing NUL");
        }
        let length =
            u64::try_from(bytes.len()).map_err(|_| "CStr literal exceeds target length")?;
        self.steps = self.steps.saturating_add(1);
        if self.steps > self.limit {
            return Err("constant literal storage work budget exceeded");
        }
        let storage = facts.types.intern(Kind::Array {
            element: profile.element,
            length,
        })?;
        let before = facts
            .layouts
            .metadata_steps()
            .saturating_add(facts.layouts.tail_steps())
            .saturating_add(facts.layouts.steps);
        let prepared = (|| {
            let metadata = facts.layouts.pointer_metadata(
                &facts.types,
                &facts.aggregates,
                profile.referent,
            )?;
            if metadata != crate::abi::metadata::Metadata::Length {
                return Err("CStr literal backing has a different actual pointer metadata");
            }
            let crate::sema::members::Selection::Nominal(field) = profile.field else {
                return Err("CStr literal lacks its actual nominal field");
            };
            let index = facts
                .members
                .ordinal(field)
                .ok_or("CStr literal field ordinal is missing")?;
            let (tail, offset) = facts
                .layouts
                .unsized_field(&facts.types, &facts.aggregates, profile.referent, index)
                .map_err(|error| error.message)?;
            if offset != crate::abi::tails::Offset::Fixed(0)
                || !matches!(facts.types.kind(tail), Some(Kind::Slice(element)) if element == profile.element)
            {
                return Err("CStr literal differs from its actual zero-offset byte tail");
            }
            Ok(())
        })();
        let after = facts
            .layouts
            .metadata_steps()
            .saturating_add(facts.layouts.tail_steps())
            .saturating_add(facts.layouts.steps);
        self.steps = self.steps.saturating_add(after.saturating_sub(before));
        if self.steps > self.limit {
            return Err("constant literal metadata work budget exceeded");
        }
        prepared?;
        let value = Constant(facts.constant_references.literal_place(text, storage)?);
        value
            .with_metadata(Metadata::Length(length))
            .map(|v| v.0)
            .map_err(Failure::message)
    }
}

impl Evaluator<'_> {
    /// Consume an already typed literal and retain its immutable backing identity.
    pub(crate) fn text_literal(&mut self, text: text::Id, ty: TypeId) -> Result<Constant, Failure> {
        self.queries
            .text_value(text, ty)
            .map(Constant)
            .map_err(Failure::Invalid)
    }
}

impl Reader<'_> {
    pub(super) fn literal_tail_storage(
        self,
        owner: TypeId,
        actual: TypeId,
        index: u64,
        target: TypeId,
        metadata: Metadata,
    ) -> Result<Option<TailStorage>, &'static str> {
        let types = &self.facts.types;
        if let Some(Kind::Array { element, length }) = types.kind(actual) {
            let profile =
                crate::sema::compiler_items::c_strings::storage_profile(self.facts, owner)?
                    .ok_or("constant tail has a different physical owner")?;
            let crate::sema::members::Selection::Nominal(field) = profile.field else {
                return Err("constant literal tail lacks its actual nominal field");
            };
            let ordinal = self
                .facts
                .members
                .ordinal(field)
                .ok_or("constant literal tail lacks its actual field ordinal")?;
            let (tail, offset) = self
                .facts
                .layouts
                .cached_unsized_field(types, owner, ordinal)
                .ok_or("constant literal tail lacks its completed actual ABI field")??;
            if index != 0
                || element != profile.element
                || metadata != Metadata::Length(length)
                || tail != target
                || offset != crate::abi::tails::Offset::Fixed(0)
                || !matches!(types.kind(target), Some(Kind::Slice(selected)) if selected == element)
            {
                return Err("constant literal tail differs from its element, field or metadata");
            }
            return Ok(Some(TailStorage::Backing(actual)));
        }
        Ok(None)
    }
}
