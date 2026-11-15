//! A reached type identity builds or compares the two address words of the actual source call.
use super::{Constant, Failure, Frame, Machine, code::Code, executed_calls::Answer};
use crate::{
    sema::{
        compiler_items::identity_words::{Operation, Selection},
        model::Scalar,
        values::{Reader, View},
    },
    types::instance::Instance,
};
impl Machine<'_, '_> {
    pub(super) fn source_type_identity(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = self.evaluator.checked_type_identity(target, body.source)? else {
            return Ok(None);
        };
        self.charge(64)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|current| !std::ptr::eq(current, body))
        {
            return Err("source type identity belongs to another reached caller".into());
        }
        let arguments = super::reached_layout::checked_call_arguments_with_signature(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            (selected.definition(), selected.output()),
            |actual, public| {
                public == selected.signature()
                    && selected.opened_matches(self.evaluator.facts(), actual)
            },
        )?;
        self.evaluator
            .facts()
            .source_bindings
            .as_deref()
            .ok_or("source type identity catalog disappeared")?
            .check_const_call(selected.definition())?;
        let inputs = body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source type identity lost its evaluated arguments")?;
        let value = match (selected.operation(), inputs) {
            (Operation::Identity(subject), []) => {
                let (words, spelled) = Selection::identity(self.evaluator.facts(), subject)?;
                self.charge(spelled)?;
                let words = self
                    .evaluator
                    .fields(&words.map(Constant::raw_address), None)?;
                self.evaluator.fields(&[words], None)?
            }
            (Operation::Equality, &[left, right]) => {
                let left = self.identity_words(Self::value(frame, left)?)?;
                let right = self.identity_words(Self::value(frame, right)?)?;
                Constant::scalar(Scalar::Boolean(left == right))
            }
            _ => return Err("source type identity changed its actual arguments".into()),
        };
        self.work.calls += 1;
        Ok(Some(Answer::Ready(value)))
    }
    /// The two words of an identity value as plain numbers. A word with
    /// provenance, and any value of another shape, has no comparison here.
    fn identity_words(&self, identity: Constant) -> Result<[u64; 2], Failure> {
        let words = self.evaluator.field(identity, 0)?;
        let mut halves = [0; 2];
        for (index, half) in (0..).zip(&mut halves) {
            let word = self.evaluator.field(words, index)?;
            let View::Address { bits } = Reader::new(self.evaluator.facts()).view(word)? else {
                return Err("type identity comparison requires two evaluated identities".into());
            };
            *half = bits;
        }
        Ok(halves)
    }
}
