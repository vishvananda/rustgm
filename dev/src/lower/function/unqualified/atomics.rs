//! Atomic accesses of one 8- to 64-bit integer or pointer word. Every access
//! is sequentially consistent, which satisfies each weaker requested ordering.
use super::{
    super::{Lower, Selected, word_access::Access},
    Call,
};
use crate::{
    lowir::{
        atomic::{CompareOrder, LoadOrder, RmwOrder, StoreOrder},
        program::Operand,
        vocabulary::Type,
    },
    support::word_operation::Update,
    types::TypeId,
};

const SEQUENTIAL: CompareOrder = CompareOrder::new(RmwOrder::Sequential, LoadOrder::Sequential);

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_atomic(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        let Some((&pointer, operands)) = call.inputs.split_first() else {
            return self.unqualified_fence(call);
        };
        let Some(word) = self.atomic_word(pointer)? else {
            return Ok(None);
        };
        match (call.name, operands) {
            ("atomic_load", []) if call.result == word.ty => {
                return self.atomic_access(word, Access::Load(LoadOrder::Sequential));
            }
            ("atomic_store", &[value]) if call.result == TypeId::UNIT => {
                return self.atomic_access(word, Access::Store(StoreOrder::Sequential, value));
            }
            ("atomic_load" | "atomic_store", _) => return Ok(None),
            _ => {}
        }
        let Some(operation) = Update::intrinsic(call.name) else {
            return Ok(None);
        };
        let paired = operation.compares();
        if operands.len() != if paired { 2 } else { 1 }
            || !self.arithmetic_result(call.result, word.ty, paired)
        {
            return Ok(None);
        }
        self.atomic_update(operation, word, operands, call.result, SEQUENTIAL)
    }
    /// A fence takes no word. A locked exchange with a private word is a full
    /// barrier, which is at least what any fence ordering asks for.
    fn unqualified_fence(&mut self, call: Call<'_>) -> Result<Option<Selected>, &'static str> {
        if call.name != "atomic_fence" || call.result != TypeId::UNIT {
            return Ok(None);
        }
        let scratch = Operand::Slot(self.builder.slot(Type::I64)?);
        let zero = self.builder.integer(Type::I64, 0)?;
        self.builder.store(zero, scratch)?;
        let address = self.builder.address(scratch)?;
        self.builder
            .atomic_exchange(zero, address.operand, RmwOrder::Sequential)?;
        Ok(Some(Selected::Unit))
    }
}
