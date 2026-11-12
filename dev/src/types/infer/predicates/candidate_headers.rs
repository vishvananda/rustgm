//! An implementation header cannot equal a goal's rigid normal form with another head.
use super::{Context, Error, Predicate, Requirement, Store, TermId};
use crate::types::infer::{Constructor, Value, arguments::Argument};

impl Context {
    /// Equate a normalized goal with an implementation header in a disposable
    /// trial context. rustc 1.98.0 relates a rigid alias structurally: it
    /// equals an unknown or the same alias, never another head. The ordinary
    /// relation runs first. Only when it is blocked on an associated result
    /// and the goal carries rigid normal-form marks are all inputs related
    /// again in blocked mode. A blocked pair joining a recorded rigid normal
    /// form to a parameter or any other non-alias constructor refutes the
    /// header with `Error::Mismatch`. Every other blocked pair, including a
    /// projection or opaque result from either side, keeps
    /// `Error::Normalization`. No premise, impl condition or lifetime is read.
    ///
    /// # Errors
    /// Rejects mismatches, occurs cycles, invalid identities and exhausted work.
    pub fn equate_candidate_header(
        &mut self,
        store: &Store,
        goal: &Predicate,
        header: &Predicate,
    ) -> Result<(), Error> {
        match self.equate_predicates(store, goal, header) {
            Err(Error::Normalization) if self.has_rigid_marks() => {}
            result => return result,
        }
        let (Requirement::Trait(a), Requirement::Trait(b)) =
            (&goal.requirement, &header.requirement)
        else {
            return Err(Error::Normalization);
        };
        let inputs = a.arguments.iter().zip(&b.arguments).filter_map(|pair| {
            if let (Argument::Type(left), Argument::Type(right)) = pair {
                Some((*left, *right))
            } else {
                None
            }
        });
        let mut blocked = Vec::new();
        for (left, right) in std::iter::once((goal.subject, header.subject)).chain(inputs) {
            self.equate_scoped_graph(store, left, right, &[], Some(&mut blocked))?;
        }
        self.account_work(blocked.len())?;
        for (left, right) in blocked {
            let (left, right) = (self.root(left)?, self.root(right)?);
            if self.rigid_against(store, left, right)? || self.rigid_against(store, right, left)? {
                return Err(Error::Mismatch);
            }
        }
        Err(Error::Normalization)
    }

    fn rigid_against(
        &mut self,
        store: &Store,
        rigid: TermId,
        other: TermId,
    ) -> Result<bool, Error> {
        if self.rigid_projection(rigid).is_none()
            || matches!(self.terms[other.0 as usize].value, Value::Variable(_))
        {
            return Ok(false);
        }
        Ok(!matches!(
            self.materialize(store, other)?,
            Constructor::Projection(_) | Constructor::Opaque(..) | Constructor::Future(..)
        ))
    }
}
