//! Trait equality preserves local binder correlations; only outer regions escape.
mod binders;
mod rebase;
use super::{Constructor, Context, Error, Store, TermId, TypeId, Value, arguments::Slot};
use binders::Binders;
use rebase::Rebasing;
use std::collections::HashSet;

enum Work {
    Pair(TermId, TermId),
    Exit,
}

impl Context {
    pub(super) fn equate_scoped(
        &mut self,
        store: &Store,
        left: TermId,
        right: TypeId,
    ) -> Result<(), Error> {
        let known = self.known(store, right)?;
        if self.root(left)? == known {
            return Ok(());
        }
        if !store
            .contains_callables(right)
            .ok_or(Error::InvalidIdentity)?
        {
            return self.equate(store, left, known);
        }
        let mut work = vec![Work::Pair(left, known)];
        let mut binders = Binders::default();
        let mut seen = HashSet::new();
        let mut rebasing = Rebasing::default();
        while let Some(next) = work.pop() {
            self.step()?;
            let Work::Pair(a, b) = next else {
                binders.exit()?;
                continue;
            };
            let a = self.scoped_root(store, a, binders.depth()?, &mut rebasing)?;
            let b = self.root(b)?;
            if !seen.insert((a, b, binders.scope())) {
                continue;
            }
            if let Value::Variable(domain) = self.terms[a.0 as usize].value {
                let candidate = rebasing.apply(self, store, b, binders.depth()?, false)?;
                self.bind(store, a, candidate, domain)?;
                continue;
            }
            if a == b && (binders.is_empty() || self.scoped_closed(store, a)?) {
                continue;
            }
            let left = self.materialize(store, a)?;
            let right = self.materialize(store, b)?;
            self.pairs.clear();
            self.structural(store, a, b, false)?;
            self.scoped_regions(store, left, right, &mut binders)?;
            if let (
                Constructor::Function {
                    bound_regions: a, ..
                },
                Constructor::Function {
                    bound_regions: b, ..
                },
            ) = (left, right)
            {
                binders.enter(a, b)?;
                work.push(Work::Exit);
            }
            self.account_work(self.pairs.len())?;
            work.extend(self.pairs.drain(..).rev().map(|(a, b)| Work::Pair(a, b)));
        }
        Ok(())
    }

    /// A closed canonical subtree cannot mention the surrounding alpha mapping.
    fn scoped_closed(&self, store: &Store, term: TermId) -> Result<bool, Error> {
        Ok(match self.terms[term.0 as usize].value {
            Value::Known(ty) => store.escaping_binders(ty).ok_or(Error::InvalidIdentity)? == 0,
            Value::Variable(_) | Value::Construct(_) => false,
        })
    }

    fn scoped_root(
        &mut self,
        store: &Store,
        term: TermId,
        depth: u16,
        rebasing: &mut Rebasing,
    ) -> Result<TermId, Error> {
        let variable = matches!(
            self.terms
                .get(term.0 as usize)
                .ok_or(Error::InvalidIdentity)?
                .value,
            Value::Variable(_)
        );
        let root = self.root(term)?;
        if variable && root != term && depth != 0 {
            rebasing.apply(self, store, root, depth, true)
        } else {
            Ok(root)
        }
    }

    fn scoped_regions(
        &mut self,
        store: &Store,
        left: Constructor,
        right: Constructor,
        binders: &mut Binders,
    ) -> Result<(), Error> {
        if let (
            Constructor::Reference { region: a, .. },
            Constructor::Reference { region: b, .. },
        ) = (left, right)
        {
            binders.equate(a, b, &mut self.region_equalities)?;
        }
        let Some(a) = self.scoped_arguments(store, left)? else {
            return Ok(());
        };
        let b = self
            .scoped_arguments(store, right)?
            .ok_or(Error::InvalidIdentity)?;
        let count = self.slots(a)?.len();
        self.account_work(count)?;
        for index in 0..count {
            if let (Slot::Lifetime(a), Slot::Lifetime(b)) =
                (self.slots(a)?[index], self.slots(b)?[index])
            {
                binders.equate(a, b, &mut self.region_equalities)?;
            }
        }
        Ok(())
    }

    fn scoped_arguments(
        &mut self,
        store: &Store,
        kind: Constructor,
    ) -> Result<Option<super::arguments::ArgumentsId>, Error> {
        Ok(match kind {
            Constructor::Adt(_, args) | Constructor::FunctionItem(_, args) => {
                Some(self.materialize_arguments(store, args, &mut Vec::new())?)
            }
            Constructor::AdtArgs(_, args) | Constructor::FunctionItemArgs(_, args) => Some(args),
            _ => None,
        })
    }
}
