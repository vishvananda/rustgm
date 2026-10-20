//! Structural impl disjointness uses simultaneous existential type/const equations.
mod disjointness;
mod inputs;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        infer::{self, Context},
        parameters::predicates::Predicate,
    },
};

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
