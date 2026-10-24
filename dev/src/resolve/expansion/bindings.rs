//! Expansion ancestry constrains which textual shadows are unambiguous.
use super::Definition;
use crate::{
    expand::{
        ancestry::Ancestry,
        marks::{ExpansionId, ItemContext},
        textual::{Lookup, Scope, Scopes},
    },
    lex::token::LexError,
    support::names::NameId,
    syntax::token_tree::error,
};

#[derive(Clone, Copy)]
pub(super) struct Bindings<'a> {
    pub definitions: &'a [Definition],
}
impl Bindings<'_> {
    pub fn textual(
        &self,
        scopes: &mut Scopes,
        ancestry: &Ancestry,
        scope: Scope,
        name: (NameId, ItemContext),
        context: ExpansionId,
    ) -> Result<Lookup, LexError> {
        let found = scopes.lookup_context(scope, name.0, name.1)?;
        if let Lookup::Found(index) = found {
            let definition = &self.definitions[index];
            if !ancestry.ancestor(definition.introduction, context) {
                match scopes.lookup_context(definition.preceding_scope, name.0, name.1)? {
                    Lookup::Found(_) => {
                        return Err(error(0, "ambiguous macro-expanded textual name"));
                    }
                    Lookup::Pending(scope) => return Ok(Lookup::Pending(scope)),
                    Lookup::Missing => {}
                }
            }
        }
        Ok(found)
    }
}
