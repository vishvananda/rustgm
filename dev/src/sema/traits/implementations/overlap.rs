//! Structural impl disjointness uses simultaneous existential type/const equations.
mod disjointness;
#[cfg(test)]
#[path = "../../../../tests/invariants/overlap_headers.rs"]
mod head_tests;
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
    let (a, mut left_inputs) = inputs.head(facts, &mut context, left)?;
    let (b, mut right_inputs) = inputs.head(facts, &mut context, right)?;
    let mode = crate::sema::coherence_policy::predicate(facts, left.0);
    let possible = match context.equate_predicate_shapes(&facts.types, &a, &b) {
        Ok(()) => {
            inputs.conditions(facts, &mut context, left.1, &mut left_inputs)?;
            inputs.conditions(facts, &mut context, right.1, &mut right_inputs)?;
            !disjointness::has_unsized(facts, &mut context, &inputs)?
                && !inputs.excluded(facts, &mut context, &[&left_inputs, &right_inputs], mode)?
        }
        Err(infer::Error::Mismatch | infer::Error::Occurs) => false,
        Err(error) => return Err(error.to_string()),
    };
    Ok((possible, context.steps))
}

/// The same complete owner conditions also govern inherent method ambiguity.
pub(in crate::sema) fn inherent(
    facts: &mut Declarations,
    left: DefId,
    right: DefId,
) -> Result<(bool, usize), String> {
    let mut context = Context::default();
    let mut inputs = inputs::Inputs::default();
    let (a, mut first) = inputs.target(facts, &mut context, left)?;
    let (b, mut second) = inputs.target(facts, &mut context, right)?;
    let mode = crate::sema::coherence_policy::inherent(facts, left);
    let possible = match context.equate_shape(&facts.types, a, b) {
        Ok(()) => {
            inputs.conditions(facts, &mut context, left, &mut first)?;
            inputs.conditions(facts, &mut context, right, &mut second)?;
            !disjointness::has_unsized(facts, &mut context, &inputs)?
                && !inputs.excluded(facts, &mut context, &[&first, &second], mode)?
        }
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
    let Some((application, _)) = header.0.bound.trait_parts() else {
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
    let (goal, bindings) = inputs.instantiate(facts, &mut context, header)?;
    let candidate = primitive_methods::candidate(facts, &mut context, &goal, false)
        .map_err(|e| e.to_string())?;
    let Candidate::Conditional { prerequisite, .. } = candidate else {
        return Ok((false, context.steps));
    };
    if disjointness::has_unsized(facts, &mut context, &inputs)? {
        return Ok((false, context.steps));
    }
    let mode = crate::sema::coherence_policy::predicate(facts, header.0);
    if mode.negative() {
        inputs::Inputs::freeze(facts, &mut context, &[&bindings])?;
    }
    let possible = selection::may_overlap_with(
        facts,
        &mut context,
        &prerequisite,
        mode,
        |facts, context, proof, sized| inputs.pay_negative(facts, context, proof, sized),
    )?;
    Ok((possible, context.steps))
}

#[cfg(test)]
pub(in crate::sema) fn input_layout() -> usize {
    size_of::<inputs::Inputs>()
}
