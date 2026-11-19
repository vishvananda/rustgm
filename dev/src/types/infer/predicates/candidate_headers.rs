//! An implementation header cannot equal a goal's rigid normal form with another head.
use super::{Context, Error, Predicate, Store, TermId};
use crate::types::{
    infer::{Constructor, Value, arguments::Argument, scoped::Blocked},
    parameters::predicates::Bound,
};

impl Context {
    /// Equate a normalized goal with an implementation header in a disposable
    /// trial context. Ordinary headers use the bulk input relation. A projected
    /// header keeps each unresolved result comparison as a condition, after
    /// matching every independent input. These conditions can use the trial's
    /// now-bound parameters and scoped caller evidence without treating the
    /// projection as injective. A rigid normal form excludes a non-alias head.
    /// Comparisons beneath an inner binder stay in scoped normalization.
    ///
    /// # Errors
    /// Rejects mismatches, occurs cycles, invalid identities and exhausted work.
    pub fn equate_candidate_header(
        &mut self,
        store: &Store,
        goal: &Predicate,
        header: &Predicate,
        projected_header: bool,
    ) -> Result<Vec<Predicate>, Error> {
        if !projected_header {
            match self.equate_predicates(store, goal, header) {
                Err(Error::Normalization) if self.has_rigid_marks() => {}
                result => return result.map(|()| Vec::new()),
            }
        }
        self.relate_header_regions(store, goal, header, projected_header, true)
    }

    /// A specialization implication retains the parent's actual lifetime
    /// equations. Its structural test ignores effects just like plain headers.
    pub(crate) fn equate_specializing_header(
        &mut self,
        store: &Store,
        child: &Predicate,
        parent: &Predicate,
    ) -> Result<Vec<Predicate>, Error> {
        self.relate_header_regions(store, child, parent, true, false)
    }

    fn relate_header_regions(
        &mut self,
        store: &Store,
        goal: &Predicate,
        header: &Predicate,
        projected_header: bool,
        effects: bool,
    ) -> Result<Vec<Predicate>, Error> {
        let mut blocked = Vec::new();
        let mut under_binder = false;
        for (left, right) in header_inputs(goal, header, effects)? {
            self.step()?;
            match (left, right) {
                (Argument::Type(left), Argument::Type(right)) => self.equate_scoped_graph(
                    store,
                    left,
                    right,
                    &[],
                    Some(Blocked::WithScope(&mut blocked, &mut under_binder)),
                )?,
                (Argument::Const(left), Argument::Const(right)) => {
                    self.equate_consts(left, right)?;
                }
                (Argument::Lifetime(left), Argument::Lifetime(right)) => {
                    if left != right {
                        self.region_equalities.push((left, right));
                    }
                }
                _ => return Err(Error::Mismatch),
            }
        }
        self.header_conditions(store, blocked, projected_header && !under_binder)
    }

    /// Compare existential headers without binding a parameter to an unevaluated
    /// result. The overlap checker may use completed equality proofs to rule out
    /// an intersection. It cannot use an unresolved one to establish absence.
    pub(crate) fn equate_projected_header_shapes(
        &mut self,
        store: &Store,
        left: &Predicate,
        right: &Predicate,
    ) -> Result<Vec<Predicate>, Error> {
        let mut conditions = Vec::new();
        for (left, right) in header_inputs(left, right, false)? {
            self.step()?;
            match (left, right) {
                (Argument::Type(left), Argument::Type(right)) => {
                    conditions.extend(self.equate_shape_goals(store, left, right)?);
                }
                (Argument::Const(left), Argument::Const(right)) => {
                    self.equate_consts(left, right)?;
                }
                (Argument::Lifetime(_), Argument::Lifetime(_)) => {}
                _ => return Err(Error::Mismatch),
            }
        }
        Ok(conditions)
    }

    fn header_conditions(
        &mut self,
        store: &Store,
        blocked: Vec<(TermId, TermId)>,
        normalize: bool,
    ) -> Result<Vec<Predicate>, Error> {
        self.account_work(blocked.len())?;
        let mut conditions = Vec::new();
        let mut deferred = !normalize;
        for (actual, pattern) in blocked {
            let (actual, pattern) = (self.root(actual)?, self.root(pattern)?);
            if actual == pattern {
                continue;
            }
            if self.rigid_against(store, actual, pattern)?
                || self.rigid_against(store, pattern, actual)?
            {
                return Err(Error::Mismatch);
            }
            if normalize
                && self.opaque_projection(pattern).is_none()
                && self.projection_item(store, pattern)?.is_some()
            {
                conditions.push(Predicate::equality(pattern, actual));
            } else {
                deferred = true;
            }
        }
        if deferred {
            Err(Error::Normalization)
        } else {
            Ok(conditions)
        }
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

fn header_inputs<'a>(
    left: &'a Predicate,
    right: &'a Predicate,
    effects: bool,
) -> Result<impl Iterator<Item = (Argument, Argument)> + 'a, Error> {
    let (actual, pattern) = match (left.trait_parts(), right.trait_parts()) {
        (Some((left_item, left_effect, actual)), Some((right_item, right_effect, pattern)))
            if left_item == right_item
                && actual.len() == pattern.len()
                && (!effects || left_effect == right_effect) =>
        {
            (actual, pattern)
        }
        _ if matches!(
            (left.builtin_bound(), right.builtin_bound()),
            (Some(Bound::Auto(a)), Some(Bound::Auto(b))) if a == b
        ) =>
        {
            (&[][..], &[][..])
        }
        _ => return Err(Error::Mismatch),
    };
    let subjects = (Argument::Type(left.subject), Argument::Type(right.subject));
    Ok(std::iter::once(subjects).chain(actual.iter().copied().zip(pattern.iter().copied())))
}
