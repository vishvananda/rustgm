//! A solved Metadata result and the actual pointer ABI jointly determine extracted data.
use super::{Constant, Evaluator, Failure, Reader, View};
use crate::{
    resolve::body_origin::BodyOrigin, sema::compiler_items::PtrMetadataSelection,
    types::instance::Instance,
};
impl Reader<'_> {
    pub(crate) fn source_ptr_metadata(
        self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<PtrMetadataSelection>, &'static str> {
        self.facts
            .source_bindings
            .as_deref()
            .map(|c| c.ptr_metadata_selection(self.facts, instance, origin))
            .transpose()
            .map(Option::flatten)
    }
    pub(crate) fn source_ptr_metadata_input(
        self,
        selected: &PtrMetadataSelection,
    ) -> Result<crate::types::TypeId, &'static str> {
        selected.input(self.facts)
    }
    pub(crate) fn pointer_text_length(
        self,
        pointer: Constant,
    ) -> Result<Option<u64>, &'static str> {
        if let Some(length) = pointer.length() {
            return Ok(Some(length));
        }
        Ok(match self.view(pointer)? {
            View::Text(text) | View::Bytes(text) | View::BytePointer { text } => Some(
                u64::try_from(self.bytes(text)?.len())
                    .map_err(|_| "source pointer text exceeds usize")?,
            ),
            View::ByteRange { length, .. } => Some(length),
            _ => None,
        })
    }
}
impl Evaluator<'_> {
    pub(crate) fn checked_source_ptr_metadata(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<PtrMetadataSelection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.ptr_metadata_selection(self.facts(), instance, origin)? else {
            return Ok(None);
        };
        catalog.require_ptr_metadata_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
}

impl Reader<'_> {
    pub(crate) fn source_raw_parts_inputs(
        self,
        selected: &crate::sema::compiler_items::RawPartsSelection,
    ) -> Result<[crate::types::TypeId; 2], &'static str> {
        selected.inputs(self.facts)
    }
    pub(crate) fn source_raw_parts(
        self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<crate::sema::compiler_items::RawPartsSelection>, &'static str> {
        self.facts
            .source_bindings
            .as_deref()
            .map(|c| c.raw_parts_selection(self.facts, instance, origin))
            .transpose()
            .map(Option::flatten)
    }
}
impl Evaluator<'_> {
    pub(crate) fn checked_source_raw_parts(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<crate::sema::compiler_items::RawPartsSelection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.raw_parts_selection(self.facts(), instance, origin)? else {
            return Ok(None);
        };
        catalog.require_raw_parts_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
}

impl Evaluator<'_> {
    /// Reuse the prepared ABI metadata owner; this is no pointed-to value query.
    pub(crate) fn require_raw_parts_metadata(
        &mut self,
        pointer: crate::types::TypeId,
        argument: crate::types::TypeId,
        limit: usize,
    ) -> Result<usize, Failure> {
        if limit == 0 {
            return Err("source pointer metadata query work exhausted".into());
        }
        let facts = &mut *self.queries.typing.facts;
        let Some(crate::types::Kind::Pointer { pointee, .. }) = facts.types.kind(pointer) else {
            return Err("source pointer metadata query requires a raw pointer type".into());
        };
        let before = facts.layouts.metadata_steps();
        let metadata = facts
            .layouts
            .pointer_metadata(&facts.types, &facts.aggregates, pointee)?;
        let work = facts
            .layouts
            .metadata_steps()
            .saturating_sub(before)
            .saturating_add(1);
        if work > limit {
            return Err("source pointer metadata query work exhausted".into());
        }
        let supported = if argument == crate::types::TypeId::UNIT {
            metadata == crate::abi::metadata::Metadata::Thin
        } else if argument == crate::types::TypeId::primitive(crate::types::Primitive::Usize) {
            metadata == crate::abi::metadata::Metadata::Length
        } else {
            false
        };
        if !supported {
            return Err(
                "source aggregate_raw_ptr metadata needs its actual supported ABI producer".into(),
            );
        }
        Ok(work)
    }
}
