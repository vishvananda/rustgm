//! Trait equality preserves local binder correlations; only outer regions escape.
mod binders;
mod rebase;
use super::{Constructor, Context, Error, Store, TermId, Value, arguments::Slot};
use binders::Binders;
use rebase::Rebasing;
use std::collections::HashSet;

enum Work {
    Pair(TermId, TermId),
    Exit,
}

impl Context {
    /// Compare independently normalized types while preserving callable binders.
    /// Projection inputs are never treated as injective result constructors.
    /// Outer region equations still require the caller's lifetime proof.
    /// # Errors
    /// Rejects mismatched types, unresolved projections, capture and exhausted work.
    pub fn equate_normal_forms(
        &mut self,
        store: &Store,
        left: super::TypeId,
        right: super::TypeId,
    ) -> Result<(), Error> {
        let left = self.known(store, left)?;
        let right = self.known(store, right)?;
        self.equate_scoped_terms(store, left, right)
    }

    /// Compare inferred normal forms without treating projections as injective.
    /// Callable-local lifetimes remain rigid and outer equations remain obligations.
    /// # Errors
    /// Rejects mismatch, unresolved projections, local capture and exhausted work.
    pub fn equate_scoped_terms(
        &mut self,
        store: &Store,
        left: TermId,
        right: TermId,
    ) -> Result<(), Error> {
        self.equate_scoped_inside(store, left, right, &[])
    }
    pub(super) fn equate_scoped_inside(
        &mut self,
        store: &Store,
        left: TermId,
        right: TermId,
        ambient: &[u32],
    ) -> Result<(), Error> {
        self.join_slot_classes(left, right)?;
        let left = self.root(left)?;
        let right = self.root(right)?;
        if left == right {
            return Ok(());
        }
        for term in [right, left] {
            if let Value::Known(ty) = self.terms[term.0 as usize].value
                && ambient.is_empty()
                && !store.contains_callables(ty).ok_or(Error::InvalidIdentity)?
            {
                return self.equate(store, left, right);
            }
        }
        let mut work = vec![Work::Pair(left, right)];
        let mut binders = Binders::default();
        self.account_work(ambient.len())?;
        for &count in ambient {
            binders.enter_rigid(count)?;
        }
        let mut seen = HashSet::new();
        let mut rebasing = Rebasing::default();
        while let Some(next) = work.pop() {
            self.step()?;
            let Work::Pair(a, b) = next else {
                binders.exit()?;
                continue;
            };
            if binders.is_empty() {
                self.join_slot_classes(a, b)?;
            }
            let a = self.scoped_root(store, a, &binders, false, &mut rebasing)?;
            let b = self.scoped_root(store, b, &binders, true, &mut rebasing)?;
            if !seen.insert((a, b, binders.scope())) {
                continue;
            }
            if a == b
                && (binders.is_empty()
                    || matches!(self.terms[a.0 as usize].value, Value::Variable(_))
                    || self.scoped_closed(store, a)?)
            {
                continue;
            }
            if let (Value::Variable(left), Value::Variable(right)) = (
                self.terms[a.0 as usize].value,
                self.terms[b.0 as usize].value,
            ) {
                self.variable_pair(a, b, left, right)?;
                continue;
            }
            if let Value::Variable(domain) = self.terms[a.0 as usize].value {
                let candidate = rebasing.apply(self, store, b, binders.depth()?, false)?;
                self.bind(store, a, candidate, domain)?;
                continue;
            }
            if let Value::Variable(domain) = self.terms[b.0 as usize].value {
                let candidate = rebasing.apply(self, store, a, binders.depth()?, false)?;
                self.bind(store, b, candidate, domain)?;
                continue;
            }
            let left = self.materialize(store, a)?;
            let right = self.materialize(store, b)?;
            self.pairs.clear();
            self.structural(store, a, b, false)?;
            let entered = self.scoped_binder((a, b), left, right, &mut binders)?;
            self.scoped_regions(store, left, right, &mut binders)?;
            if entered {
                work.push(Work::Exit);
            }
            self.account_work(self.pairs.len())?;
            work.extend(self.pairs.drain(..).rev().map(|(a, b)| Work::Pair(a, b)));
        }
        Ok(())
    }

    fn scoped_binder(
        &mut self,
        roots: (TermId, TermId),
        left: Constructor,
        right: Constructor,
        binders: &mut Binders,
    ) -> Result<bool, Error> {
        let (a, b) = match (left, right) {
            (
                Constructor::Function {
                    bound_regions: a, ..
                },
                Constructor::Function {
                    bound_regions: b, ..
                },
            ) => (a, b),
            (
                Constructor::Dynamic { lifetimes: a, .. },
                Constructor::Dynamic { lifetimes: b, .. },
            ) => {
                let a = self.object_lifetimes(a)?;
                let b = self.object_lifetimes(b)?;
                binders.equate(a.region, b.region, &mut self.region_equalities)?;
                (a.bound_regions, b.bound_regions)
            }
            _ => return Ok(false),
        };
        if matches!(left, Constructor::Function { .. }) {
            binders.enter_owned(
                a,
                b,
                (self.signature_frame(roots.0), self.signature_frame(roots.1)),
            )?;
        } else {
            binders.enter(a, b)?;
        }
        Ok(true)
    }

    pub(super) fn equate_regions_inside(
        &mut self,
        left: super::RegionId,
        right: super::RegionId,
        ambient: &[u32],
    ) -> Result<(), Error> {
        if ambient.is_empty() {
            if left != right {
                self.region_equalities.push((left, right));
            }
            return Ok(());
        }
        let mut binders = Binders::default();
        self.account_work(ambient.len().saturating_add(1))?;
        for &count in ambient {
            binders.enter_rigid(count)?;
        }
        binders.equate(left, right, &mut self.region_equalities)
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
        binders: &Binders,
        right: bool,
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
        let depth = binders.depth()?;
        if variable && root != term && depth != 0 {
            let amount = if let Some((owner, local_only)) = self.signature_definition(term, root)?
                && owner.closed
            {
                if local_only {
                    let defined = binders.owner_depth(owner, right)?.ok_or(Error::Mismatch)?;
                    depth.checked_sub(defined).ok_or(Error::InvalidIdentity)?
                } else {
                    if self.contains_escaping_regions(store, root)? {
                        return Err(Error::Mismatch);
                    }
                    depth
                }
            } else {
                depth
            };
            rebasing.apply(self, store, root, amount, true)
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
            Constructor::AdtArgs(_, args)
            | Constructor::Opaque(_, args)
            | Constructor::Dynamic {
                arguments: args, ..
            }
            | Constructor::FunctionItemArgs(_, args) => Some(args),
            _ => None,
        })
    }
}
