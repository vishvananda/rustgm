//! Exact checked premises need no new lifetime universe or erased input graph.
use super::{Context, Declarations, Source, inference};
use crate::{
    sema::traits::{assumptions::Assumptions, results},
    types::{
        GenericArgs, Kind, Store, TypeId,
        parameters::{
            clauses::Clause,
            environment::Environment,
            predicates::{Bound, Predicate},
        },
        projection::Application,
        substitute::{Request, Resolution},
    },
};

#[cfg(test)]
#[path = "../../../../tests/invariants/canonical_equalities.rs"]
mod tests;

/// The caller already retained formation of the complete request inputs. Exact
/// facts need no fresh aliases; ambiguous buckets retain ordinary scoped proof.
pub(in crate::sema) fn normalized(
    facts: &mut Declarations,
    environment: Option<&Environment>,
    request: Request,
) -> Result<Option<(Source, Resolution)>, String> {
    let Some(application) = request.alias.associated() else {
        return Ok(None);
    };
    if let Some(environment) = environment
        && let Some((parent, result, steps)) =
            exact_equality(&facts.types, environment, application)?
    {
        let mut context = Context::default();
        context.account_work(steps).map_err(inference)?;
        let competing = if let Some(Kind::Projection(parent)) =
            facts.types.kind(application.subject)
        {
            let (competing, steps) = results::has_equalities(facts, parent.item, application.item)?;
            context.account_work(steps).map_err(inference)?;
            competing
        } else {
            false
        };
        if !competing {
            return Ok(Some((Source::Assumption(parent), Resolution::Type(result))));
        }
    }
    Ok(opaque_normalized(facts, environment, request)?.map(|source| (source, Resolution::Opaque)))
}

fn exact_equality(
    store: &Store,
    environment: &Environment,
    application: Application,
) -> Result<Option<(Predicate, TypeId, usize)>, String> {
    let mut candidates = environment.projection_candidates(application.item);
    let Some(Clause::Predicate(equality)) = candidates.next() else {
        return Ok(None);
    };
    if candidates.next().is_some() || store.kind(equality.ty) != Some(Kind::Projection(application))
    {
        return Ok(None);
    }
    let Bound::ProjectionEqual(result) = equality.bound else {
        return Err("invalid exact projection equality".into());
    };
    let goal = Predicate {
        ty: application.subject,
        bound: Bound::Trait(application.trait_application),
    };
    let (assumptions, steps) = Assumptions::from_environment(store, Some(environment))?;
    Ok(assumptions
        .exact(store, goal)
        .map_err(inference)?
        .map(|proof| (proof.assumption(), result, steps)))
}

/// Only a normalizer retaining its inputs' formation proofs can use this fact.
/// A sole exact clause contributes no equations. Competing clauses or possible
/// result equalities retain the ordinary scoped selection/normalization route.
pub(in crate::sema) fn opaque_normalized(
    facts: &mut Declarations,
    environment: Option<&Environment>,
    request: Request,
) -> Result<Option<Source>, String> {
    let Some(application) = request.alias.associated() else {
        return Ok(None);
    };
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
    if let Some(candidate) = candidates.next() {
        return Ok(
            (candidate == Clause::Predicate(goal) && candidates.next().is_none())
                .then_some(Source::Assumption(goal)),
        );
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
