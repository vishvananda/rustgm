//! Scalar and paired result types for unqualified atomic calls.
use super::super::Lower;
use crate::types::{Kind, Primitive, TypeId};

impl Lower<'_, '_, '_> {
    /// The operand type itself, or its pair with the overflow flag.
    pub(super) fn arithmetic_result(&self, result: TypeId, operand: TypeId, paired: bool) -> bool {
        if !paired {
            return result == operand;
        }
        let Some(Kind::Tuple(fields)) = self.context.types.kind(result) else {
            return false;
        };
        self.context
            .types
            .list(fields)
            .is_some_and(|fields| fields == [operand, TypeId::primitive(Primitive::Bool)])
    }
}
