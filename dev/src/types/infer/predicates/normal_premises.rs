//! Relate a goal's rigid normal forms with a premise its environment certified normal.
use super::{Canonical, CanonicalArgument, Context, Error, Store, TermId, TypeId};
use crate::support::hash::HashSet;
use crate::types::infer::{Constructor, arguments::Slot};
#[cfg(test)]
#[path = "../../../../tests/invariants/normal_premises.rs"]
mod tests;

/// One certified premise. Its outermost projection subterms are collected at
/// most once per equation, on the first blocked pair whose identities differ.
pub(in crate::types::infer) struct Premise {
    canonical: Canonical,
    subterms: Option<HashSet<TypeId>>,
}

impl Premise {
    pub(in crate::types::infer) const fn new(canonical: Canonical) -> Self {
        Self {
            canonical,
            subterms: None,
        }
    }
}

/// A relation failure past the gate keeps the exact compare's result.
const fn unchanged(error: Error) -> Error {
    match error {
        Error::Mismatch | Error::Occurs | Error::Normalization => Error::Normalization,
        other => other,
    }
}

impl Context {
    /// Blocked pairs of a certified premise equation, in their original order.
    /// A pair with differing identities is related only when its goal side is a
    /// recorded rigid normal form, its premise side is one of the premise's
    /// projection subterms, and both apply one associated item with one layout.
    /// Every other differing pair keeps the exact compare's `Error::Normalization`.
    pub(in crate::types::infer) fn relate_normal_premise(
        &mut self,
        store: &mut Store,
        mut work: Vec<(TermId, TermId)>,
        premise: &mut Premise,
    ) -> Result<(), Error> {
        work.reverse();
        let mut related: Option<HashSet<(TermId, TermId)>> = None;
        while let Some((left, right)) = work.pop() {
            self.step()?;
            let other = self.resolved(store, right)?;
            if self.resolved(store, left)? == other {
                continue;
            }
            let pair = (self.root(left)?, self.root(right)?);
            if !self.rigid_premise_pair(store, pair, other, premise)? {
                return Err(Error::Normalization);
            }
            if related.get_or_insert_with(HashSet::default).insert(pair) {
                self.relate_rigid_inputs(store, pair, &mut work)
                    .map_err(unchanged)?;
            }
        }
        Ok(())
    }

    fn rigid_premise_pair(
        &mut self,
        store: &Store,
        (left, right): (TermId, TermId),
        other: TypeId,
        premise: &mut Premise,
    ) -> Result<bool, Error> {
        if self.rigid_projection(left).is_none() || !self.premise_subterm(store, premise, other)? {
            return Ok(false);
        }
        let (Constructor::Projection(a), Constructor::Projection(b)) = (
            self.materialize(store, left)?,
            self.materialize(store, right)?,
        ) else {
            return Ok(false);
        };
        if a.item != b.item || a.trait_arguments != b.trait_arguments {
            return Ok(false);
        }
        let (first, second) = (self.slots(a.arguments)?, self.slots(b.arguments)?);
        if first.len() != second.len()
            || !first.iter().zip(second).all(|pair| {
                matches!(
                    pair,
                    (Slot::Lifetime(_), Slot::Lifetime(_))
                        | (Slot::Type, Slot::Type)
                        | (Slot::Const(_), Slot::Const(_))
                )
            })
        {
            return Ok(false);
        }
        let (a, b) = (self.terms[left.0 as usize], self.terms[right.0 as usize]);
        Ok(a.end - a.start == b.end - b.start)
    }

    /// Relate the complete input list invariantly, as rustc 1.98.0 relates a
    /// rigid projection's arguments: lifetimes become region equalities for the
    /// caller's region proof, constants are equated and type inputs unify again
    /// in blocked mode. Both roots are fully resolved, so nothing is inferred.
    fn relate_rigid_inputs(
        &mut self,
        store: &Store,
        (left, right): (TermId, TermId),
        work: &mut Vec<(TermId, TermId)>,
    ) -> Result<(), Error> {
        let (Constructor::Projection(a), Constructor::Projection(b)) = (
            self.materialize(store, left)?,
            self.materialize(store, right)?,
        ) else {
            return Err(Error::InvalidIdentity);
        };
        self.equate_arguments(a.arguments, b.arguments, true)?;
        let (a, b) = (self.terms[left.0 as usize], self.terms[right.0 as usize]);
        self.account_work((a.end - a.start) as usize)?;
        for offset in 0..a.end - a.start {
            let child = (
                self.edges[(a.start + offset) as usize],
                self.edges[(b.start + offset) as usize],
            );
            let mut nested = Vec::new();
            self.unify(store, child.0, child.1, true, Some(&mut nested))?;
            work.extend(nested.into_iter().rev());
        }
        Ok(())
    }

    /// The premise's outermost projection subterms, collected once per equation
    /// and charged to this context's ordinary inference work.
    fn premise_subterm(
        &mut self,
        store: &Store,
        premise: &mut Premise,
        ty: TypeId,
    ) -> Result<bool, Error> {
        if let Some(subterms) = &premise.subterms {
            return Ok(subterms.contains(&ty));
        }
        let mut types = vec![premise.canonical.ty];
        if let Some((id, _)) = premise.canonical.bound.trait_parts() {
            let application = store.trait_application(id).ok_or(Error::InvalidIdentity)?;
            let arguments = store
                .arguments(application.arguments)
                .ok_or(Error::InvalidIdentity)?;
            self.account_work(arguments.len())?;
            types.extend(arguments.filter_map(|argument| match argument {
                CanonicalArgument::Type(ty) => Some(ty),
                _ => None,
            }));
        }
        let mut subterms = HashSet::default();
        for canonical in types {
            subterms.extend(self.projection_subterms(store, canonical)?);
        }
        let found = subterms.contains(&ty);
        premise.subterms = Some(subterms);
        Ok(found)
    }
}
