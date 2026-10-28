//! Conditional obligations retain the effect of their enclosing trait use.
use crate::{
    support::constness::Constness,
    types::{
        Store,
        parameters::{
            clauses::{Clause, Quantified},
            predicates::Bound,
        },
    },
};

pub(in crate::sema) fn in_context(
    store: &mut Store,
    clause: Clause,
    context: Constness,
) -> Result<Clause, &'static str> {
    let (mut predicate, count) = match clause {
        Clause::UnstableFeature(id) => {
            store
                .feature_name(id)
                .ok_or("invalid feature effect clause")?;
            return Ok(clause);
        }
        Clause::Predicate(predicate) => (predicate, None),
        Clause::ForAll(id) => {
            let value = store
                .quantified(id)
                .ok_or("invalid conditional obligation clause")?;
            (value.predicate, Some(value.bound_regions))
        }
    };
    let Bound::MaybeConstTrait(application) = predicate.bound else {
        return Ok(clause);
    };
    predicate.bound = Bound::trait_application(application, context);
    count.map_or(Ok(Clause::Predicate(predicate)), |bound_regions| {
        store
            .intern_quantified(Quantified {
                bound_regions,
                predicate,
            })
            .map(Clause::ForAll)
    })
}
