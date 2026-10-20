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
        let pairs: Vec<_> = inputs
            .iter()
            .filter_map(|&(left, right)| {
                if let (Argument::Type(a), Argument::Type(b)) = (left, right) {
                    Some((a, b))
                } else {
                    None
                }
            })
            .collect();
        // A single variable binds to an unexpanded canonical leaf in constant
        // work. Composite patterns still need the shared cycle check: canonical
        // roots can contain cached children expanded by earlier constraints.
        if let [(left, right)] = pairs.as_slice() {
            let left = self.root(*left)?;
            let right = self.root(*right)?;
            if matches!(
                (
                    self.terms[left.0 as usize].value,
                    self.terms[right.0 as usize].value,
                ),
                (Value::Known(_), Value::Variable(_)) | (Value::Variable(_), Value::Known(_))
            ) {
                return self.equate_mixed_inputs(store, inputs, true);
            }
        }
        let anchored = self.anchored_inputs(store, &pairs)?;
        if !anchored && self.input_callables(store, &pairs)? {
            return self.equate_mixed_inputs(store, inputs, true);
        }
        self.pairs.clear();
        // A bulk memo must not leave later tiny equations clearing its capacity.
        // Keep ordinary scratch separate, including when this operation fails.
        let ordinary = std::mem::take(&mut self.seen);
        let result = self.equate_mixed_inputs(store, inputs, false);
        self.seen = ordinary;
        result?;
        if anchored {
            Ok(())
        } else {
            self.acyclic_inputs(store, &pairs)
        }
    }

    /// Matching every root against a fixed finite graph establishes its shape
    /// and finiteness together. No unknown can make a cycle in these immutable
    /// anchors; a callable or projection on the other side is a shape mismatch.
    /// A single unanchored pair instead requires the complete joint cycle check.
    fn anchored_inputs(
        &mut self,
        store: &Store,
        pairs: &[(TermId, TermId)],
    ) -> Result<bool, Error> {
        for &(left, right) in pairs {
            self.step()?;
            let left = self.root(left)?;
            let right = self.root(right)?;
            if self
                .completed(store, left)?
                .is_none_or(|facts| facts.callables)
                && self
                    .completed(store, right)?
                    .is_none_or(|facts| facts.callables)
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn equate_mixed_inputs(
        &mut self,
        store: &Store,
        inputs: &[(Argument, Argument)],
        scoped: bool,
    ) -> Result<(), Error> {
        // Preserve subject/argument order in the lifetime proof stream. The
        // structural memo spans every root, even across non-type arguments.
        for &(left, right) in inputs {
            match (left, right) {
                (Argument::Type(a), Argument::Type(b)) if scoped => {
                    self.equate_scoped_terms(store, a, b)?;
                }
                (Argument::Type(a), Argument::Type(b)) => {
                    self.pairs.push((a, b));
                    self.unify_pending::<false>(store, true, None)?;
                }
                (Argument::Const(a), Argument::Const(b)) => self.equate_consts(a, b)?,
                (Argument::Lifetime(a), Argument::Lifetime(b)) => {
                    if a != b {
                        self.region_equalities.push((a, b));
                    }
                }
                _ => return Err(Error::InvalidIdentity),
            }
        }
        Ok(())
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
            if let Some(facts) = self.completed(store, id)? {
                if facts.callables {
                    return Ok(true);
                }
                continue;
            }
            let term = self.terms[id.0 as usize];
            match term.value {
                Value::Known(ty) => {
                    if store.contains_callables(ty).ok_or(Error::InvalidIdentity)? {
                        return Ok(true);
                    }
                }
                Value::Construct(Constructor::Function { .. } | Constructor::Dynamic { .. }) => {
                    return Ok(true);
                }
                Value::Construct(_) => {
                    self.account_work((term.end - term.start) as usize)?;
                    work.extend_from_slice(&self.edges[term.start as usize..term.end as usize]);
                }
                Value::Variable(_) => {}
            }
        }
        Ok(false)
    }

    fn acyclic_inputs(&mut self, store: &Store, pairs: &[(TermId, TermId)]) -> Result<(), Error> {
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
            // Fully determined projection-free inputs cannot acquire a cycle
            // through compatible equations. Unknown neighbors still need colors.
            if self.completed(store, id)?.is_some() {
                continue;
            }
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
    fn finite_anchors_bound_matching_work_and_leave_other_cycles_checked() -> Result<(), Error> {
        let store = Store::default();
        for width in [16, 64, 256, 1024] {
            let mut context = Context::default();
            let byte = context.known(&store, TypeId::primitive(Primitive::U8))?;
            let known = context.construct(Constructor::Tuple, &vec![byte; width])?;
            context.remember_structure(&store, known)?;
            let hole = context.variable(super::super::Domain::Any)?;
            let unknown = context.construct(Constructor::Tuple, &vec![hole; width])?;
            let before = context.steps;
            context
                .equate_input_pairs(&store, &[(Argument::Type(known), Argument::Type(unknown))])?;
            assert!(context.steps - before <= width + 8);
            assert_eq!(context.root(hole)?, context.root(byte)?);
        }
        let mut context = Context::default();
        let byte = context.known(&store, TypeId::primitive(Primitive::U8))?;
        let x = context.variable(super::super::Domain::Any)?;
        let y = context.variable(super::super::Domain::Any)?;
        let cycle = context.construct(Constructor::Tuple, &[y])?;
        assert_eq!(
            context.equate_input_pairs(
                &store,
                &[
                    (Argument::Type(x), Argument::Type(byte)),
                    (Argument::Type(y), Argument::Type(cycle)),
                ]
            ),
            Err(Error::Occurs)
        );
        Ok(())
    }

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
