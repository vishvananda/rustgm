//! Physical C object signatures are derived once from the canonical target ABI plan.
use super::{Context, MemorySpan, Representation, SignatureId, Type, TypeId};
use crate::{abi::classification, lowir::boundary};
impl Context<'_> {
    pub(super) fn require_unadjusted_scalar(
        &self,
        convention: crate::types::callable::Convention,
        ty: TypeId,
        representation: Representation,
    ) -> Result<(), &'static str> {
        if convention.abi == crate::types::callable::Abi::Unadjusted
            && (!matches!(representation, Representation::Scalar(_))
                || self.non_scalar_boundary(ty)?)
        {
            return Err("unadjusted aggregate/SIMD ABI lowering is not implemented");
        }
        Ok(())
    }
    pub(super) fn non_scalar_boundary(&self, ty: TypeId) -> Result<bool, &'static str> {
        let head = self.types.storage_head(ty)?;
        Ok(head != TypeId::UNIT
            && matches!(
                self.types.kind(head),
                Some(
                    crate::types::Kind::Adt(..)
                        | crate::types::Kind::Tuple(_)
                        | crate::types::Kind::Array { .. }
                )
            ))
    }
    pub(super) fn object_signature(&mut self, ty: TypeId) -> Result<SignatureId, &'static str> {
        let plan = self.c_abi.get_or_insert_default().demand(
            self.types,
            self.aggregates,
            self.layouts,
            ty,
        )?;
        if plan.signature != ty {
            return Err("C ABI plan belongs to another signature");
        }
        let mut parameters = Vec::with_capacity(plan.parameters.len());
        for &parameter in &plan.parameters {
            let parameter = self.boundary_type(parameter)?;
            if parameter != boundary::Type::Scalar(Type::Void) {
                parameters.push(parameter);
            }
        }
        let result = self.boundary_type(plan.result)?;
        let Some(crate::types::Kind::Function { convention, .. }) = self.types.kind(ty) else {
            return Err("C ABI plan requires a function signature");
        };
        let noreturn = matches!(plan.result, classification::Passing::Scalar(TypeId::NEVER));
        let signature = if !matches!(result, boundary::Type::Object(_))
            && parameters
                .iter()
                .all(|parameter| matches!(parameter, boundary::Type::Scalar(_)))
        {
            let parameters = parameters
                .iter()
                .map(|parameter| match parameter {
                    boundary::Type::Scalar(ty) => Ok(*ty),
                    boundary::Type::Object(_) => Err("scalar C signature has an object parameter"),
                })
                .collect::<Result<Vec<_>, _>>()?;
            let boundary::Type::Scalar(result) = result else {
                return Err("scalar C signature has an object result");
            };
            self.output
                .signature_with_arity(&parameters, result, noreturn, convention.variadic)?
        } else {
            self.output.object_signature_with_arity(
                &parameters,
                result,
                noreturn,
                convention.variadic,
            )?
        };
        self.signatures.insert((ty, false), signature);
        Ok(signature)
    }
    pub(in crate::lower) fn c_argument(
        &mut self,
        ty: TypeId,
    ) -> Result<boundary::Type, &'static str> {
        let passing = self.c_abi.get_or_insert_default().argument(
            self.types,
            self.aggregates,
            self.layouts,
            ty,
        )?;
        self.boundary_type(passing)
    }
    pub(in crate::lower) fn indirect_argument(
        &mut self,
        ty: TypeId,
    ) -> Result<Option<MemorySpan>, &'static str> {
        if !self.aggregates.has_indirect_arguments() {
            return Ok(None);
        }
        Ok(self
            .c_abi
            .get_or_insert_default()
            .indirect_argument(self.types, self.aggregates, self.layouts, ty)?
            .map(|(bytes, align)| MemorySpan { bytes, align }))
    }
    pub(in crate::lower) fn indirect_parameters(
        &mut self,
        signature: TypeId,
    ) -> Result<bool, &'static str> {
        if !self.aggregates.has_indirect_arguments() {
            return Ok(false);
        }
        let Some(crate::types::Kind::Function {
            parameters,
            convention,
            ..
        }) = self.types.kind(signature)
        else {
            return Err("argument policy requires a callable signature");
        };
        if convention.abi != crate::types::callable::Abi::C {
            return Ok(false);
        }
        for &ty in self
            .types
            .list(parameters)
            .ok_or("missing C signature parameters")?
        {
            if self.indirect_argument(ty)?.is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn boundary_type(
        &mut self,
        value: classification::Passing,
    ) -> Result<boundary::Type, &'static str> {
        Ok(match value {
            classification::Passing::Indirect(_) => boundary::Type::Scalar(Type::Ptr),
            classification::Passing::Scalar(ty) => match self.representation(ty)? {
                Representation::Scalar(ty) => boundary::Type::Scalar(ty),
                Representation::ScalarObject(storage) => boundary::Type::Scalar(storage.ty),
                Representation::Object(_) => {
                    return Err("C scalar ABI plan requires an object representation");
                }
            },
            classification::Passing::Object(object) => boundary::Type::Object(boundary::Object {
                span: MemorySpan {
                    bytes: object.bytes,
                    align: object.align,
                },
                classes: match object.classes {
                    classification::Classes::Memory => boundary::Classes::Memory,
                    classification::Classes::Eightbytes { first, second } => {
                        boundary::Classes::Eightbytes {
                            first: class(first),
                            second: second.map(class),
                        }
                    }
                },
            }),
        })
    }
}
fn class(value: classification::Class) -> boundary::Class {
    match value {
        classification::Class::Integer => boundary::Class::Integer,
        classification::Class::Sse => boundary::Class::Sse,
        classification::Class::SseUp => boundary::Class::SseUp,
        classification::Class::None => boundary::Class::None,
    }
}
