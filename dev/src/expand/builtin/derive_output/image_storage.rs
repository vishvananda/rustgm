//! Prepay actual append capacity; lexical bytes are not token-slot counts.
use super::{Field, LexError, Output, Splice, Token, error};
use crate::{expand::stream::Origin, support::retained::Growth};

impl Output<'_> {
    pub(super) fn prepare_image(&mut self, bytes: usize) -> Result<(), LexError> {
        let additional = bytes.saturating_add(1);
        let mut growth = Growth::vector::<u8>(
            self.stream.source.len(),
            self.stream.source.capacity(),
            additional,
        );
        growth.include(Growth::vector::<Origin>(
            self.stream.origins.len(),
            self.stream.origins.capacity(),
            1,
        ));
        (self.charge)(
            bytes.saturating_add(growth.work).saturating_add(1),
            growth.bytes,
        )?;
        self.stream
            .source
            .try_reserve(additional)
            .map_err(|_| error(self.site.original.start, "derived image allocation failed"))?;
        self.stream
            .origins
            .try_reserve(1)
            .map_err(|_| error(self.site.original.start, "derived origin allocation failed"))?;
        Ok(())
    }
    pub(super) fn push_token(&mut self, token: Token) -> Result<(), LexError> {
        let growth =
            Growth::vector::<Token>(self.stream.tokens.len(), self.stream.tokens.capacity(), 1);
        (self.charge)(growth.work.saturating_add(1), growth.bytes)?;
        self.stream
            .tokens
            .try_reserve(1)
            .map_err(|_| error(self.site.original.start, "derived token allocation failed"))?;
        self.stream.tokens.push(token);
        Ok(())
    }
    pub(super) fn prepare_field(&mut self) -> Result<(), LexError> {
        let growth = Growth::vector::<Field>(self.fields.len(), self.fields.capacity(), 1);
        (self.charge)(growth.work.saturating_add(1), growth.bytes)?;
        self.fields
            .try_reserve(1)
            .map_err(|_| error(self.site.original.start, "derived field allocation failed"))
    }
    pub(super) fn prepare_parser(&self) -> Result<(), LexError> {
        // Parsing owns fresh syntax and temporary token/tree work. Copied typed
        // splices retain their own selected-subtree charges, not source-unit size.
        let token_bound = size_of::<crate::syntax::ast::Node>()
            + size_of::<crate::syntax::token_tree::Tree>()
            + 64;
        let splices = self
            .fields
            .len()
            .saturating_mul(4 * size_of::<Splice<'_>>());
        (self.charge)(
            self.stream.tokens.len().saturating_add(self.fields.len()),
            self.stream
                .tokens
                .len()
                .saturating_mul(token_bound)
                .saturating_mul(4)
                .saturating_add(splices),
        )
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/image_accounting.rs"]
mod tests;
