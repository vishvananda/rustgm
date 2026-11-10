//! Detect a forced mismatch before choosing a function-item coercion join.
use super::{
    Constructor, Context, Error, Store, TermId,
    arguments::{ArgumentsId, Slot},
};

impl Context {
    /// A false result leaves unknowns unconstrained; it is not a successful proof
    /// of equality. This query never binds variables or records lifetime facts.
    /// # Errors
    /// Rejects invalid identities and exhausted graph traversal resources.
    pub fn incompatible(&mut self, store: &Store, a: TermId, b: TermId) -> Result<bool, Error> {
        self.incompatible_under(store, a, b, false)
    }
    /// Unsizing may specialize a quantified object's trait inputs even when
    /// ordinary inference sees no type-constructor mismatch. Full conversion
    /// obligations still decide whether that change is permitted. A projected
    /// head awaiting normalization cannot establish a forced mismatch; a false
    /// result still supplies no equality or conversion proof.
    /// # Errors
    /// Reports invalid identities and exhausted graph traversal.
    pub fn unsizing_mismatch(
        &mut self,
        store: &Store,
        a: TermId,
        b: TermId,
    ) -> Result<bool, Error> {
        self.incompatible_under(store, a, b, true)
    }
    fn incompatible_under(
        &mut self,
        store: &Store,
        a: TermId,
        b: TermId,
        unsizing: bool,
    ) -> Result<bool, Error> {
        self.pairs.clear();
        self.seen.clear();
        self.pairs.push((a, b));
        while let Some((a, b)) = self.pairs.pop() {
            self.step()?;
            let a = self.root(a)?;
            let b = self.root(b)?;
            if a == b || !self.seen.insert((a, b)) {
                continue;
            }
            let (Some(left), Some(right)) = (
                self.mismatch_head(store, a, unsizing)?,
                self.mismatch_head(store, b, unsizing)?,
            ) else {
                continue;
            };
            let compatible = self.compatible_heads(store, (a, b), (left, right), unsizing)?;
            if !compatible {
                return Ok(true);
            }
            let a = self.terms[a.0 as usize];
            let b = self.terms[b.0 as usize];
            if a.end - a.start != b.end - b.start {
                return Ok(true);
            }
            self.pairs.extend(
                self.edges[a.start as usize..a.end as usize]
                    .iter()
                    .copied()
                    .zip(self.edges[b.start as usize..b.end as usize].iter().copied()),
            );
        }
        Ok(false)
    }
    fn compatible_heads(
        &mut self,
        store: &Store,
        terms: (TermId, TermId),
        heads: (Constructor, Constructor),
        unsizing: bool,
    ) -> Result<bool, Error> {
        let (a, b) = terms;
        let (left, right) = heads;
        Ok(match (left, right) {
            (Constructor::Adt(x, known), Constructor::AdtArgs(y, inferred))
            | (Constructor::AdtArgs(y, inferred), Constructor::Adt(x, known))
            | (Constructor::FunctionItem(x, known), Constructor::FunctionItemArgs(y, inferred))
            | (Constructor::FunctionItemArgs(y, inferred), Constructor::FunctionItem(x, known)) => {
                let known = self.materialize_arguments(store, known, &mut Vec::new())?;
                x == y && !self.argument_mismatch(known, inferred)?
            }
            (
                Constructor::Reference { mutable: a, .. },
                Constructor::Reference { mutable: b, .. },
            ) => a == b,
            (Constructor::Adt(x, a), Constructor::Adt(y, b))
            | (Constructor::FunctionItem(x, a), Constructor::FunctionItem(y, b)) => {
                x == y
                    && store.region_args(a).ok_or(Error::InvalidIdentity)?.len()
                        == store.region_args(b).ok_or(Error::InvalidIdentity)?.len()
            }
            (Constructor::Closure(x, a), Constructor::Closure(y, b)) => {
                x == y && !self.argument_mismatch(a, b)?
            }
            (Constructor::Opaque(x, a), Constructor::Opaque(y, b))
            | (Constructor::Future(x, a), Constructor::Future(y, b)) => {
                x == y && !self.argument_mismatch(a, b)?
            }
            (
                Constructor::Dynamic {
                    shape: a,
                    arguments: aa,
                    lifetimes: al,
                },
                Constructor::Dynamic {
                    shape: b,
                    arguments: ba,
                    lifetimes: bl,
                },
            ) => {
                a == b
                    && !self.argument_mismatch(aa, ba)?
                    && !(unsizing
                        && (self.object_lifetimes(al)?.bound_regions != 0
                            || self.object_lifetimes(bl)?.bound_regions != 0))
            }
            (Constructor::AdtArgs(x, a), Constructor::AdtArgs(y, b))
            | (Constructor::FunctionItemArgs(x, a), Constructor::FunctionItemArgs(y, b)) => {
                x == y && !self.argument_mismatch(a, b)?
            }
            (Constructor::ArrayInfer(a), Constructor::ArrayInfer(b)) => {
                !self.const_mismatch(a, b)?
            }
            (Constructor::ArrayInfer(a), Constructor::Array(length))
            | (Constructor::Array(length), Constructor::ArrayInfer(a)) => {
                let b = self.const_length(length)?;
                !self.const_mismatch(a, b)?
            }
            (Constructor::ArrayInfer(a), Constructor::ArrayTerm(id))
            | (Constructor::ArrayTerm(id), Constructor::ArrayInfer(a)) => {
                let b = self.known_const(store, id)?;
                !self.const_mismatch(a, b)?
            }
            (Constructor::Projection(_), Constructor::Projection(_)) if unsizing => {
                !self.opaque_mismatch(store, a, b)?
            }
            (a, b) => a == b,
        })
    }
    /// Completed opaque results have independent canonical inputs. Their
    /// existing shape relation ignores regions without proving their equality.
    fn opaque_mismatch(&mut self, store: &Store, a: TermId, b: TermId) -> Result<bool, Error> {
        let a = self.opaque_projection(a).ok_or(Error::InvalidIdentity)?;
        let b = self.opaque_projection(b).ok_or(Error::InvalidIdentity)?;
        match self.opaque_shape(store, a, b) {
            Ok(()) => Ok(false),
            Err(Error::Mismatch) => Ok(true),
            Err(error) => Err(error),
        }
    }
    fn mismatch_head(
        &mut self,
        store: &Store,
        term: TermId,
        unsizing: bool,
    ) -> Result<Option<Constructor>, Error> {
        match self.head(store, term) {
            Err(Error::Normalization) if unsizing => Ok(None),
            result => result,
        }
    }
    fn const_mismatch(
        &mut self,
        a: super::constants::ConstTermId,
        b: super::constants::ConstTermId,
    ) -> Result<bool, Error> {
        if self.constants().ty(a)? != self.constants().ty(b)? {
            return Ok(true);
        }
        Ok(
            matches!((self.constants().value(a)?,self.constants().value(b)?),(Some(a),Some(b)) if a!=b),
        )
    }
    fn argument_mismatch(&mut self, a: ArgumentsId, b: ArgumentsId) -> Result<bool, Error> {
        let count = self.slots(a)?.len();
        if count != self.slots(b)?.len() {
            return Ok(true);
        }
        for index in 0..count {
            self.step()?;
            match (self.slots(a)?[index], self.slots(b)?[index]) {
                (Slot::Lifetime(_), Slot::Lifetime(_)) | (Slot::Type, Slot::Type) => {}
                (Slot::Const(a), Slot::Const(b)) => {
                    if self.const_mismatch(a, b)? {
                        return Ok(true);
                    }
                }
                _ => return Ok(true),
            }
        }
        Ok(false)
    }
}
