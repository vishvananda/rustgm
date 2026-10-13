//! Exact text comparisons check length before reading bytes and emit a bounded-size loop.
use super::Lower;
use crate::{
    body::ir::Place,
    lowir::{
        program::{Operand, Terminator, Value},
        vocabulary::{Binary, Convert, Predicate, Type},
    },
    sema::text::Id,
};

impl Lower<'_, '_> {
    pub(super) fn text_equals(&mut self, place: Place, text: Id) -> Result<Value, &'static str> {
        let count = self.context.constants.bytes(text)?.len();
        let actual = self.slice_length(place)?;
        let count_value = self.builder.integer(Type::I64, count as u128)?;
        let same = self.builder.compare(Predicate::Eq, actual, count_value)?;
        if count == 0 {
            return self.builder.convert(Convert::Trunc, Type::I1, same);
        }
        let input = self
            .place(place)?
            .ok_or("text comparison lacks data storage")?;
        let input = self.pointer(input)?;
        let global = self.context.bytes_data(text)?;
        let expected = self.builder.address(Operand::Global(global))?;
        let position = Operand::Slot(self.builder.slot(Type::I64)?);
        let result = Operand::Slot(self.builder.slot(Type::I1)?);
        let zero = self.builder.integer(Type::I64, 0)?;
        let no = self.builder.integer(Type::I1, 0)?;
        self.builder.store(zero, position)?;
        self.builder.store(no, result)?;
        let compare = self.builder.block()?;
        let advance = self.builder.block()?;
        let success = self.builder.block()?;
        let done = self.builder.block()?;
        self.builder.terminate(Terminator::Branch {
            condition: same.operand,
            yes: compare,
            no: done,
        })?;
        self.builder.enter(compare)?;
        let index = self.builder.load(Type::I64, position)?;
        let a = self.builder.index(Type::U8, input, index)?;
        let b = self.builder.index(Type::U8, expected, index)?;
        let a = self.builder.load(Type::U8, a.operand)?;
        let b = self.builder.load(Type::U8, b.operand)?;
        let same = self.builder.compare(Predicate::Eq, a, b)?;
        self.builder.terminate(Terminator::Branch {
            condition: same.operand,
            yes: advance,
            no: done,
        })?;
        self.builder.enter(advance)?;
        let one = self.builder.integer(Type::I64, 1)?;
        let next = self.builder.binary(Binary::Add, index, one)?;
        self.builder.store(next, position)?;
        let finished = self.builder.compare(Predicate::Eq, next, count_value)?;
        self.builder.terminate(Terminator::Branch {
            condition: finished.operand,
            yes: success,
            no: compare,
        })?;
        self.builder.enter(success)?;
        let yes = self.builder.integer(Type::I1, 1)?;
        self.builder.store(yes, result)?;
        self.builder.terminate(Terminator::Jump(done))?;
        self.builder.enter(done)?;
        self.builder.load(Type::I1, result)
    }
}
