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
            || !selected.opened_signature_matches(self.context.types, signature)
            || self
                .body
                .values
                .get(input.0 as usize)
                .is_none_or(|v| v.ty != selected.source)
            || !matches!(self.value(callee)?,Selected::Item(actual) if actual==self.body.callable_target(instance))
        {
            return Err("source transmute differs from its evaluated full call/instance");
        }
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

#[cfg(test)]
#[path = "../../../tests/invariants/reinterpreted_calls.rs"]
mod tests;
