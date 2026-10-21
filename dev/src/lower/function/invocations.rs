//! Concrete function call operators reuse the ordinary call ABI after receiver access.
use super::{Lower, Selected};
use crate::{
    body::ir::ValueList,
    lowir::vocabulary::Type,
    types::{Kind, TypeId, instance::Instance},
};
impl Lower<'_, '_> {
    pub(super) fn invoke(
        &mut self,
        signature: TypeId,
        arguments: ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let receiver = *self
            .body
            .arguments(arguments)
            .first()
            .ok_or("missing call operator receiver")?;
        let mut ty = self
            .context
            .types
            .storage_head(self.body.values[receiver.0 as usize].ty)?;
        let mut value = self.value(receiver)?;
        while let Some(Kind::Reference { referent, .. }) = self.context.types.kind(ty) {
            ty = self.context.types.storage_head(referent)?;
            if matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
                value = Selected::Item(Instance::new(self.context.types, ty)?);
            } else if !matches!(self.context.types.kind(ty), Some(Kind::Closure { .. })) {
                let pointer = value.runtime()?;
                value = Selected::Runtime(self.builder.load(Type::Ptr, pointer.operand)?);
            }
        }
        if matches!(self.context.types.kind(ty), Some(Kind::Closure { .. })) {
            let symbol = self.context.demand_closure(ty)?;
            return self.call_operand(
                signature,
                crate::lowir::program::Operand::Symbol(symbol),
                ValueList {
                    start: arguments.start + 1,
                    ..arguments
                },
                result,
            );
        }
        match self.context.types.kind(ty) {
            Some(Kind::FunctionItem { .. }) => {
                // An owned opaque result may physically store only a ZST. Its
                // revealed item identity still selects the executable target.
                value = Selected::Item(Instance::new(self.context.types, ty)?);
            }
            Some(Kind::Function { .. }) => {}
            _ => return Err("call operator requires a concrete executable adapter"),
        }
        self.call_selected(
            signature,
            value,
            ValueList {
                start: arguments.start + 1,
                ..arguments
            },
            result,
        )
    }
}
