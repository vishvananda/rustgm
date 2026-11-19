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
    pub(super) fn has_c_boundary(&self) -> bool {
        self.body
            .owner
            .signatures()
            .is_some_and(|(signature, _)| self.context.has_c_boundary(signature))
    }
    pub(super) fn object_parameters(&mut self) -> Result<(), &'static str> {
        let mut parameter = 0_u32;
        for (index, local) in self.body.locals.iter().enumerate() {
            if !matches!(local.kind, LocalKind::Parameter(_)) {
                continue;
            }
            let representation = self.context.representation(local.ty)?;
            let indirect = self.context.indirect_argument(local.ty)?;
            if indirect.is_none() && matches!(representation, Representation::Scalar(Type::Void)) {
                continue;
            }
            let expected = self
                .context
                .output
                .parameter_type(self.native, parameter as usize)?;
            if let Some(span) = indirect {
                if expected != boundary::Type::Scalar(Type::Ptr) {
                    return Err("indirect incoming argument requires its canonical pointer");
                }
                if span.bytes != 0 {
                    let destination =
                        self.locals[index].ok_or("missing indirect parameter snapshot")?;
                    let target = self.pointer(destination)?;
                    self.builder.copy_object(
                        span,
                        Value {
                            ty: Type::Ptr,
                            operand: Operand::Parameter(parameter),
                        },
                        target,
                    )?;
                }
                parameter = parameter
                    .checked_add(1)
                    .ok_or("too many C incoming parameters")?;
                continue;
            }
            let operand = Operand::Parameter(parameter);
            match (representation, expected) {
                (Representation::Scalar(ty), boundary::Type::Scalar(expected))
                    if ty == expected =>
                {
                    self.write(
                        self.locals[index],
                        Selected::Runtime(Value { ty, operand }),
                        local.ty,
                    )?;
                }
                (Representation::ScalarObject(storage), boundary::Type::Scalar(expected))
                    if storage.ty == expected =>
                {
                    self.write(
                        self.locals[index],
                        Selected::Runtime(Value {
                            ty: storage.ty,
                            operand,
                        }),
                        local.ty,
                    )?;
                }
                (Representation::Object(span), boundary::Type::Object(expected))
                    if span == expected.span =>
                {
                    let destination = self.locals[index].ok_or("missing C parameter snapshot")?;
                    let pointer = self.pointer(destination)?;
                    self.builder.copy_object_value(
                        &self.context.output,
                        self.native,
                        boundary::ObjectValue { operand, span },
                        pointer,
                    )?;
                }
                (Representation::Object(_), boundary::Type::Scalar(ty))
                    if self.context.c_argument(local.ty)? == expected =>
                {
                    if let Some(destination) = self.locals[index] {
                        self.builder
                            .store(Value { ty, operand }, destination.operand)?;
                    }
                }
                _ => return Err("C incoming parameter differs from canonical ABI plan"),
            }
            parameter = parameter
                .checked_add(1)
                .ok_or("too many C incoming parameters")?;
        }
        if parameter as usize != self.context.output.parameter_count(self.native)? {
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
        arguments: super::argument_sources::Arguments,
        result: crate::types::TypeId,
    ) -> Result<Selected, &'static str> {
        let mut retained = std::mem::take(&mut self.object_arguments);
        retained.clear();
        let fixed = self.context.output.parameter_count(signature)?;
        let variadic = self.context.output.variadic(signature)?;
        let mut tail = Vec::new();
        for index in 0..arguments.len(self)? {
            let (ty, value) = arguments.get(self, index)?;
            let indirect = self.context.indirect_argument(ty)?;
            if variadic
                && retained.len() >= fixed
                && (indirect.is_some() || !matches!(value, Selected::Unit | Selected::Item(_)))
            {
                tail.push(self.context.c_argument(ty)?);
            }
            if let Some(span) = indirect {
                let destination = Address {
                    operand: Operand::Slot(self.builder.object_slot(MemorySpan {
                        bytes: span.bytes.max(1),
                        ..span
                    })?),
                    align: span.align,
                };
                self.write(Some(destination), value, ty)?;
                retained.push(boundary::Value::Scalar(self.pointer(destination)?));
                continue;
            }
            match value {
                Selected::Runtime(value) => retained.push(boundary::Value::Scalar(value)),
                Selected::Object(address) => match self.context.representation(ty)? {
                    Representation::Object(span) => {
                        retained.push(self.boundary_object_argument(ty, address, span)?);
                    }
                    Representation::ScalarObject(storage) => {
                        retained.push(boundary::Value::Scalar(
                            self.builder.load(storage.ty, address.operand)?,
                        ));
                    }
                    Representation::Scalar(_) => {
                        return Err("C object argument lacks object storage");
                    }
                },
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
        self.boundary_result(value, result)
    }
    fn boundary_object_argument(
        &mut self,
        source: crate::types::TypeId,
        address: Address,
        span: MemorySpan,
    ) -> Result<boundary::Value, &'static str> {
        Ok(match self.context.c_argument(source)? {
            boundary::Type::Object(object) if object.span == span => {
                boundary::Value::Object(self.object_value(address, span)?)
            }
            boundary::Type::Scalar(ty) if ty != Type::Void => {
                boundary::Value::Scalar(self.builder.load(ty, address.operand)?)
            }
            _ => return Err("C object argument differs from canonical ABI plan"),
        })
    }
    fn boundary_result(
        &mut self,
        value: boundary::ResultValue,
        result: crate::types::TypeId,
    ) -> Result<Selected, &'static str> {
        Ok(match (value, self.context.representation(result)?) {
            (boundary::ResultValue::Void, Representation::Scalar(Type::Void)) => Selected::Unit,
            (boundary::ResultValue::Scalar(value), Representation::Scalar(ty))
                if value.ty == ty =>
            {
                Selected::Runtime(value)
            }
            (boundary::ResultValue::Scalar(value), Representation::ScalarObject(storage))
                if value.ty == storage.ty =>
            {
                self.materialize_c_scalar(value, result)?
            }
            (boundary::ResultValue::Scalar(value), Representation::Object(_))
                if self.context.c_result(result)? == boundary::Type::Scalar(value.ty) =>
            {
                self.materialize_c_scalar(value, result)?
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
    fn materialize_c_scalar(
        &mut self,
        value: Value,
        result: crate::types::TypeId,
    ) -> Result<Selected, &'static str> {
        let destination = self
            .allocate(result)?
            .ok_or("missing C scalar result storage")?;
        self.builder.store(value, destination.operand)?;
        Ok(Selected::Object(destination))
    }
    fn c_result_storage(&mut self, ty: Type) -> Result<bool, &'static str> {
        if ty == Type::Void {
            return Ok(false);
        }
        let result = self.body.result;
        Ok(match self.context.representation(result)? {
            Representation::ScalarObject(storage) => storage.ty == ty,
            Representation::Object(_) => {
                self.context.c_result(result)? == boundary::Type::Scalar(ty)
            }
            Representation::Scalar(_) => false,
        })
    }
    pub(super) fn returned(&mut self, value: ir::ValueId) -> Result<Terminator, &'static str> {
        let selected = self.value(value)?;
        if self.has_c_boundary() {
            let expected = self.context.output.result_type(self.native)?;
            return match (selected, expected) {
                (Selected::Runtime(value), boundary::Type::Scalar(ty)) if value.ty == ty => {
                    Ok(Terminator::Return(Some(value)))
                }
                (Selected::Unit | Selected::Item(_), boundary::Type::Scalar(Type::Void)) => {
                    Ok(Terminator::Return(None))
                }
                (Selected::Object(address), boundary::Type::Scalar(ty))
                    if self.c_result_storage(ty)? =>
                {
                    Ok(Terminator::Return(Some(
                        self.builder.load(ty, address.operand)?,
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
                if let Representation::ScalarObject(storage) =
                    self.context.representation(self.body.result)?
                {
                    return Ok(Terminator::Return(Some(
                        self.builder.load(storage.ty, address.operand)?,
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
