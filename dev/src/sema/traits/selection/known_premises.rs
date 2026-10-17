//! Exact checked premises need no new lifetime universe or erased input graph.
use super::{Context, Declarations, Source, inference};
use crate::{
    sema::traits::results,
    types::{
        GenericArgs, Kind,
        parameters::{
            environment::Environment,
            predicates::{Bound, Predicate},
        },
        substitute::Request,
    },
};

/// Only a normalizer retaining its inputs' formation proofs can use this fact.
/// A sole exact clause contributes no equations. Competing clauses or possible
/// result equalities retain the ordinary scoped selection/normalization route.
pub(in crate::sema) fn opaque_normalized(
    facts: &mut Declarations,
    environment: Option<&Environment>,
    request: Request,
) -> Result<Option<Source>, String> {
    let application = request.application;
    let Some(Kind::Projection(parent)) = facts.types.kind(application.subject) else {
        return Ok(None);
    };
    if application.arguments != GenericArgs::EMPTY
        || environment
            .is_some_and(|env| env.projection_candidates(application.item).next().is_some())
    {
        return Ok(None);
    }
    let (equations, steps) = results::has_equalities(facts, parent.item, application.item)?;
    let mut context = Context::default();
    context.account_work(steps).map_err(inference)?;
    if equations {
        return Ok(None);
    }
    let trait_item = facts
        .types
        .trait_application(application.trait_application)
        .ok_or("missing canonical projection trait")?
        .item;
    let goal = Predicate {
        ty: application.subject,
        bound: Bound::Trait(application.trait_application),
    };
    let mut candidates = environment
        .into_iter()
        .flat_map(|env| env.projected_trait_candidates(trait_item, parent.item));
    if let Some(&candidate) = candidates.next() {
        return Ok((candidate == goal && candidates.next().is_none())
            .then_some(Source::Assumption(candidate)));
    }
    let Some((rules, steps)) = results::traits(facts, application.subject, trait_item)? else {
        return Ok(None);
    };
    context.account_work(steps).map_err(inference)?;
    Ok(rules
        .exact(&facts.types, goal)
        .map_err(inference)?
        .map(|proof| Source::ResultBound(proof.assumption())))
}
