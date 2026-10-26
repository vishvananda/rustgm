//! A checked by-value move uses owned ABI snapshots, never numeric conversion.
use super::{Address, Lower, Representation, Selected, ir};
use crate::{
    lowir::{program::MemorySpan, vocabulary::Type},
    types::{Kind, instance::Instance},
};
impl Lower<'_, '_, '_> {
    pub(super) fn runtime_transmute(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("source transmute lost its value")?;
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
            .ok_or("source transmute lost its callee")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(None);
        }
        let instance = Instance::new(self.context.types, ty)?;
        let Some(selected) = self
            .context
            .constants
            .source_transmute(instance, self.body.source)?
        else {
            return Ok(None);
        };
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let [input] = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source transmute lost its runtime argument range")?
        else {
            return Err("source transmute requires one evaluated runtime argument");
        };
        if value.ty != selected.destination
            || signature != selected.signature
            || self
                .body
                .values
                .get(input.0 as usize)
                .is_none_or(|v| v.ty != selected.source)
            || !matches!(self.value(callee)?,Selected::Item(actual) if actual==self.body.callable_target(instance))
        {
            return Err("source transmute differs from its evaluated full call/instance");
        }
        self.transmute_function_signature(*input, selected.source, selected.destination)?;
        let source = self.context.layout(selected.source)?;
        let destination = self.context.layout(selected.destination)?;
        if source.size != destination.size {
            return Err("source transmute runtime size relation changed");
        }
        if source.size == 0 {
            return Err("zero-sized transmute requires a supported runtime validity profile");
        }
        let span = MemorySpan {
            bytes: source.size,
            align: source.align.max(destination.align),
        };
        let slot = self.builder.object_slot(span)?;
        let backing = Address {
            operand: crate::lowir::program::Operand::Slot(slot),
            align: span.align,
        };
        self.write(Some(backing), self.value(*input)?, selected.source)?;
        let result = match self.context.representation(selected.destination)? {
            Representation::Scalar(Type::Void) => {
                return Err("source transmute cannot fabricate a void value");
            }
            Representation::Scalar(ty) => {
                Selected::Runtime(self.builder.load(ty, backing.operand)?)
            }
            Representation::Object(_) | Representation::PointerObject(_) => {
                Selected::Object(backing)
            }
        };
        Ok(Some(result))
    }
}

impl Lower<'_, '_, '_> {
    fn transmute_function_signature(
        &mut self,
        input: ir::ValueId,
        source: crate::types::TypeId,
        destination: crate::types::TypeId,
    ) -> Result<(), &'static str> {
        if !matches!(
            self.context.types.kind(destination),
            Some(Kind::Function { .. })
        ) {
            return Ok(());
        }
        if matches!(self.context.types.kind(source), Some(Kind::Function { .. })) {
            return if source == destination {
                Ok(())
            } else {
                Err("function transmute requires an independently proved destination ABI/signature")
            };
        }
        let origins = self
            .callable_origins
            .get_or_insert_with(|| Box::new(crate::body::operand_origins::Origins::new(self.body)));
        origins.prepare_transport(self.context.types)?;
        let mut value = input;
        // Every traversed structural/call edge charges the existing cold owner.
        // A body-size ceiling diagnoses cycles without a fixed chain-depth cap.
        for _ in 0..=self.body.values.len() {
            let edge = origins.transport_origin(self.body, value)?;
            if edge != value {
                value = edge;
                continue;
            }
            let actual = self
                .body
                .values
                .get(value.0 as usize)
                .ok_or("function transmute origin disappeared")?;
            let signature = match self.context.types.kind(actual.ty) {
                Some(Kind::Function { .. }) => Some(actual.ty),
                Some(Kind::FunctionItem { signature, .. }) => {
                    Instance::new(self.context.types, actual.ty)?;
                    Some(signature)
                }
                _ => None,
            };
            if let Some(signature) = signature {
                return if signature == destination {
                    Ok(())
                } else {
                    Err("function transmute changed the recovered original ABI/signature")
                };
            }
            let ir::Rvalue::Call {
                callee,
                arguments,
                signature,
            } = actual.kind
            else {
                return Err("function transmute requires a proved original callable signature");
            };
            let callable = self
                .body
                .values
                .get(callee.0 as usize)
                .ok_or("function transmute producer lost its callee")?;
            let instance = Instance::new(self.context.types, callable.ty)?;
            let selected = self
                .context
                .constants
                .source_transmute(instance, self.body.source)?
                .ok_or("function transmute origin is not an actual source transmute")?;
            let [operand] = self
                .body
                .arguments
                .get(arguments.start as usize..arguments.end as usize)
                .ok_or("function transmute producer lost its arguments")?
            else {
                return Err("function transmute producer requires one actual input");
            };
            if signature != selected.signature
                || selected.destination != actual.ty
                || self
                    .body
                    .values
                    .get(operand.0 as usize)
                    .is_none_or(|v| v.ty != selected.source)
            {
                return Err("function transmute producer changed its full instance/signature");
            }
            value = *operand;
        }
        Err("function transmute origin graph contains a cycle")
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/reinterpreted_calls.rs"]
mod tests;
