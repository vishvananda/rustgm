//! Checked and bring-up calls share one emission path for native atomic updates.
use super::{Lower, Selected, casts, ir, word_access::Word};
use crate::{
    lowir::{
        atomic::{CompareOrder, LoadOrder, RmwOrder},
        builder::Builder,
        program::{Operand, Terminator, Value},
        vocabulary::{Binary, Predicate, Type, Unary},
    },
    sema::compiler_items::update_contract,
    support::word_operation::Update,
    types::TypeId,
};

impl Lower<'_, '_, '_> {
    pub(super) fn source_atomic_update(
        &mut self,
        operation: Update,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Option<Selected>, &'static str> {
        let Some(selection) =
            update_contract::select(self.context.ledger, operation, callee, signature)?
        else {
            return Ok(None);
        };
        let Some((&pointer, values)) = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .and_then(<[_]>::split_first)
        else {
            return Err("source atomic update lost its evaluated Call inputs");
        };
        if self.input_type(pointer)? != selection.pointer
            || result != selection.result
            || values.len() != if operation.compares() { 2 } else { 1 }
            || values
                .iter()
                .any(|&value| self.input_type(value) != Ok(selection.operand))
        {
            return Err("source atomic update differs from its evaluated Call");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let word = self
            .atomic_word(pointer)?
            .ok_or("source atomic update has no native word")?;
        let ordering = CompareOrder::new(
            RmwOrder::from_rust(selection.success),
            LoadOrder::from_rust(selection.failure)
                .ok_or("source atomic update lost its failure ordering")?,
        );
        self.atomic_update(operation, word, values, result, ordering)?
            .map(Some)
            .ok_or("source atomic update has no native operand")
    }

    pub(super) fn atomic_update(
        &mut self,
        operation: Update,
        word: Word,
        operands: &[ir::ValueId],
        result: TypeId,
        ordering: CompareOrder,
    ) -> Result<Option<Selected>, &'static str> {
        let mut values = [None; 2];
        for (value, &operand) in values.iter_mut().zip(operands) {
            *value = self.atomic_operand(word, operand)?;
            if value.is_none() {
                return Ok(None);
            }
        }
        let builder = &mut self.builder;
        let previous = match (operation, values) {
            (Update::Exchange, [Some(value), None]) => {
                builder.atomic_exchange(value, word.address, ordering.success())?
            }
            (Update::CompareExchange, [Some(expected), Some(replacement)]) => {
                return self
                    .atomic_compare(word, expected, replacement, result, ordering)
                    .map(Some);
            }
            (Update::Add, [Some(value), None]) => {
                // The backend returns the sum; Rust returns the previous word.
                let sum = builder.atomic_add_fetch(value, word.address, ordering.success())?;
                builder.binary(Binary::Sub, sum, value)?
            }
            (Update::Subtract, [Some(value), None]) => {
                let zero = builder.integer(word.carrier, 0)?;
                let negated = builder.binary(Binary::Sub, zero, value)?;
                let difference =
                    builder.atomic_add_fetch(negated, word.address, ordering.success())?;
                builder.binary(Binary::Add, difference, value)?
            }
            (operation, [Some(value), None]) => {
                let Some(update) = retry(operation) else {
                    return Ok(None);
                };
                retried(builder, update, word, value, ordering)?
            }
            _ => return Ok(None),
        };
        self.atomic_value(word, previous)
            .map(|value| Some(Selected::Runtime(value)))
    }

    /// Failed exchanges update the expected scratch with the observed word.
    /// A strong exchange also implements the permitted weak behavior.
    fn atomic_compare(
        &mut self,
        word: Word,
        expected: Value,
        replacement: Value,
        result: TypeId,
        ordering: CompareOrder,
    ) -> Result<Selected, &'static str> {
        let scratch = Operand::Slot(self.builder.slot(word.carrier)?);
        self.builder.store(expected, scratch)?;
        let address = self.builder.address(scratch)?;
        let stored = self.builder.atomic_compare_exchange(
            replacement,
            word.address,
            address.operand,
            ordering,
        )?;
        let observed = self.builder.load(word.carrier, scratch)?;
        let observed = self.atomic_value(word, observed)?;
        self.store_subtraction_tuple(observed, stored, word.ty, result)
    }
}

/// How a retried update derives the stored value from the observed one.
#[derive(Clone, Copy)]
enum Retry {
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

fn retry(operation: Update) -> Option<Retry> {
    let bitwise = |op, inverted| Retry::Bitwise { op, inverted };
    let extremum = |keep, signed| Retry::Extremum { keep, signed };
    Some(match operation {
        Update::And => bitwise(Binary::And, false),
        Update::Nand => bitwise(Binary::And, true),
        Update::Or => bitwise(Binary::Or, false),
        Update::Xor => bitwise(Binary::Xor, false),
        Update::Max { signed: true } => extremum(Predicate::Ge, true),
        Update::Min { signed: true } => extremum(Predicate::Le, true),
        Update::Max { signed: false } => extremum(Predicate::Uge, false),
        Update::Min { signed: false } => extremum(Predicate::Ule, false),
        _ => return None,
    })
}

/// Exchange the word for its update until no other access intervened; the
/// result is the word the stored update was derived from.
fn retried(
    builder: &mut Builder,
    update: Retry,
    word: Word,
    value: Value,
    ordering: CompareOrder,
) -> Result<Value, &'static str> {
    let scratch = Operand::Slot(builder.slot(word.carrier)?);
    let seed = builder.atomic_load(word.carrier, word.address, ordering.failure())?;
    builder.store(seed, scratch)?;
    let address = builder.address(scratch)?;
    let (retry, done) = (builder.block()?, builder.block()?);
    builder.terminate(Terminator::Jump(retry))?;
    builder.enter(retry)?;
    let observed = builder.load(word.carrier, scratch)?;
    let replacement = match update {
        Retry::Bitwise { op, inverted } => {
            let combined = builder.binary(op, observed, value)?;
            if inverted {
                builder.unary(Unary::BitNot, combined)?
            } else {
                combined
            }
        }
        Retry::Extremum { keep, signed } => {
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
        builder.atomic_compare_exchange(replacement, word.address, address.operand, ordering)?;
    builder.terminate(Terminator::Branch {
        condition: stored.operand,
        yes: done,
        no: retry,
    })?;
    builder.enter(done)?;
    builder.load(word.carrier, scratch)
}
