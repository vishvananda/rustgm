//! C aggregate calls preserve owned snapshots while the backend assigns native locations.
use super::{Address, Lower, Operand, Representation, Selected, Terminator, Type, Value};
use crate::{
    body::ir::{self, LocalKind},
    lowir::{
        boundary,
        program::{MemorySpan, SignatureId},
    },
};
impl Lower<'_, '_, '_> {
    pub(super) fn object_parameters(&mut self) -> Result<(), &'static str> {
        let mut parameter = 0_u32;
        for (index, local) in self.body.locals.iter().enumerate() {
            if !matches!(local.kind, LocalKind::Parameter(_)) {
                continue;
            }
            let representation = self.context.representation(local.ty)?;
            if matches!(representation, Representation::Scalar(Type::Void)) {
                continue;
            }
            let expected = *self
                .context
                .output
                .object_boundary(self.native)?
                .parameters
                .get(parameter as usize)
                .ok_or("missing C incoming ABI parameter")?;
            let selected = match (representation, expected) {
                (Representation::Scalar(ty), boundary::Type::Scalar(expected))
                    if ty == expected =>
                {
                    Selected::Runtime(Value {
                        ty,
                        operand: Operand::Parameter(parameter),
                    })
                }
                (Representation::PointerObject(_), boundary::Type::Scalar(Type::Ptr)) => {
                    Selected::Runtime(Value {
                        ty: Type::Ptr,
                        operand: Operand::Parameter(parameter),
                    })
                }
                (Representation::Object(span), boundary::Type::Object(expected))
                    if span == expected.span =>
                {
                    let destination = self.locals[index].ok_or("missing C parameter snapshot")?;
                    let pointer = self.pointer(destination)?;
                    self.builder.copy_object_value(
                        &self.context.output,
                        self.native,
                        boundary::ObjectValue {
                            operand: Operand::Parameter(parameter),
                            span,
                        },
                        pointer,
                    )?;
                    parameter = parameter
                        .checked_add(1)
                        .ok_or("too many C incoming parameters")?;
                    continue;
                }
                _ => return Err("C incoming parameter differs from canonical ABI plan"),
            };
            self.write(self.locals[index], selected, local.ty)?;
            parameter = parameter
                .checked_add(1)
                .ok_or("too many C incoming parameters")?;
        }
        if parameter as usize
            != self
                .context
                .output
                .object_boundary(self.native)?
                .parameters
                .len()
        {
            return Err("C incoming parameter count differs from ABI plan");
        }
        Ok(())
    }
    fn object_value(
        &mut self,
        address: Address,
        span: MemorySpan,
    ) -> Result<boundary::ObjectValue, &'static str> {
        if matches!(
            address.operand,
            Operand::Slot(_) | Operand::ObjectTemporary(_) | Operand::Global(_)
        ) {
            return Ok(boundary::ObjectValue {
                operand: address.operand,
                span,
            });
        }
        let slot = self.builder.object_slot(span)?;
        let source = self.pointer(address)?;
        let destination = self.builder.address(Operand::Slot(slot))?;
        self.builder.copy_object(
            MemorySpan {
                bytes: span.bytes,
                align: span.align.min(address.align),
            },
            source,
            destination,
        )?;
        Ok(boundary::ObjectValue {
            operand: Operand::Slot(slot),
            span,
        })
    }
    pub(super) fn boundary_call(
        &mut self,
        signature: SignatureId,
        callee: Operand,
        arguments: ir::ValueList,
        result: crate::types::TypeId,
    ) -> Result<Selected, &'static str> {
        let mut retained = std::mem::take(&mut self.object_arguments);
        retained.clear();
        let fixed = self.context.output.parameter_count(signature)?;
        let variadic = self.context.output.variadic(signature)?;
        let mut tail = Vec::new();
        for id in &self.body.arguments[arguments.start as usize..arguments.end as usize] {
            let value = self.value(*id)?;
            if variadic
                && retained.len() >= fixed
                && !matches!(value, Selected::Unit | Selected::Item(_))
            {
                tail.push(
                    self.context
                        .c_argument(self.body.values[id.0 as usize].ty)?,
                );
            }
            match value {
                Selected::Runtime(value) => retained.push(boundary::Value::Scalar(value)),
                Selected::Object(address) => {
                    match self
                        .context
                        .representation(self.body.values[id.0 as usize].ty)?
                    {
                        Representation::Object(span) => retained
                            .push(boundary::Value::Object(self.object_value(address, span)?)),
                        Representation::PointerObject(_) => retained.push(boundary::Value::Scalar(
                            self.builder.load(Type::Ptr, address.operand)?,
                        )),
                        Representation::Scalar(_) => {
                            return Err("C object argument lacks object storage");
                        }
                    }
                }
                Selected::Unit | Selected::Item(_) => {}
            }
        }
        let value = self.builder.boundary_call(
            &self.context.output,
            signature,
            callee,
            &retained,
            &tail,
        )?;
        self.object_arguments = retained;
        Ok(match (value, self.context.representation(result)?) {
            (boundary::ResultValue::Void, Representation::Scalar(Type::Void)) => Selected::Unit,
            (boundary::ResultValue::Scalar(value), Representation::Scalar(ty))
                if value.ty == ty =>
            {
                Selected::Runtime(value)
            }
            (boundary::ResultValue::Scalar(value), Representation::PointerObject(_))
                if value.ty == Type::Ptr =>
            {
                let destination = self
                    .allocate(result)?
                    .ok_or("missing nullable C result storage")?;
                self.builder.store(value, destination.operand)?;
                Selected::Object(destination)
            }
            (boundary::ResultValue::Object(value), Representation::Object(span))
                if value.span == span =>
            {
                let destination = self.allocate(result)?.ok_or("missing C result snapshot")?;
                let target = self.pointer(destination)?;
                self.builder
                    .copy_object_value(&self.context.output, self.native, value, target)?;
                Selected::Object(destination)
            }
            _ => return Err("C call result differs from canonical ABI plan"),
        })
    }
    pub(super) fn returned(&mut self, value: ir::ValueId) -> Result<Terminator, &'static str> {
        let selected = self.value(value)?;
        if matches!(
            self.context.output.signatures.get(self.native.0 as usize),
            Some(crate::lowir::program::Signature::Object(_))
        ) {
            let expected = self.context.output.object_boundary(self.native)?.result;
            return match (selected, expected) {
                (Selected::Runtime(value), boundary::Type::Scalar(ty)) if value.ty == ty => {
                    Ok(Terminator::Return(Some(value)))
                }
                (Selected::Unit | Selected::Item(_), boundary::Type::Scalar(Type::Void)) => {
                    Ok(Terminator::Return(None))
                }
                (Selected::Object(address), boundary::Type::Scalar(Type::Ptr))
                    if matches!(
                        self.context.representation(self.body.result)?,
                        Representation::PointerObject(_)
                    ) =>
                {
                    Ok(Terminator::Return(Some(
                        self.builder.load(Type::Ptr, address.operand)?,
                    )))
                }
                (Selected::Object(address), boundary::Type::Object(object)) => {
                    let value = self.object_value(address, object.span)?;
                    self.builder
                        .object_return(&self.context.output, self.native, value)
                }
                _ => Err("C return differs from canonical ABI plan"),
            };
        }
        Ok(Terminator::Return(match selected {
            Selected::Runtime(value) => Some(value),
            Selected::Object(address) => {
                if matches!(
                    self.context.representation(self.body.result)?,
                    Representation::PointerObject(_)
                ) {
                    return Ok(Terminator::Return(Some(
                        self.builder.load(Type::Ptr, address.operand)?,
                    )));
                }
                let align = self.context.layout(self.body.result)?.align;
                self.write(
                    Some(Address {
                        operand: Operand::Parameter(0),
                        align,
                    }),
                    Selected::Object(address),
                    self.body.result,
                )?;
                None
            }
            Selected::Unit | Selected::Item(_) => None,
        }))
    }
}
