//! Borrowed declared Future/Output equality carries no hidden representation.
use crate::{
    resolve::model::DefId,
    types::{
        GenericArgs, ItemId, Kind,
        parameters::{clauses::Clause, predicates::Bound},
    },
};

pub(crate) fn check(
    store: &crate::types::Store,
    contract: &crate::sema::opaque::Declaration,
    output: crate::types::TypeId,
    future: DefId,
    member: DefId,
) -> Result<(), &'static str> {
    let [Clause::Predicate(first), Clause::Predicate(second)] = contract.bounds.as_slice() else {
        return Err("opaque Future bounds changed their actual quantified contract");
    };
    let Bound::Trait(application) = first.bound else {
        return Err("opaque Future lost its actual trait bound");
    };
    let trait_ = store
        .trait_application(application)
        .ok_or("opaque Future trait application is unknown")?;
    let Some(Kind::Projection(projected)) = store.kind(second.ty) else {
        return Err("opaque Output equality lost its actual projection");
    };
    if first.ty != contract.application
        || trait_.item != ItemId(future.0)
        || trait_.arguments != GenericArgs::EMPTY
        || projected.item != ItemId(member.0)
        || projected.subject != contract.application
        || projected.trait_application != application
        || projected.arguments != GenericArgs::EMPTY
        || second.bound != Bound::ProjectionEqual(output)
    {
        return Err("opaque Future/Output bounds changed their actual canonical identity");
    }
    Ok(())
}
