//! Persistent textual scope links stop lookup at unresolved expansion output.
use super::marks::ItemContext;
use crate::support::hash::HashMap;
use crate::{lex::token::LexError, support::names::NameId, syntax::token_tree::error};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Scope(pub u32);
impl Scope {
    pub const ROOT: Self = Self(0);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lookup {
    Found(usize),
    Missing,
    Pending(Scope),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observer {
    Invocation(usize),
    Import(usize),
}
enum Entry {
    Root,
    Definition {
        name: NameId,
        definition: usize,
        previous: Scope,
    },
    Output {
        resolved: Option<Scope>,
    },
}
/// Physical and legacy-only sources retain the original entries and cache.
#[derive(Default)]
struct Contexts {
    definitions: HashMap<Scope, ItemContext>,
    cache: HashMap<(Scope, NameId, ItemContext), Lookup>,
}
pub struct Scopes {
    entries: Vec<Entry>,
    cache: HashMap<(Scope, NameId), Lookup>,
    waiting: HashMap<Scope, Vec<Observer>>,
    scratch: Vec<Scope>,
    contexts: Option<Box<Contexts>>,
    pub lookup_steps: usize,
}
impl Default for Scopes {
    fn default() -> Self {
        Self {
            entries: vec![Entry::Root],
            cache: HashMap::default(),
            waiting: HashMap::default(),
            scratch: Vec::new(),
            contexts: None,
            lookup_steps: 0,
        }
    }
}
impl Scopes {
    fn push(&mut self, entry: Entry) -> Result<Scope, LexError> {
        let id = Scope(
            u32::try_from(self.entries.len())
                .map_err(|_| error(0, "too many textual macro scopes"))?,
        );
        self.entries.push(entry);
        Ok(id)
    }
    /// # Errors
    /// Rejects invalid predecessors or scope-identity overflow.
    pub fn define(
        &mut self,
        previous: Scope,
        name: NameId,
        definition: usize,
    ) -> Result<Scope, LexError> {
        if previous.0 as usize >= self.entries.len() {
            return Err(error(0, "invalid textual scope predecessor"));
        }
        self.push(Entry::Definition {
            name,
            definition,
            previous,
        })
    }
    pub(crate) fn define_context(
        &mut self,
        previous: Scope,
        name: NameId,
        context: ItemContext,
        definition: usize,
    ) -> Result<Scope, LexError> {
        let scope = self.define(previous, name, definition)?;
        if context != ItemContext::ROOT {
            self.contexts
                .get_or_insert_with(Box::default)
                .definitions
                .insert(scope, context);
        }
        Ok(scope)
    }

    fn definition_context(&self, scope: Scope) -> ItemContext {
        self.contexts
            .as_ref()
            .and_then(|contexts| contexts.definitions.get(&scope))
            .copied()
            .unwrap_or(ItemContext::ROOT)
    }

    fn cached(&self, scope: Scope, name: NameId, context: ItemContext) -> Option<Lookup> {
        if context == ItemContext::ROOT {
            self.cache.get(&(scope, name)).copied()
        } else {
            self.contexts
                .as_ref()
                .and_then(|contexts| contexts.cache.get(&(scope, name, context)))
                .copied()
        }
    }
    /// # Errors
    /// Rejects scope-identity overflow.
    pub fn output(&mut self) -> Result<Scope, LexError> {
        self.push(Entry::Output { resolved: None })
    }
    /// # Errors
    /// Rejects missing identities, repeated publication or a self-cycle.
    pub fn complete(&mut self, output: Scope, head: Scope) -> Result<Vec<Observer>, LexError> {
        if head.0 as usize >= self.entries.len() || head == output {
            return Err(error(0, "invalid expansion output scope"));
        }
        let Some(Entry::Output { resolved }) = self.entries.get_mut(output.0 as usize) else {
            return Err(error(0, "textual scope is not an expansion output"));
        };
        if resolved.is_some() {
            return Err(error(0, "expansion output scope already completed"));
        }
        *resolved = Some(head);
        Ok(self.waiting.remove(&output).unwrap_or_default())
    }
    pub fn watch(&mut self, scope: Scope, request: Observer) {
        self.waiting.entry(scope).or_default().push(request);
    }
    /// Query immutable scope links. Pending cache entries retain the exact
    /// dependency, so completing a placeholder needs no global cache flush.
    ///
    /// # Errors
    /// Rejects malformed identities and cycles in caller-constructed graphs.
    pub fn lookup(&mut self, scope: Scope, name: NameId) -> Result<Lookup, LexError> {
        self.lookup_context(scope, name, ItemContext::ROOT)
    }

    pub(crate) fn lookup_context(
        &mut self,
        mut scope: Scope,
        name: NameId,
        context: ItemContext,
    ) -> Result<Lookup, LexError> {
        self.scratch.clear();
        let mut steps = 0;
        let result = loop {
            if steps > self.entries.len() {
                return Err(error(0, "cycle in textual macro scope"));
            }
            steps += 1;
            self.lookup_steps += 1;
            if let Some(found) = self.cached(scope, name, context) {
                if let Lookup::Pending(dependency) = found {
                    if let Some(Entry::Output { resolved: Some(_) }) =
                        self.entries.get(dependency.0 as usize)
                    {
                        self.scratch.push(scope);
                        scope = dependency;
                    } else {
                        break found;
                    }
                } else {
                    break found;
                }
            }
            self.scratch.push(scope);
            match self
                .entries
                .get(scope.0 as usize)
                .ok_or_else(|| error(0, "invalid textual macro scope"))?
            {
                Entry::Root => break Lookup::Missing,
                Entry::Definition {
                    name: bound,
                    definition,
                    previous,
                } => {
                    if *bound == name && self.definition_context(scope) == context {
                        break Lookup::Found(*definition);
                    }
                    scope = *previous;
                }
                Entry::Output {
                    resolved: Some(head),
                } => scope = *head,
                Entry::Output { resolved: None } => break Lookup::Pending(scope),
            }
        };
        if context == ItemContext::ROOT {
            for scope in self.scratch.drain(..) {
                self.cache.insert((scope, name), result);
            }
        } else {
            let contexts = self.contexts.get_or_insert_with(Box::default);
            for scope in self.scratch.drain(..) {
                contexts.cache.insert((scope, name, context), result);
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/textual_contexts.rs"]
mod tests;
