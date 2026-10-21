//! Physical C object signatures are derived once from the canonical target ABI plan.
use super::{Context, MemorySpan, Representation, SignatureId, Type, TypeId};
use crate::{abi::classification, lowir::boundary};
impl Context<'_> {
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
        let signature = self.output.object_signature(
            &parameters,
            result,
            matches!(plan.result, classification::Passing::Scalar(TypeId::NEVER)),
        )?;
        self.signatures.insert(ty, signature);
        Ok(signature)
    }
    fn boundary_type(
        &mut self,
        value: classification::Passing,
    ) -> Result<boundary::Type, &'static str> {
        Ok(match value {
            classification::Passing::Scalar(ty) => match self.representation(ty)? {
                Representation::Scalar(ty) => boundary::Type::Scalar(ty),
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
        classification::Class::None => boundary::Class::None,
    }
}
