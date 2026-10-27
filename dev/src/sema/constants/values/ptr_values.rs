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
