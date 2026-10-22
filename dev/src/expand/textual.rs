//! Persistent textual scope links stop lookup at unresolved expansion output.
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
pub struct Scopes {
    entries: Vec<Entry>,
    cache: HashMap<(Scope, NameId), Lookup>,
    waiting: HashMap<Scope, Vec<Observer>>,
    scratch: Vec<Scope>,
    pub lookup_steps: usize,
}
impl Default for Scopes {
    fn default() -> Self {
        Self {
            entries: vec![Entry::Root],
            cache: HashMap::default(),
            waiting: HashMap::default(),
            scratch: Vec::new(),
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
    pub fn lookup(&mut self, mut scope: Scope, name: NameId) -> Result<Lookup, LexError> {
        self.scratch.clear();
        let mut steps = 0;
        let result = loop {
            if steps > self.entries.len() {
                return Err(error(0, "cycle in textual macro scope"));
            }
            steps += 1;
            self.lookup_steps += 1;
            if let Some(found) = self.cache.get(&(scope, name)).copied() {
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
                    if *bound == name {
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
        for scope in self.scratch.drain(..) {
            self.cache.insert((scope, name), result);
        }
        Ok(result)
    }
}
