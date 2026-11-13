//! Nested bodies assume the lifetime requirements their creator's signature implies.
//!
//! rustc propagates a closure's unproved type-outlives requirement, such as
//! `T: 'a` from a captured `FnMut(&'a T)` call, to the creator, which proves
//! it from its own signature's implied bounds. The creator always proves those
//! bounds, so assuming them in the closure accepts the same programs. The
//! nested body's leading slots are its creator's universals in order, as its
//! inherited explicit bounds already require.
use crate::support::hash::HashMap;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        Kind, TypeId,
        outlives::{Pair, Queries},
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};
use std::collections::hash_map::Entry;

#[derive(Clone, Default)]
pub(super) struct Implied {
    pub regions: Vec<Pair>,
    pub types: Vec<Clause>,
}

/// Each creator's implied requirements are computed once for its nested bodies.
#[derive(Default)]
pub(super) struct Creators {
    implied: HashMap<DefId, Implied>,
}
impl Creators {
    /// Projection requirements are not assumed; each nested body still proves them.
    /// An async creator's body has its own signature and contributes none.
    pub fn implied(
        &mut self,
        facts: &Declarations,
        owner: DefId,
    ) -> Result<Option<&Implied>, &'static str> {
        // Constant demands can precede later declaration publication. Look up
        // the current indexed header; a missing owner is not a cached absence.
        let Some(function) = facts
            .function_indices
            .get(&owner)
            .and_then(|&index| facts.functions.get(index))
        else {
            return Ok(None);
        };
        if function.body.is_none() || facts.authored_async_body(function)?.is_some() {
            return Ok(None);
        }
        let signature = function.signature;
        Ok(Some(match self.implied.entry(owner) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(compute(facts, signature)?),
        }))
    }
}

fn compute(facts: &Declarations, signature: TypeId) -> Result<Implied, &'static str> {
    let types = &facts.types;
    let Some(Kind::Function {
        parameters, result, ..
    }) = types.kind(signature)
    else {
        return Err("closure creator has no lifetime signature");
    };
    let parameters = types
        .list(parameters)
        .ok_or("missing closure creator inputs")?;
    let mut queries = Queries::new(
        types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let mut implied = Implied::default();
    for &ty in parameters.iter().chain(std::iter::once(&result)) {
        let requirements = queries.symbolic_requirements(ty)?;
        implied
            .regions
            .extend_from_slice(requirements.scalars.regions);
        for &(id, shorter) in requirements.scalars.parameters {
            let subject = types
                .find_kind(Kind::Parameter(id))
                .ok_or("closure creator parameter has no type identity")?;
            implied.types.push(Clause::Predicate(Predicate {
                ty: subject,
                bound: Bound::Outlives(shorter),
            }));
        }
    }
    Ok(implied)
}
