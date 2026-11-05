//! Borrow completed vararg source contracts without exposing semantic storage.
use super::Reader;
use crate::{
    resolve::body_origin::BodyOrigin,
    sema::compiler_items::vararg::{self, BindingRequest, Read},
    types::{RegionId, TypeId},
};
impl Reader<'_> {
    pub(crate) fn vararg_work(self, steps: usize) -> Result<(), &'static str> {
        vararg::charge(self.facts, steps)
    }
    pub(crate) fn require_vararg_binding(
        self,
        request: BindingRequest,
    ) -> Result<(), &'static str> {
        vararg::require_binding(self.facts, request)
    }
    pub(crate) fn source_vararg(
        self,
        callable: TypeId,
        source: BodyOrigin,
    ) -> Result<Option<Read>, &'static str> {
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        catalog.vararg_read(self.facts, callable, source)
    }
    pub(crate) fn vararg_opened(
        self,
        read: &Read,
        signature: TypeId,
    ) -> Result<(TypeId, [RegionId; 2]), &'static str> {
        read.opened(self.facts, signature)
    }
    pub(crate) fn vararg_target(
        self,
        layouts: &mut crate::abi::layout::Layouts,
        ty: TypeId,
    ) -> Result<(u64, u32), &'static str> {
        vararg::target(self.facts, layouts, ty)
    }
}
