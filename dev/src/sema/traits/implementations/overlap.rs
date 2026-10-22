//! Structural impl disjointness uses simultaneous existential type/const equations.
mod disjointness;
mod inputs;
mod iteration;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        infer::{self, Context},
        parameters::predicates::Predicate,
    },
};
pub(in crate::sema) use iteration::possible as iteration;

pub(in crate::sema) fn possible(
    facts: &mut Declarations,
    left: (Predicate, DefId),
    right: (Predicate, DefId),
) -> Result<(bool, usize), String> {
    let mut context = Context::default();
    let mut inputs = inputs::Inputs::default();
    let a = inputs.instantiate(facts, &mut context, left)?;
    let b = inputs.instantiate(facts, &mut context, right)?;
    let possible = match context.equate_predicate_shapes(&facts.types, &a, &b) {
        Ok(()) => !disjointness::has_unsized(facts, &mut context, &inputs)?,
        Err(infer::Error::Mismatch | infer::Error::Occurs) => false,
        Err(error) => return Err(error.to_string()),
    };
    Ok((possible, context.steps))
}

/// Source reference impls compete with the canonical blanket only if its exact
/// existential referent condition remains possible. No evidence from this
/// declaration-only query escapes into a caller or executable factory.
pub(in crate::sema) fn reference_comparison(
    facts: &mut Declarations,
    header: (Predicate, DefId),
) -> Result<(bool, usize), String> {
    use crate::sema::{
        primitive_methods::{self, Candidate},
        traits::selection,
    };
    let crate::types::parameters::predicates::Bound::Trait(application) = header.0.bound else {
        return Ok((false, 0));
    };
    let item = facts
        .types
        .trait_application(application)
        .ok_or("missing coherence trait application")?
        .item;
    if facts.associated.comparison(DefId(item.0)).is_none()
        || !matches!(
            facts.types.kind(header.0.ty),
            Some(crate::types::Kind::Reference { .. })
        )
    {
        return Ok((false, 0));
    }
    let mut context = Context::default();
    let mut inputs = inputs::Inputs::default();
    let goal = inputs.instantiate(facts, &mut context, header)?;
    let candidate = primitive_methods::candidate(facts, &mut context, &goal, false)
        .map_err(|e| e.to_string())?;
    let Candidate::Conditional { prerequisite, .. } = candidate else {
        return Ok((false, context.steps));
    };
    if disjointness::has_unsized(facts, &mut context, &inputs)? {
        return Ok((false, context.steps));
    }
    let possible = selection::may_overlap(facts, &mut context, &prerequisite)?;
    Ok((possible, context.steps))
}
