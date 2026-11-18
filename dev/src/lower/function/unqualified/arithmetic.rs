//! Exact integer division and scalar result checks for unqualified calls.
use super::{
    super::{Lower, Selected},
    Call,
};
use crate::{
    lex::token::Punctuation,
    types::{Kind, Primitive, TypeId},
};

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_arithmetic(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        let &[left, right] = call.inputs else {
            return Ok(None);
        };
        let (Some(left), Some(right)) = (self.integer_input(left)?, self.integer_input(right)?)
        else {
            return Ok(None);
        };
        if left.ty != right.ty || call.result != left.ty {
            return Ok(None);
        }
        // The remainder is zero by contract, so this is plain division.
        self.scalar_binary(Punctuation::Slash, left.value, right.value, left.signed)
            .map(Some)
    }
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
