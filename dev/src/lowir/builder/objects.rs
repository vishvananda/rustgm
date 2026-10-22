//! Object calls share ordinary instructions and keep rare arguments in a separate arena.
use super::{Builder, Type, program};
use crate::lowir::{
    boundary,
    program::{
        BoundaryArgument, CallArguments, ObjectTempId, Operand, Operation, Program, Range,
        ResultId, SignatureId,
    },
};
impl Builder {
    /// # Errors
    /// Rejects cross-kind arguments, invalid object storage and inconsistent boundary spans.
    pub fn object_call(
        &mut self,
        program: &Program,
        signature: SignatureId,
        callee: Operand,
        arguments: &[boundary::Value],
    ) -> Result<boundary::ResultValue, &'static str> {
        program.object_boundary(signature)?;
        self.boundary_call(program, signature, callee, arguments, &[])
    }
    /// Construct a fixed boundary prefix followed by explicitly classified variadic inputs.
    /// # Errors
    /// Rejects inconsistent arity, types, spans and variadic classes.
    pub fn boundary_call(
        &mut self,
        program: &Program,
        signature: SignatureId,
        callee: Operand,
        arguments: &[boundary::Value],
        tail: &[boundary::Type],
    ) -> Result<boundary::ResultValue, &'static str> {
        let fixed = program.parameter_count(signature)?;
        if (!tail.is_empty() && !program.variadic(signature)?)
            || fixed.checked_add(tail.len()) != Some(arguments.len())
        {
            return Err("object call arity differs from its boundary");
        }
        let mut retained = Vec::with_capacity(arguments.len());
        for (index, &value) in arguments.iter().enumerate() {
            let expected = if index < fixed {
                program.parameter_type(signature, index)?
            } else {
                tail[index - fixed]
            };
            retained.push(match (value, expected) {
                (boundary::Value::Scalar(mut value), boundary::Type::Scalar(ty))
                    if value.ty == ty && ty != Type::Void =>
                {
                    if matches!(value.operand, Operand::ObjectTemporary(_)) {
                        return Err("object temporary used as scalar call argument");
                    }
                    if index >= fixed && matches!(value.operand, Operand::Literal(_)) {
                        value = self.copy(value.ty, value)?;
                    }
                    BoundaryArgument::Scalar(value)
                }
                (boundary::Value::Object(value), boundary::Type::Object(expected))
                    if value.span == expected.span =>
                {
                    expected.validate(false)?;
                    super::super::validation::object_operand(program, &self.function, None, value)?;
                    BoundaryArgument::Object {
                        operand: value.operand,
                        span: self.span(value.span)?,
                        classes: (index >= fixed).then_some(expected.classes),
                    }
                }
                _ => return Err("object call argument disagrees with boundary type"),
            });
        }
        let objects = self.function.objects.get_or_insert_default();
        let start = program::index(objects.arguments.len())?;
        let end = program::index(
            objects
                .arguments
                .len()
                .checked_add(retained.len())
                .ok_or("object argument space exhausted")?,
        )?;
        objects.arguments.extend(retained);
        let operation = Operation::Call {
            signature,
            callee,
            arguments: CallArguments::Object(Range { start, end }),
        };
        match program.result_type(signature)? {
            boundary::Type::Scalar(Type::Void) => {
                self.emit(operation, None)?;
                Ok(boundary::ResultValue::Void)
            }
            boundary::Type::Scalar(ty) => {
                self.value(ty, operation).map(boundary::ResultValue::Scalar)
            }
            boundary::Type::Object(object) => {
                let span = self.span(object.span)?;
                let objects = self.function.objects.get_or_insert_default();
                let id = ObjectTempId(program::index(objects.temporaries.len())?);
                objects.temporaries.push(span);
                self.emit(operation, Some(ResultId::Object(id)))?;
                Ok(boundary::ResultValue::Object(boundary::ObjectValue {
                    operand: Operand::ObjectTemporary(id),
                    span: object.span,
                }))
            }
        }
    }
    /// Copy a direct object boundary value into ordinary proved destination storage.
    /// # Errors
    /// Rejects invalid source spans or non-pointer destinations.
    pub fn copy_object_value(
        &mut self,
        program: &Program,
        signature: SignatureId,
        source: boundary::ObjectValue,
        destination: program::Value,
    ) -> Result<(), &'static str> {
        if destination.ty != Type::Ptr {
            return Err("object value copy requires a pointer destination");
        }
        super::super::validation::object_operand(program, &self.function, Some(signature), source)?;
        let span = self.span(source.span)?;
        self.emit(
            Operation::CopyObject {
                span,
                source: source.operand,
                destination: destination.operand,
            },
            None,
        )
    }
    /// # Errors
    /// Rejects invalid spans and cross-kind object return operands.
    pub fn return_object(
        &mut self,
        program: &Program,
        signature: SignatureId,
        value: boundary::ObjectValue,
    ) -> Result<(), &'static str> {
        let terminator = self.object_return(program, signature, value)?;
        self.terminate(terminator)
    }
    pub(crate) fn object_return(
        &mut self,
        program: &Program,
        signature: SignatureId,
        value: boundary::ObjectValue,
    ) -> Result<program::Terminator, &'static str> {
        let boundary::Type::Object(result) = program.object_boundary(signature)?.result else {
            return Err("object return requires an object result boundary");
        };
        if result.span != value.span {
            return Err("object return span differs from boundary");
        }
        super::super::validation::object_operand(program, &self.function, Some(signature), value)?;
        let span = self.span(value.span)?;
        Ok(program::Terminator::ReturnObject {
            operand: value.operand,
            span,
        })
    }
}
