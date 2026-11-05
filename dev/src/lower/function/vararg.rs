//! Actual source entry storage initializes one real cursor through typed `LowIR`.
use super::{Lower, ir};
impl Lower<'_, '_, '_> {
    pub(super) fn source_vararg_read(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<super::Selected>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("source va_arg Call disappeared")?;
        let ir::Rvalue::Call { callee, .. } = value.kind else {
            return Ok(None);
        };
        let callable = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("source va_arg callee disappeared")?
            .ty;
        let Some(read) = crate::body::vararg::read_call(
            self.context.constants,
            self.context.types,
            self.body,
            value,
            callable,
        )?
        else {
            return Ok(None);
        };
        let instance = crate::types::instance::Instance::new(self.context.types, callable)?;
        self.context
            .require_reached_body(self.body_index, self.body)?;
        if !matches!(self.value(callee)?,super::Selected::Item(actual) if actual==self.body.callable_target(instance))
        {
            return Err("source va_arg callable differs from its actual complete instance");
        }
        self.context
            .constants
            .vararg_target(self.context.layouts, read.list)?;
        let ty = super::super::context::representation(self.context.types, read.selected.output)?;
        let pointer = self.value(read.input)?.runtime()?;
        let value = self
            .builder
            .va_arg(&self.context.output, self.native, ty, pointer)?;
        Ok(Some(super::Selected::Runtime(value)))
    }
    pub(super) fn initialize_vararg(
        &mut self,
        local: ir::LocalId,
        location: crate::body::storage::Location,
    ) -> Result<(), &'static str> {
        let crate::body::storage::Location::At { block, before } = location else {
            return Err("variadic initialization requires its actual storage statement");
        };
        if block.0 != 0 {
            return Err("variadic initialization must be in the actual entry block");
        }
        self.vararg
            .as_ref()
            .ok_or("variadic initialized storage has no authenticated entry plan")?
            .publication(self.body, local, self.source, before)?;
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let address = self
            .locals
            .get(local.0 as usize)
            .copied()
            .flatten()
            .ok_or("variadic cursor has no actual owned storage")?;
        let pointer = self.pointer(address)?;
        self.builder
            .va_start(&self.context.output, self.native, pointer)
    }
}
