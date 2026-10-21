//! Suspended equations retain their roots and resume only changed associated-result leaves.
mod normalization;
mod occurrences;
mod predicate_inputs;
mod rewrite;
use super::{Constructor, Context, Error, Kind, Store, TermId, Value};
pub use predicate_inputs::ProjectionInputs;
use std::collections::{HashMap, VecDeque};

/// A body-local equation identity, retained until its normalization debt is discharged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EquationId(u32);

/// Lifetime equality and shape-only constraints have distinct proof obligations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    Equality,
    Shape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Complete,
    Pending(EquationId),
}

struct Equation {
    roots: (TermId, TermId),
    relation: Relation,
    remaining: usize,
    ready: Vec<usize>,
    queued: bool,
}

struct Leaf {
    equation: EquationId,
    roots: (TermId, TermId),
    pending: bool,
    queued: bool,
}

#[derive(Default)]
pub(super) struct Storage {
    pub(super) opaque: HashMap<TermId, super::TypeId>,
    equations: Vec<Equation>,
    leaves: Vec<Leaf>,
    waiting: HashMap<TermId, Vec<usize>>,
    ready: VecDeque<EquationId>,
    pending: usize,
}
impl Storage {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.map(&self.opaque);
        total.vector(&self.equations);
        total.vector(&self.leaves);
        for equation in &self.equations {
            total.vector(&equation.ready);
        }
        total.map(&self.waiting);
        for waiting in self.waiting.values() {
            total.vector(waiting);
        }
        total.deque(&self.ready);
        total
    }

    fn normalized(&mut self, term: TermId) -> usize {
        let Some(waiting) = self.waiting.remove(&term) else {
            return 0;
        };
        let count = waiting.len();
        for id in waiting {
            let leaf = &mut self.leaves[id];
            if !leaf.pending || leaf.queued {
                continue;
            }
            leaf.queued = true;
            let equation = &mut self.equations[leaf.equation.0 as usize];
            equation.ready.push(id);
            if !equation.queued {
                equation.queued = true;
                self.ready.push_back(leaf.equation);
            }
        }
        count
    }
}

impl Context {
    pub(super) fn opaque_projection(&self, term: TermId) -> Option<super::TypeId> {
        self.dependencies
            .as_ref()
            .and_then(|storage| storage.equations.opaque.get(&term))
            .copied()
    }

    pub(super) fn needs_normalization(&self, store: &Store, term: TermId) -> Result<bool, Error> {
        Ok(self.is_projection(store, term)? && self.opaque_projection(term).is_none())
    }

    pub(super) fn is_projection(&self, store: &Store, term: TermId) -> Result<bool, Error> {
        Ok(match self.terms[term.0 as usize].value {
            Value::Known(ty) => match store.kind(ty).ok_or(Error::InvalidIdentity)? {
                Kind::Projection(_) => true,
                Kind::Opaque(id, _) => store.opaque_declaration(id).is_some_and(|d| d.trait_result),
                _ => false,
            },
            Value::Construct(kind) => kind.normalizable(store),
            Value::Variable(_) => false,
        })
    }

    /// Distinguish complete-capture result queries from named associated applications.
    /// # Errors
    /// Rejects invalid inference identities and exhausted work.
    pub fn trait_result(&mut self, store: &Store, term: TermId) -> Result<bool, Error> {
        let term = self.root(term)?;
        Ok(match self.terms[term.0 as usize].value {
            Value::Known(ty) => {
                matches!(store.kind(ty), Some(Kind::Opaque(id, _)) if store.opaque_declaration(id).is_some_and(|d| d.trait_result))
            }
            Value::Construct(Constructor::Opaque(id, _)) => {
                store.opaque_declaration(id).is_some_and(|d| d.trait_result)
            }
            _ => false,
        })
    }

    /// Constrain independent structure, postponing associated-result comparisons.
    /// This does not select associated definitions or prove their requirements.
    /// Completion refuses any returned equation that has not subsequently resumed.
    /// Callable alpha equality remains a separate, strict inference operation.
    ///
    /// # Errors
    /// Rejects ordinary mismatches, invalid identities and exhausted inference work.
    pub fn equate_deferred(
        &mut self,
        store: &Store,
        left: TermId,
        right: TermId,
        relation: Relation,
    ) -> Result<Outcome, Error> {
        let mut blocked = Vec::new();
        self.unify(
            store,
            left,
            right,
            relation == Relation::Equality,
            Some(&mut blocked),
        )?;
        if blocked.is_empty() {
            return Ok(Outcome::Complete);
        }
        self.step()?;
        let storage = &mut self
            .dependencies
            .get_or_insert_with(Default::default)
            .equations;
        let id = EquationId(u32::try_from(storage.equations.len()).map_err(|_| Error::Overflow)?);
        storage.equations.push(Equation {
            roots: (left, right),
            relation,
            remaining: 0,
            ready: Vec::new(),
            queued: false,
        });
        storage.pending += 1;
        self.block_equation(store, id, blocked)?;
        Ok(Outcome::Pending(id))
    }

    fn block_equation(
        &mut self,
        store: &Store,
        id: EquationId,
        blocked: Vec<(TermId, TermId)>,
    ) -> Result<(), Error> {
        self.account_work(blocked.len())?;
        for (left, right) in blocked {
            let first = self.needs_normalization(store, left)?;
            let second = self.needs_normalization(store, right)?;
            let storage = &mut self
                .dependencies
                .as_mut()
                .ok_or(Error::InvalidIdentity)?
                .equations;
            let index = storage.leaves.len();
            storage.leaves.push(Leaf {
                equation: id,
                roots: (left, right),
                pending: true,
                queued: false,
            });
            storage.equations[id.0 as usize].remaining += 1;
            if first {
                storage.waiting.entry(left).or_default().push(index);
            }
            if second {
                storage.waiting.entry(right).or_default().push(index);
            }
        }
        Ok(())
    }

    /// Retry only leaves awakened by semantic normalization. Their relation kind
    /// is retained; already constrained ancestors and siblings are not replayed.
    ///
    /// # Errors
    /// Rejects completed/invalid tickets, ordinary mismatches and exhausted work.
    pub fn resume_equation(&mut self, store: &Store, id: EquationId) -> Result<Outcome, Error> {
        let storage = &mut self
            .dependencies
            .as_mut()
            .ok_or(Error::InvalidIdentity)?
            .equations;
        let equation = storage
            .equations
            .get_mut(id.0 as usize)
            .ok_or(Error::InvalidIdentity)?;
        if equation.remaining == 0 {
            return Err(Error::InvalidIdentity);
        }
        let relation = equation.relation;
        let ready = std::mem::take(&mut equation.ready);
        let mut blocked = Vec::new();
        for index in ready {
            self.step()?;
            let storage = &mut self
                .dependencies
                .as_mut()
                .ok_or(Error::InvalidIdentity)?
                .equations;
            let leaf = &mut storage.leaves[index];
            leaf.pending = false;
            let (left, right) = leaf.roots;
            storage.equations[id.0 as usize].remaining -= 1;
            self.unify(
                store,
                left,
                right,
                relation == Relation::Equality,
                Some(&mut blocked),
            )?;
        }
        self.block_equation(store, id, blocked)?;
        let storage = &mut self
            .dependencies
            .as_mut()
            .ok_or(Error::InvalidIdentity)?
            .equations;
        if storage.equations[id.0 as usize].remaining == 0 {
            storage.pending -= 1;
            Ok(Outcome::Complete)
        } else {
            Ok(Outcome::Pending(id))
        }
    }

    /// Original whole roots remain available for the caller's scoped proof work.
    /// # Errors
    /// Rejects an invalid equation identity.
    pub fn equation_roots(&self, id: EquationId) -> Result<(TermId, TermId), Error> {
        self.dependencies
            .as_ref()
            .and_then(|s| s.equations.equations.get(id.0 as usize))
            .map(|e| e.roots)
            .ok_or(Error::InvalidIdentity)
    }

    /// Receive equations whose blocked projections have actually normalized.
    /// # Errors
    /// Rejects exhausted work while discarding already-consumed queue entries.
    pub fn ready_equation(&mut self) -> Result<Option<EquationId>, Error> {
        while let Some(storage) = &mut self.dependencies {
            let Some(id) = storage.equations.ready.pop_front() else {
                return Ok(None);
            };
            let equation = &mut storage.equations.equations[id.0 as usize];
            equation.queued = false;
            let ready = !equation.ready.is_empty();
            self.step()?;
            if ready {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    #[must_use]
    pub fn has_deferred_equations(&self) -> bool {
        self.dependencies
            .as_ref()
            .is_some_and(|s| s.equations.pending != 0)
    }

    fn normalized_projection(&mut self, term: TermId) -> Result<(), Error> {
        let count = self
            .dependencies
            .as_mut()
            .map_or(0, |s| s.equations.normalized(term));
        self.account_work(count)
    }
}

impl Constructor {
    pub(super) fn normalizable(self, store: &Store) -> bool {
        matches!(self, Self::Projection(_))
            || matches!(self, Self::Opaque(id, _) if store.opaque_declaration(id).is_some_and(|d| d.trait_result))
    }
}
