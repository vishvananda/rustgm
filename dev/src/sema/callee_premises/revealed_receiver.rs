//! Selected implementation methods take the verified storage of opaque receivers.
use super::{Argument, Clause, Declarations, DefId, PredicateBound};
use crate::types::{Kind, TypeId, parameters::predicates::Predicate, traits::ApplicationId};

pub(super) fn storage(facts: &mut Declarations, clause: Clause) -> Result<Clause, &'static str> {
    match clause {
        Clause::Predicate(predicate) => adjusted(facts, predicate).map(Clause::Predicate),
        Clause::ForAll(id) => {
            let mut quantified = facts
                .types
                .quantified(id)
                .ok_or("missing selected method premise binder")?;
            let predicate = adjusted(facts, quantified.predicate)?;
            if predicate == quantified.predicate {
                return Ok(clause);
            }
            quantified.predicate = predicate;
            facts
                .types
                .intern_quantified(quantified)
                .map(Clause::ForAll)
        }
        Clause::UnstableFeature(_) => Ok(clause),
    }
}

fn adjusted(facts: &mut Declarations, mut premise: Predicate) -> Result<Predicate, &'static str> {
    if let Some((application, effect)) = premise.bound.trait_parts()
        && let Some(updated) = receiver(facts, premise.ty, application)?
    {
        premise.bound = PredicateBound::trait_application(updated, effect);
    } else if matches!(premise.bound, PredicateBound::ProjectionEqual(_))
        && let Some(Kind::Projection(mut projection)) = facts.types.kind(premise.ty)
        && facts.associated.callable_output() == Some(DefId(projection.item.0))
        && let Some(updated) = receiver(facts, projection.subject, projection.trait_application)?
    {
        projection.trait_application = updated;
        premise.ty = facts.types.intern(Kind::Projection(projection))?;
    }
    Ok(premise)
}

fn receiver(
    facts: &mut Declarations,
    method: TypeId,
    id: ApplicationId,
) -> Result<Option<ApplicationId>, &'static str> {
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(method)
    else {
        return Ok(None);
    };
    let mut application = facts
        .types
        .trait_application(id)
        .ok_or("missing callable trait application")?;
    if facts
        .associated
        .callable(DefId(application.item.0))
        .is_none()
        || facts.associated.owner(DefId(item.0)).is_none()
    {
        return Ok(None);
    }
    let mut arguments = facts
        .types
        .arguments(application.arguments)
        .ok_or("missing callable arguments")?;
    let (Some(Argument::Type(tuple)), None) = (arguments.next(), arguments.next()) else {
        return Ok(None);
    };
    let Some(Kind::Tuple(inputs)) = facts.types.kind(tuple) else {
        return Ok(None);
    };
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
        return Err("selected method has no callable signature");
    };
    let first = |list| {
        facts
            .types
            .list(list)
            .and_then(|types| types.first())
            .and_then(|&ty| facts.types.kind(ty))
    };
    let (
        Some(Kind::Reference {
            referent,
            mutable,
            region,
        }),
        Some(Kind::Reference {
            referent: actual,
            mutable: actual_mutable,
            ..
        }),
    ) = (first(inputs), first(parameters))
    else {
        return Ok(None);
    };
    if actual_mutable != mutable
        || !matches!(facts.types.kind(referent), Some(Kind::Opaque(..)))
        || matches!(facts.types.kind(actual), Some(Kind::Opaque(..)))
    {
        return Ok(None);
    }
    // Dispatch has selected an implementation method's concrete signature.
    // Its source receiver still names the opaque application. Retain the exact
    // borrow and binder, then let ordinary callable proof relate the inputs.
    facts.prepare_opaque_storage()?;
    let referent = facts.types.storage_head(referent)?;
    let receiver = facts.types.intern(Kind::Reference {
        referent,
        mutable,
        region,
    })?;
    let mut values = facts
        .types
        .list(inputs)
        .ok_or("missing callable input list")?
        .to_vec();
    values[0] = receiver;
    let inputs = facts.types.intern_list(&values)?;
    let tuple = facts.types.intern(Kind::Tuple(inputs))?;
    application.arguments = facts.types.intern_arguments(&[Argument::Type(tuple)])?;
    facts.types.intern_trait_application(application).map(Some)
}
