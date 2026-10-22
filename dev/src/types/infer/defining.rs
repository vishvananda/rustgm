//! Defining equations retain complete opaque capture identities and private roots.
use super::{Constructor, Context, Domain, Error, Store, TermId, TypeId, Value};
use crate::support::hash::HashMap;
use crate::types::{Kind, opaque::OpaqueId};

#[derive(Default)]
pub(super) struct Definitions {
    entries: HashMap<OpaqueId, Definition>,
    uses: HashMap<TermId, TermId>,
}
#[derive(Clone, Copy)]
struct Definition {
    application: TermId,
    hidden: TermId,
    used: bool,
}

impl Context {
    /// Authorize equations for one opaque result in its defining body only.
    /// The complete application remains the required type/const/lifetime context.
    /// # Errors
    /// Rejects invalid applications, repeated registration and exhausted storage.
    pub fn define_opaque(
        &mut self,
        store: &Store,
        application: TypeId,
        hidden: TermId,
    ) -> Result<(), Error> {
        let Some(Kind::Opaque(id, _)) = store.kind(application) else {
            return Err(Error::InvalidIdentity);
        };
        self.root(hidden)?;
        let application = self.known(store, application)?;
        let entries = &mut self.defining.get_or_insert_with(Default::default).entries;
        if entries
            .insert(
                id,
                Definition {
                    application,
                    hidden,
                    used: false,
                },
            )
            .is_some()
        {
            return Err(Error::InvalidIdentity);
        }
        Ok(())
    }

    pub(super) fn defining_root(
        &mut self,
        store: &Store,
        term: TermId,
        other: TermId,
    ) -> Result<TermId, Error> {
        if self.defining.is_none() {
            return Ok(term);
        }
        let id = match self.terms[term.0 as usize].value {
            Value::Known(ty) => match store.kind(ty) {
                Some(Kind::Opaque(id, _)) => id,
                _ => return Ok(term),
            },
            Value::Construct(Constructor::Opaque(id, _)) => id,
            _ => return Ok(term),
        };
        let Some(definition) = self
            .defining
            .as_ref()
            .and_then(|s| s.entries.get(&id))
            .copied()
        else {
            return Ok(term);
        };
        let hidden_root = self.root(definition.hidden)?;
        if matches!(self.terms[other.0 as usize].value, Value::Variable(_)) && other != hidden_root
        {
            return Ok(term);
        }
        let same_identity = match self.terms[other.0 as usize].value {
            Value::Known(ty) => {
                matches!(store.kind(ty), Some(Kind::Opaque(other, _)) if id == other)
            }
            Value::Construct(Constructor::Opaque(other, _)) => id == other,
            _ => false,
        };
        if same_identity {
            return Ok(term);
        }
        self.defining
            .as_mut()
            .and_then(|s| s.entries.get_mut(&id))
            .ok_or(Error::InvalidIdentity)?
            .used = true;
        let Definition {
            application,
            hidden,
            ..
        } = definition;
        let Constructor::Opaque(_, actual) = self.materialize(store, term)? else {
            return Err(Error::InvalidIdentity);
        };
        let Constructor::Opaque(_, expected) = self.materialize(store, application)? else {
            return Err(Error::InvalidIdentity);
        };
        // Capture equality is required even when the surrounding equation only
        // compares shape. It never reveals a different instantiation's output.
        self.equate_arguments(actual, expected, true)?;
        let actual = self.terms[term.0 as usize];
        let expected = self.terms[application.0 as usize];
        self.account_work((actual.end - actual.start) as usize)?;
        self.pairs.extend(
            self.edges[actual.start as usize..actual.end as usize]
                .iter()
                .copied()
                .zip(
                    self.edges[expected.start as usize..expected.end as usize]
                        .iter()
                        .copied(),
                ),
        );
        self.defining
            .as_mut()
            .ok_or(Error::InvalidIdentity)?
            .uses
            .insert(term, hidden);
        self.root(hidden)
    }

    pub(super) fn defining_types(&self, types: &[TypeId]) -> HashMap<TypeId, TypeId> {
        self.defining
            .iter()
            .flat_map(|d| &d.uses)
            .map(|(application, hidden)| (types[application.0 as usize], types[hidden.0 as usize]))
            .collect()
    }

    /// Pinned RPIT recursion defaults an otherwise unconstrained defining use to unit.
    /// Never-coercion and numeric fallbacks run separately; ordinary holes stay unknown.
    /// # Errors
    /// Rejects invalid registered roots and exhausted inference resources.
    pub fn default_defining_results(&mut self, store: &Store) -> Result<(), Error> {
        let Some(definitions) = &self.defining else {
            return Ok(());
        };
        let roots: Vec<_> = definitions
            .entries
            .values()
            .filter_map(|d| d.used.then_some(d.hidden))
            .collect();
        self.account_work(roots.len())?;
        for term in roots {
            let term = self.root(term)?;
            if matches!(
                self.terms[term.0 as usize].value,
                Value::Variable(Domain::Any)
            ) {
                let unit = self.known(store, TypeId::UNIT)?;
                self.bind(store, term, unit, Domain::Any)?;
            }
        }
        Ok(())
    }
}

impl Definitions {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.map(&self.entries);
        total.map(&self.uses);
        total
    }
}
