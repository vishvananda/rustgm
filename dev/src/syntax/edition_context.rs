//! Borrowed edition provenance shares the parser's existing input-context slot.
//! Ordinary tokens, cursor entries and syntax nodes acquire no new fields.
use super::interpolation::{EMPTY_INPUTS, Inputs, Splice};
use crate::{
    lex::token::{LexError, Token},
    support::edition::Edition,
};

impl Inputs for Edition {
    fn is_empty(&self) -> bool {
        true
    }
    fn get(&self, token: Token) -> Result<Option<Splice<'_>>, LexError> {
        EMPTY_INPUTS.get(token)
    }
    fn edition(&self, _token: Token) -> Result<Edition, LexError> {
        Ok(*self)
    }
}

/// Preserve the opaque-input provider while resolving each demanded token's
/// edition from its source provenance. Keep the mapping fixed for an invocation;
/// capture memoization belongs to that invocation, not to the defining macro.
pub struct WithEditions<'a, F> {
    inputs: &'a dyn Inputs,
    editions: F,
}
impl<'a, F> WithEditions<'a, F>
where
    F: Fn(Token) -> Result<Edition, LexError>,
{
    #[must_use]
    pub const fn new(inputs: &'a dyn Inputs, editions: F) -> Self {
        Self { inputs, editions }
    }
}
impl<F> Inputs for WithEditions<'_, F>
where
    F: Fn(Token) -> Result<Edition, LexError>,
{
    fn is_empty(&self) -> bool {
        self.inputs.is_empty()
    }
    fn get(&self, token: Token) -> Result<Option<Splice<'_>>, LexError> {
        self.inputs.get(token)
    }
    fn edition(&self, token: Token) -> Result<Edition, LexError> {
        (self.editions)(token)
    }
}
