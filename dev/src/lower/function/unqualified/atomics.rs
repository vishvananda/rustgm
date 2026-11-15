//! Atomic accesses of one 8- to 64-bit integer or pointer word. Every access
//! is sequentially consistent, which satisfies each weaker requested ordering.
use super::{
    super::{Lower, Representation, Selected, casts},
    Call,
};
use crate::{
    body::ir,
    lowir::{
        atomic::{CompareOrder, LoadOrder, RmwOrder, StoreOrder},
        builder::Builder,
        program::{Operand, Terminator, Value},
        vocabulary::{Binary, Predicate, Type, Unary},
    },
    types::TypeId,
};

const EXCHANGE: CompareOrder = CompareOrder::new(RmwOrder::Sequential, LoadOrder::Sequential);

/// The accessed word: its type, its address and the integer that carries it.
#[derive(Clone, Copy)]
struct Word {
    ty: TypeId,
    address: Operand,
    carrier: Type,
    pointer: bool,
}
/// How a retried update derives the stored value from the observed one.
#[derive(Clone, Copy)]
enum Update {
    Bitwise {
        op: Binary,
        inverted: bool,
    },
    /// Keeps the observed value while the predicate holds against the operand.
    Extremum {
        keep: Predicate,
        signed: bool,
    },
}

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
        let (count, paired) = match call.name {
            "atomic_load" => (0, false),
            "atomic_cxchg" | "atomic_cxchgweak" => (2, true),
            _ => (1, false),
        };
        let update = update(call.name);
        let accepted = match call.name {
            "atomic_store" => call.result == TypeId::UNIT,
            "atomic_load" | "atomic_xchg" | "atomic_cxchg" | "atomic_cxchgweak" | "atomic_xadd"
            | "atomic_xsub" => self.arithmetic_result(call.result, word.ty, paired),
            _ => update.is_some() && call.result == word.ty,
        };
        if !accepted || operands.len() != count {
            return Ok(None);
        }
        let mut values = [None; 2];
        for (value, &operand) in values.iter_mut().zip(operands) {
            *value = self.atomic_operand(word, operand)?;
            if value.is_none() {
                return Ok(None);
            }
        }
        let builder = &mut self.builder;
        let previous = match (call.name, values, update) {
            ("atomic_load", ..) => {
                builder.atomic_load(word.carrier, word.address, LoadOrder::Sequential)?
            }
            ("atomic_store", [Some(value), _], _) => {
                builder.atomic_store(value, word.address, StoreOrder::Sequential)?;
                return Ok(Some(Selected::Unit));
            }
            ("atomic_xchg", [Some(value), _], _) => {
                builder.atomic_exchange(value, word.address, RmwOrder::Sequential)?
            }
            (_, [Some(expected), Some(replacement)], _) => {
                return self
                    .atomic_compare(call, word, expected, replacement)
                    .map(Some);
            }
            ("atomic_xadd", [Some(value), _], _) => {
                // The backend returns the sum; the intrinsic the word before it.
                let sum = builder.atomic_add_fetch(value, word.address, RmwOrder::Sequential)?;
                builder.binary(Binary::Sub, sum, value)?
            }
            ("atomic_xsub", [Some(value), _], _) => {
                let zero = builder.integer(word.carrier, 0)?;
                let negated = builder.binary(Binary::Sub, zero, value)?;
                let difference =
                    builder.atomic_add_fetch(negated, word.address, RmwOrder::Sequential)?;
                builder.binary(Binary::Add, difference, value)?
            }
            (_, [Some(value), _], Some(update)) => retried(builder, update, word, value)?,
            _ => return Ok(None),
        };
        self.atomic_value(word, previous)
            .map(|value| Some(Selected::Runtime(value)))
    }
    /// The word behind a thin pointer to an integer or pointer the backend's
    /// atomic operations cover.
    fn atomic_word(&mut self, pointer: ir::ValueId) -> Result<Option<Word>, &'static str> {
        let Some((address, ty)) = self.thin_input(pointer)? else {
            return Ok(None);
        };
        let (carrier, pointer) = match self.context.representation(ty)? {
            Representation::Scalar(Type::Ptr) => (Type::I64, true),
            Representation::Scalar(
                scalar @ (Type::I8
                | Type::U8
                | Type::I16
                | Type::U16
                | Type::I32
                | Type::U32
                | Type::I64),
            ) => (scalar, false),
            _ => return Ok(None),
        };
        Ok(Some(Word {
            ty,
            address: address.operand,
            carrier,
            pointer,
        }))
    }
    /// An operand in the word's carrier: the word's own type, or the machine
    /// word that updates a pointer.
    fn atomic_operand(
        &mut self,
        word: Word,
        operand: ir::ValueId,
    ) -> Result<Option<Value>, &'static str> {
        let Selected::Runtime(mut value) = self.value(operand)? else {
            return Ok(None);
        };
        if value.ty == Type::Ptr {
            value = self.builder.copy(Type::I64, value)?;
        }
        Ok((value.ty == word.carrier).then_some(value))
    }
    /// A carried word as a value of the word's own type.
    fn atomic_value(&mut self, word: Word, value: Value) -> Result<Value, &'static str> {
        if word.pointer {
            self.builder.copy(Type::Ptr, value)
        } else {
            Ok(value)
        }
    }
    /// The observed word and whether the replacement was stored. The backend
    /// leaves the expected word in the scratch on success and the observed
    /// one on failure; the strong exchange also serves the weak form.
    fn atomic_compare(
        &mut self,
        call: Call<'_>,
        word: Word,
        expected: Value,
        replacement: Value,
    ) -> Result<Selected, &'static str> {
        let scratch = Operand::Slot(self.builder.slot(word.carrier)?);
        self.builder.store(expected, scratch)?;
        let address = self.builder.address(scratch)?;
        let stored = self.builder.atomic_compare_exchange(
            replacement,
            word.address,
            address.operand,
            EXCHANGE,
        )?;
        let observed = self.builder.load(word.carrier, scratch)?;
        let observed = self.atomic_value(word, observed)?;
        self.store_subtraction_tuple(observed, stored, word.ty, call.result)
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

fn update(name: &str) -> Option<Update> {
    let bitwise = |op, inverted| Update::Bitwise { op, inverted };
    let extremum = |keep, signed| Update::Extremum { keep, signed };
    Some(match name {
        "atomic_and" => bitwise(Binary::And, false),
        "atomic_nand" => bitwise(Binary::And, true),
        "atomic_or" => bitwise(Binary::Or, false),
        "atomic_xor" => bitwise(Binary::Xor, false),
        "atomic_max" => extremum(Predicate::Ge, true),
        "atomic_min" => extremum(Predicate::Le, true),
        "atomic_umax" => extremum(Predicate::Uge, false),
        "atomic_umin" => extremum(Predicate::Ule, false),
        _ => return None,
    })
}

/// Exchange the word for its update until no other access intervened; the
/// result is the word the stored update was derived from.
fn retried(
    builder: &mut Builder,
    update: Update,
    word: Word,
    value: Value,
) -> Result<Value, &'static str> {
    let scratch = Operand::Slot(builder.slot(word.carrier)?);
    let seed = builder.atomic_load(word.carrier, word.address, LoadOrder::Sequential)?;
    builder.store(seed, scratch)?;
    let address = builder.address(scratch)?;
    let (retry, done) = (builder.block()?, builder.block()?);
    builder.terminate(Terminator::Jump(retry))?;
    builder.enter(retry)?;
    let observed = builder.load(word.carrier, scratch)?;
    let replacement = match update {
        Update::Bitwise { op, inverted } => {
            let combined = builder.binary(op, observed, value)?;
            if inverted {
                builder.unary(Unary::BitNot, combined)?
            } else {
                combined
            }
        }
        Update::Extremum { keep, signed } => {
            // Compared as full words, so narrow operands need not be canonical.
            let left = casts::integer(builder, observed, Type::I64, signed)?;
            let right = casts::integer(builder, value, Type::I64, signed)?;
            let kept = builder.compare(keep, left, right)?;
            let kept = casts::integer(builder, kept, word.carrier, false)?;
            let zero = builder.integer(word.carrier, 0)?;
            let mask = builder.binary(Binary::Sub, zero, kept)?;
            let differing = builder.binary(Binary::Xor, observed, value)?;
            let restored = builder.binary(Binary::And, differing, mask)?;
            builder.binary(Binary::Xor, value, restored)?
        }
    };
    // A failed exchange refreshes the scratch with the word it found.
    let stored =
        builder.atomic_compare_exchange(replacement, word.address, address.operand, EXCHANGE)?;
    builder.terminate(Terminator::Branch {
        condition: stored.operand,
        yes: done,
        no: retry,
    })?;
    builder.enter(done)?;
    builder.load(word.carrier, scratch)
}
