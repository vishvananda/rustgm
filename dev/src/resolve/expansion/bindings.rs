//! Expansion ancestry constrains which textual shadows are unambiguous.
use super::{Definition, Invocation};
use crate::{
    expand::{
        marks::ExpansionId,
        textual::{Lookup, Scope, Scopes},
    },
    lex::token::LexError,
    support::names::NameId,
    syntax::token_tree::error,
};

#[derive(Clone, Copy)]
pub(super) struct Bindings<'a> {
    pub definitions: &'a [Definition],
    pub invocations: &'a [Invocation],
}
impl Bindings<'_> {
    pub fn ancestor(&self, ancestor: ExpansionId, mut context: ExpansionId) -> bool {
        while context != ancestor && context.0 != 0 {
            let Some(call) = self.invocations.get(context.0 as usize - 1) else {
                return false;
            };
            context = call.parent;
        }
        context == ancestor
    }
    pub fn textual(
        &self,
        scopes: &mut Scopes,
        scope: Scope,
        name: NameId,
        context: ExpansionId,
    ) -> Result<Lookup, LexError> {
        let found = scopes.lookup(scope, name)?;
        if let Lookup::Found(index) = found {
            let definition = &self.definitions[index];
            if !self.ancestor(definition.introduction, context) {
                match scopes.lookup(definition.preceding_scope, name)? {
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
