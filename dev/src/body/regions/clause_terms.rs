//! Keep clause formation distinct from the permissions available inside a body.
use super::{Kind, NodeKey};
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::parameters::{clauses::Clause, predicates::Bound},
};

pub(super) fn collect(
    facts: &Declarations,
    owner: DefId,
    mut add: impl FnMut(NodeKey, Kind) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    for predicate in facts.generics.predicates.get(&owner).into_iter().flatten() {
        match predicate.value {
            Clause::Predicate(value) => {
                // Equality formation follows its separate trait premise. The
                // opaque subject cannot replace the result's own formation.
                let ty = match value.bound {
                    Bound::ProjectionEqual(result) => result,
                    _ => value.ty,
                };
                add(predicate.source, Kind::WellFormed(ty))?;
            }
            Clause::ForAll(id) => {
                let value = facts
                    .types
                    .quantified(id)
                    .ok_or("invalid body formation clause")?;
                let ty = match value.predicate.bound {
                    Bound::Trait(_) => {
                        // Whole local-dependent inputs were deferred at declaration.
                        // Completed independent roots already live outside the binder.
                        for ty in facts.generics.trait_formation_inputs(id)? {
                            add(predicate.source, Kind::WellFormed(ty))?;
                        }
                        continue;
                    }
                    Bound::ProjectionEqual(result) => result,
                    Bound::Outlives(_) => value.predicate.ty,
                    Bound::Copy | Bound::Clone | Bound::Sized => {
                        return Err("quantified builtin formation requires a scoped consumer");
                    }
                };
                add(
                    predicate.source,
                    Kind::ScopedWellFormed {
                        ty,
                        bound_regions: value.bound_regions,
                    },
                )?;
            }
        }
    }
    Ok(())
}
