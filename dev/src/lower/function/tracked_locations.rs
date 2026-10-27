//! Caller data and the hidden runtime ABI never change the source signature.
use super::{Lower, Operand, Selected, Type, Value, ir};
use crate::{
    sema::values::LocationId,
    types::{Kind, instance::Instance},
};

impl Lower<'_, '_, '_> {
    fn caller_pointer(&mut self, id: ir::ValueId) -> Result<Value, &'static str> {
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let site = self
            .body
            .caller_site(id)?
            .ok_or("tracked call lacks its actual sealed caller site")?;
        let ir::Rvalue::Call { callee, .. } = self.body.values[id.0 as usize].kind else {
            return Err("caller site is not an actual Call");
        };
        if self
            .body
            .values
            .get(callee.0 as usize)
            .is_none_or(|value| value.ty != site.callable)
        {
            return Err("caller site changed its actual callee");
        }
        if let ir::Owner::Function { definition, .. } = self.body.owner
            && self.context.constants.caller_tracking(definition)?
        {
            let parameters = self.context.output.parameters(self.native)?;
            if parameters.last() != Some(&Type::Ptr) {
                return Err("tracked body lost its hidden caller pointer");
            }
            return Ok(Value {
                ty: Type::Ptr,
                operand: Operand::Parameter(
                    u32::try_from(parameters.len() - 1)
                        .map_err(|_| "caller ABI parameter overflow")?,
                ),
            });
        }
        self.static_caller_pointer(site.location)
    }
    fn static_caller_pointer(&mut self, location: LocationId) -> Result<Value, &'static str> {
        self.context.constants.caller_location_value(location)?;
        let data = self.context.constants.location(location)?;
        let global = self.context.constant_data(
            super::super::data::Key::Location(location),
            data.value.ok_or("caller allocation is uninitialized")?,
            data.ty,
            true,
        )?;
        self.builder.address(Operand::Global(global))
    }
    pub(super) fn source_caller_location(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("caller intrinsic value disappeared")?;
        let ir::Rvalue::Call {
            signature,
            callee,
            arguments,
        } = value.kind
        else {
            return Ok(None);
        };
        let ty = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("caller intrinsic callee disappeared")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(None);
        }
        let instance = Instance::new(self.context.types, ty)?;
        let Some(selected) = self
            .context
            .constants
            .source_caller_location(instance, self.body.source)?
        else {
            return Ok(None);
        };
        if signature != selected.signature
            || value.ty != selected.result
            || !self
                .body
                .arguments
                .get(arguments.start as usize..arguments.end as usize)
                .is_some_and(<[ir::ValueId]>::is_empty)
            || !matches!(self.value(callee)?, Selected::Item(actual) if actual == self.body.callable_target(instance))
        {
            return Err("caller intrinsic differs from its actual typed direct Call");
        }
        self.caller_pointer(id).map(Selected::Runtime).map(Some)
    }
    pub(super) fn tracked_call(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("tracked call disappeared")?;
        let ir::Rvalue::Call {
            signature,
            callee,
            arguments,
        } = value.kind
        else {
            return Ok(None);
        };
        let ty = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("tracked callee disappeared")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(None);
        }
        let instance = self
            .body
            .callable_target(Instance::new(self.context.types, ty)?);
        let Some(Kind::FunctionItem { item, .. }) = self.context.types.kind(instance.ty()) else {
            return Err("tracked target lost its item");
        };
        if !self
            .context
            .constants
            .caller_tracking(crate::resolve::model::DefId(item.0))?
        {
            return Ok(None);
        }
        let result = value.ty;
        let location = self.caller_pointer(id)?;
        if !matches!(self.value(callee)?, Selected::Item(actual) if actual == instance) {
            return Err("tracked direct call changed its selected instance");
        }
        let symbol = self.context.demand(instance)?;
        self.call_arguments_with_caller(
            signature,
            Operand::Symbol(symbol),
            super::argument_sources::Arguments::Values(arguments),
            result,
            Some(location),
        )
        .map(Some)
    }
}
