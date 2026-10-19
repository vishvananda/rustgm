//! Reconcile shared inputs together; validate cycles before publishing any result.
use super::{Constructor, Context, Error, Store, TermId, Value, arguments::Argument};
use std::collections::{HashMap, HashSet};

impl Context {
    /// Equate complete input pairs without rescanning every shared type prefix.
    /// Callable graphs use strict scoped equality, including local capture checks.
    /// Other graphs share structural matching and one final cycle check. Temporary
    /// cycles never escape a successful operation. As with ordinary inference, a
    /// failed operation may retain progress: the caller must discard the context.
    /// Lifetime arguments are outer equations, not evidence that they hold.
    /// # Errors
    /// Rejects mismatches, cycles, local capture, unresolved projection comparisons,
    /// invalid identities and exhausted resources.
    pub fn equate_input_pairs(
        &mut self,
        store: &Store,
        inputs: &[(Argument, Argument)],
    ) -> Result<(), Error> {
        self.account_work(inputs.len())?;
        let mut pairs = Vec::new();
        for &(left, right) in inputs {
            match (left, right) {
                (Argument::Type(a), Argument::Type(b)) => pairs.push((a, b)),
                (Argument::Const(a), Argument::Const(b)) => self.equate_consts(a, b)?,
                (Argument::Lifetime(a), Argument::Lifetime(b)) => {
                    if a != b {
                        self.region_equalities.push((a, b));
                    }
                }
                _ => return Err(Error::InvalidIdentity),
            }
        }
        if self.input_callables(store, &pairs)? {
            for (left, right) in pairs {
                self.equate_scoped_terms(store, left, right)?;
            }
            return Ok(());
        }
        self.pairs.clear();
        // A bulk memo must not leave later tiny equations clearing its capacity.
        // Keep ordinary scratch separate, including when this operation fails.
        let ordinary = std::mem::take(&mut self.seen);
        self.pairs.extend(pairs.iter().rev().copied());
        let result = self.unify_pending::<false>(store, true, None);
        self.seen = ordinary;
        result?;
        self.acyclic_inputs(&pairs)
    }

    fn input_callables(
        &mut self,
        store: &Store,
        pairs: &[(TermId, TermId)],
    ) -> Result<bool, Error> {
        let mut seen = HashSet::new();
        let mut work: Vec<_> = pairs.iter().flat_map(|&(a, b)| [a, b]).collect();
        while let Some(id) = work.pop() {
            self.step()?;
            let id = self.root(id)?;
            if !seen.insert(id) {
                continue;
            }
            let term = self.terms[id.0 as usize];
            match term.value {
                Value::Known(ty) => {
                    if store.contains_callables(ty).ok_or(Error::InvalidIdentity)? {
                        return Ok(true);
                    }
                }
                Value::Construct(Constructor::Function { .. }) => return Ok(true),
                Value::Construct(_) => {
                    self.account_work((term.end - term.start) as usize)?;
                    work.extend_from_slice(&self.edges[term.start as usize..term.end as usize]);
                }
                Value::Variable(_) => {}
            }
        }
        Ok(false)
    }

    fn acyclic_inputs(&mut self, pairs: &[(TermId, TermId)]) -> Result<(), Error> {
        // All edges are now stable. Share colors across roots, including aliases;
        // a gray revisit is a cycle and a black revisit is an already proved DAG.
        let mut active = HashMap::new();
        let mut work: Vec<_> = pairs
            .iter()
            .flat_map(|&(a, b)| [(a, false), (b, false)])
            .collect();
        while let Some((id, finish)) = work.pop() {
            self.step()?;
            let id = self.root(id)?;
            if finish {
                active.insert(id, false);
                continue;
            }
            match active.get(&id) {
                Some(true) => return Err(Error::Occurs),
                Some(false) => continue,
                None => {}
            }
            let term = self.terms[id.0 as usize];
            if !matches!(term.value, Value::Construct(_)) {
                active.insert(id, false);
                continue;
            }
            active.insert(id, true);
            self.account_work((term.end - term.start) as usize)?;
            work.push((id, true));
            work.extend(
                self.edges[term.start as usize..term.end as usize]
                    .iter()
                    .map(|&id| (id, false)),
            );
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Primitive, TypeId};

    #[test]
    fn bulk_pair_memos_do_not_make_later_small_queries_clear_bulk_capacity() -> Result<(), Error> {
        let store = Store::default();
        for fail in [false, true] {
            for count in [16, 256, 2048] {
                let mut context = Context::default();
                let byte = context.known(&store, TypeId::primitive(Primitive::U8))?;
                let first = context.construct(Constructor::Tuple, &[byte])?;
                let second = context.construct(Constructor::Tuple, &[byte])?;
                context.equate(&store, first, second)?;
                assert!(context.seen.capacity() <= 16);
                let mut left = first;
                let mut right = second;
                let mut inputs = Vec::new();
                for _ in 0..count {
                    left = context.construct(Constructor::Tuple, &[left])?;
                    right = context.construct(Constructor::Tuple, &[right])?;
                    inputs.push((Argument::Type(left), Argument::Type(right)));
                }
                if fail {
                    let boolean = context.known(&store, TypeId::primitive(Primitive::Bool))?;
                    inputs.push((Argument::Type(byte), Argument::Type(boolean)));
                }
                let result = context.equate_input_pairs(&store, &inputs);
                assert_eq!(result, if fail { Err(Error::Mismatch) } else { Ok(()) });
                println!(
                    "bulk_roots={count} failed={fail} retained_pair_capacity={}",
                    context.seen.capacity()
                );
                assert!(context.seen.capacity() <= 16);
                if !fail {
                    for _ in 0..count {
                        context.equate(&store, first, second)?;
                        assert!(context.seen.capacity() <= 16);
                    }
                }
            }
        }
        Ok(())
    }
}
