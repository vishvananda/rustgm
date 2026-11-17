//! A host initializer is a checked pointer to an actual C callable.
use crate::{
    sema::values::{Constant, View},
    types::{Kind, Primitive, Store, TypeId, callable::Convention},
};

use super::Context;

impl Context<'_> {
    pub(super) fn validate_init_array(
        &self,
        ty: TypeId,
        value: Constant,
    ) -> Result<(), &'static str> {
        let View::Function(instance) = self.constants.view(value)? else {
            return Err("GNU init-array static requires an actual function address");
        };
        let Some(Kind::FunctionItem { signature, .. }) = self.types.kind(instance.ty()) else {
            return Err("GNU init-array address lost its actual selected function");
        };
        if !init_signature(self.types, signature) || !init_signature(self.types, ty) {
            return Err("GNU init-array requires a C void() or glibc three-argument signature");
        }
        // A function-pointer cast must not let a differently shaped target
        // receive the argument sequence glibc passes to the selected slot.
        let arguments = |sig| match self.types.kind(sig) {
            Some(Kind::Function { parameters, .. }) => self.types.list(parameters),
            _ => None,
        };
        if arguments(signature) != arguments(ty) {
            return Err("GNU init-array address and declared callable arguments disagree");
        }
        Ok(())
    }
}

fn init_signature(types: &Store, ty: TypeId) -> bool {
    let Some(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        convention: Convention::C,
        ..
    }) = types.kind(ty)
    else {
        return false;
    };
    match types.list(parameters) {
        Some([]) => true,
        Some([count, arguments, environment]) => {
            *count == TypeId::primitive(Primitive::I32)
                && c_string_vector(types, *arguments)
                && c_string_vector(types, *environment)
        }
        _ => false,
    }
}

fn c_string_vector(types: &Store, ty: TypeId) -> bool {
    let Some(Kind::Pointer {
        pointee,
        mutable: false,
    }) = types.kind(ty)
    else {
        return false;
    };
    matches!(types.kind(pointee), Some(Kind::Pointer { pointee, mutable: false })
        if pointee == TypeId::primitive(Primitive::U8))
}
