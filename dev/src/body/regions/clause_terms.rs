//! Keep clause formation distinct from the permissions available inside a body.
use super::Kind;
use crate::resolve::body_origin::ProofSite;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::parameters::{clauses::Clause, predicates::Bound},
};

pub(super) fn collect(
    facts: &Declarations,
    owner: DefId,
    mut add: impl FnMut(ProofSite, Kind) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    for predicate in
        crate::sema::declaration_inputs::own(&facts.generics, facts.crates.as_deref(), owner)?
    {
        match predicate.value {
            Clause::UnstableFeature(_) => {}
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
                    Bound::Trait(_) | Bound::ConstTrait(_) | Bound::MaybeConstTrait(_) => {
                        // Whole local-dependent inputs were deferred at declaration.
                        // Completed independent roots already live outside the binder.
                        for ty in facts.generics.trait_formation_inputs(id)? {
                            add(predicate.source, Kind::WellFormed(ty))?;
                        }
                        continue;
                    }
                    Bound::ProjectionEqual(result) => result,
                    Bound::Outlives(_) => value.predicate.ty,
                    Bound::Copy | Bound::Clone | Bound::Sized | Bound::Auto(_) => {
                        if let Some(ty) = facts.generics.builtin_formation_input(owner, id)? {
                            add(predicate.source, Kind::WellFormed(ty))?;
                        }
                        continue;
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
