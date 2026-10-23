//! A proved reached source selector chooses the evaluated runtime callback.
use super::{Lower, Selected, argument_sources::Arguments, ir};
use crate::{
    lowir::{program::Operand, vocabulary::Type},
    sema::compiler_items::Select,
    types::{Kind, TypeId, callable::Convention, instance::Instance},
};

/// Borrowed only from the actual body entering Lower. This private witness is
/// not clonable and carries no permission outside its exact reached operation.
pub(super) struct RuntimeCall<'body> {
    body: &'body ir::Body,
    value: ir::ValueId,
    selection: Select,
    operands: [ir::ValueId; 3],
    /// The checked value producer, distinct from its constrained logical G type.
    origin: Option<ir::ValueId>,
}
impl<'body> Lower<'_, 'body, '_> {
    pub(super) fn runtime_select(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<RuntimeCall<'body>>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("runtime selector call value disappeared")?;
        let ir::Rvalue::Call {
            signature, callee, ..
        } = value.kind
        else {
            return Err("runtime selector witness requires its actual direct Call");
        };
        let callee_type = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("runtime selector callee disappeared")?
            .ty;
        if !matches!(
            self.context.types.kind(callee_type),
            Some(Kind::FunctionItem { .. })
        ) {
            return Ok(None);
        }
        let instance = Instance::new(self.context.types, callee_type)?;
        let Some(selection) = self.context.constants.source_select(instance)? else {
            return Ok(None);
        };
        self.context
            .require_reached_body(self.body_index, self.body)?;
        // This actual original/materialized caller already paid every source
        // call clause, including F:const FnOnce. Select is only declaration
        // and relocation data; neither its cache nor this ABI adapter pays it.
        if !matches!(self.context.types.kind(callee_type), Some(Kind::FunctionItem { item, signature, .. })
            if item.0 == selection.definition.0 && signature == selection.signature)
            || selection.instance != instance
            || !matches!(self.value(callee)?, Selected::Item(actual)
                if actual == self.body.callable_target(instance))
        {
            return Err("runtime selector differs from its evaluated actual callee");
        }
        opened_signature(self.context.types, signature, value.ty, selection)?;
        let operands = crate::body::operand_origins::inputs(self.body, id, selection.types)?;
        for operand in operands {
            self.value(operand)?;
        }
        let origin = if matches!(
            self.context.types.kind(selection.types[2]),
            Some(Kind::FunctionItem { .. })
        ) {
            let origins = self.callable_origins.get_or_insert_with(|| {
                Box::new(crate::body::operand_origins::Origins::new(self.body))
            });
            origins.prepare(self.context.types, |_| Ok(()))?;
            Some(origins.origin(self.context.types, self.body, operands[2])?)
        } else {
            None
        };
        Ok(Some(RuntimeCall {
            body: self.body,
            value: id,
            selection,
            operands,
            origin,
        }))
    }
}

impl Lower<'_, '_, '_> {
    #[expect(
        clippy::needless_pass_by_value,
        reason = "consume the private non-clonable reached-call witness at its single runtime operation"
    )]
    pub(super) fn selected_callback(
        &mut self,
        call: RuntimeCall<'_>,
    ) -> Result<Selected, &'static str> {
        if !std::ptr::eq(call.body, self.body)
            || self.body.values.get(call.value.0 as usize).is_none()
        {
            return Err("runtime selector witness belongs to another reached body");
        }
        // MIR transformation consumes the three already evaluated operands in
        // source order, then spreads ARG into G. It introduces no drop of F.
        let argument = self.value(call.operands[0])?;
        self.value(call.operands[1])?;
        let runtime = self.value(call.operands[2])?;
        let (signature, callee) =
            self.runtime_callback(call.selection.types[2], runtime, call.origin)?;
        self.call_arguments(
            signature,
            callee,
            Arguments::Tuple {
                owner: call.selection.types[0],
                value: argument,
            },
            call.selection.types[3],
        )
    }

    fn runtime_callback(
        &mut self,
        ty: TypeId,
        selected: Selected,
        origin: Option<ir::ValueId>,
    ) -> Result<(TypeId, Operand), &'static str> {
        match (self.context.types.kind(ty), selected) {
            (Some(Kind::FunctionItem { .. }), Selected::Item(actual)) => {
                let origin = origin.ok_or("runtime selector callback lacks its checked origin")?;
                let source = self
                    .body
                    .values
                    .get(origin.0 as usize)
                    .ok_or("runtime selector callback origin disappeared")?;
                let raw = Instance::new(self.context.types, source.ty)?;
                if actual != self.body.callable_target(raw) {
                    return Err("runtime selector callback lost its checked executable mapping");
                }
                let Some(Kind::FunctionItem { signature, .. }) =
                    self.context.types.kind(actual.ty())
                else {
                    return Err("runtime selector callback target has no actual signature");
                };
                Ok((signature, Operand::Symbol(self.context.demand(actual)?)))
            }
            (Some(Kind::Function { .. }), Selected::Runtime(value))
                if value.ty == Type::Ptr && origin.is_none() =>
            {
                Ok((ty, value.operand))
            }
            _ => Err("runtime selector callback requires its evaluated item or function pointer"),
        }
    }
}

fn opened_signature(
    types: &crate::types::Store,
    signature: TypeId,
    result: TypeId,
    selection: Select,
) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result: output,
        unsafe_: false,
        convention: Convention::RUST,
    }) = types.kind(signature)
    else {
        return Err("runtime selector lacks its opened safe direct-call signature");
    };
    if types.list(parameters) != Some(&selection.types[..3])
        || output != selection.types[3]
        || result != selection.types[3]
    {
        return Err("runtime selector opened call differs from its full logical instance");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/selector_signatures.rs"]
mod tests;
