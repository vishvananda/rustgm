//! Expansion ancestry constrains which textual shadows are unambiguous.
use super::Definition;
use crate::expand::builtin::BuiltinKind;
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
        self.textual_selected(scopes, ancestry, scope, name, context, (None, &|_| Ok(())))
    }
    pub fn textual_kind(
        &self,
        scopes: &mut Scopes,
        ancestry: &Ancestry,
        scope: Scope,
        name: (NameId, ItemContext),
        context: ExpansionId,
        filter: (BuiltinKind, &impl Fn(usize) -> Result<(), LexError>),
    ) -> Result<Lookup, LexError> {
        self.textual_selected(
            scopes,
            ancestry,
            scope,
            name,
            context,
            (Some(filter.0), filter.1),
        )
    }
    fn textual_selected(
        &self,
        scopes: &mut Scopes,
        ancestry: &Ancestry,
        scope: Scope,
        name: (NameId, ItemContext),
        context: ExpansionId,
        filter: (Option<BuiltinKind>, &impl Fn(usize) -> Result<(), LexError>),
    ) -> Result<Lookup, LexError> {
        let found = self.selected(scopes, scope, name, filter)?;

        if let Lookup::Found(index) = found {
            let definition = &self.definitions[index];
            if !ancestry.ancestor(definition.introduction, context) {
                match self.selected(scopes, definition.preceding_scope, name, filter)? {
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
    fn selected(
        &self,
        scopes: &mut Scopes,
        mut scope: Scope,
        name: (NameId, ItemContext),
        filter: (Option<BuiltinKind>, &impl Fn(usize) -> Result<(), LexError>),
    ) -> Result<Lookup, LexError> {
        let Some(kind) = filter.0 else {
            return scopes.lookup_context(scope, name.0, name.1);
        };
        // Cache entries remain unfiltered. A rejected candidate resumes at its
        // actual preceding scope without publishing a kind-dependent miss.
        for _ in 0..=self.definitions.len() {
            filter.1(1)?;
            let found = scopes.lookup_context(scope, name.0, name.1)?;
            let Lookup::Found(index) = found else {
                return Ok(found);
            };
            let definition = self
                .definitions
                .get(index)
                .ok_or_else(|| error(0, "unknown textual macro definition"))?;
            if definition.macro_kind() == kind {
                return Ok(found);
            }
            scope = definition.preceding_scope;
        }
        Err(error(0, "cycle in filtered textual macro scope"))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/subnamespace_queries.rs"]
mod tests;
