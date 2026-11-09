//! Elaborated supertrait premises that inherit a declared bound's certificate.
//!
//! A certificate says a trait premise's projection terms are normal forms in
//! its environment. An owner's declared bound is certified only after its own
//! bound normalization completed under the raw environment, which already held
//! every elaborated premise and equation. An elaborated premise with the same
//! interned subject and binder shape whose trait application has no projection
//! has no projection term but that subject, so it is normal for the same
//! reason. Projection-bearing supertrait arguments, binder changes and other
//! subjects stay uncertified: each would need its own normalization proof.
use super::Predicate;
use crate::support::hash::HashSet;
use crate::types::{
    Store, TypeId,
    parameters::{clauses::Clause, predicates::Bound},
};

/// A trait premise's binder count and subject, and whether its trait
/// application has a projection; `None` for any other clause.
type Shape = ((Option<u32>, TypeId), bool);

fn shape(store: &Store, clause: Clause) -> Result<Option<Shape>, &'static str> {
    let (binder, predicate) = match clause {
        Clause::Predicate(predicate) => (None, predicate),
        Clause::ForAll(id) => {
            let value = store
                .quantified(id)
                .ok_or("invalid quantified elaborated premise")?;
            (Some(value.bound_regions), value.predicate)
        }
        Clause::UnstableFeature(_) => return Ok(None),
    };
    let (Bound::Trait(application)
    | Bound::ConstTrait(application)
    | Bound::MaybeConstTrait(application)) = predicate.bound
    else {
        return Ok(None);
    };
    let projections = store
        .trait_application_has_projections(application)
        .ok_or("invalid elaborated premise trait application")?;
    Ok(Some(((binder, predicate.ty), projections)))
}

/// The elaborated premises that inherit a certificate from `normal`, the
/// owner's certified declared bounds, in their elaboration order.
pub(in crate::sema) fn inherited(
    store: &Store,
    normal: &[Clause],
    elaborated: &[Predicate],
) -> Result<Vec<Clause>, &'static str> {
    let mut subjects = HashSet::default();
    for &clause in normal {
        if let Some((subject, _)) = shape(store, clause)? {
            subjects.insert(subject);
        }
    }
    let mut inherited = Vec::new();
    for predicate in elaborated {
        if let Some((subject, false)) = shape(store, predicate.value)?
            && subjects.contains(&subject)
        {
            inherited.push(predicate.value);
        }
    }
    Ok(inherited)
}
