//! Attribute capture already materializes meta/literal fragments into its forest.
use crate::{
    lex::token::{LexError, Token},
    resolve::{expansion::Report, unit::FileId},
    support::edition::Edition,
    syntax::interpolation::{self, Splice},
};
pub(super) struct Inputs<'a> {
    pub report: &'a Report,
    pub file: FileId,
}
impl interpolation::Inputs for Inputs<'_> {
    fn is_empty(&self) -> bool {
        true
    }
    fn get(&self, _: Token) -> Result<Option<Splice<'_>>, LexError> {
        Ok(None)
    }
    fn edition(&self, token: Token) -> Result<Edition, LexError> {
        crate::resolve::unit::editions::token(self.file, token, |file| {
            self.report.package.units.get(file.0 as usize)
        })
    }
}
