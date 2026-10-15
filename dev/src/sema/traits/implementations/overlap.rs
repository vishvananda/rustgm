//! Structural impl disjointness uses simultaneous existential type/const equations.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        infer::{self, Context, Instantiation},
        parameters::predicates::Predicate,
    },
};

pub(in crate::sema) fn possible(
    facts: &mut Declarations,
    left: (Predicate, DefId),
    right: (Predicate, DefId),
) -> Result<(bool, usize), infer::Error> {
    let mut context = Context::default();
    let a = instantiate(facts, &mut context, left)?;
    let b = instantiate(facts, &mut context, right)?;
    let possible = match context.equate_predicate_shapes(&facts.types, &a, &b) {
        Ok(()) => true,
        Err(infer::Error::Mismatch | infer::Error::Occurs) => false,
        Err(error) => return Err(error),
    };
    Ok((possible, context.steps))
}

fn instantiate(
    facts: &mut Declarations,
    context: &mut Context,
    (predicate, owner): (Predicate, DefId),
) -> Result<infer::predicates::Predicate, infer::Error> {
    let ids = facts
        .generics
        .parameter_ids(&facts.types, owner)
        .map_err(|_| infer::Error::InvalidIdentity)?;
    let mut bindings = Vec::with_capacity(ids.len());
    for id in ids {
        bindings.push((id, context.parameter_variable(&facts.types, id)?));
    }
    let mut substitution = Instantiation::new(context, &facts.types, &bindings)?;
    context.instantiate_predicate(&mut facts.types, predicate, &mut substitution)
}
