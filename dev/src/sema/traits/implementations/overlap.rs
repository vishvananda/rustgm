//! Structural impl disjointness uses simultaneous existential type/const equations.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        infer::{self, Context, Instantiation, TermId, arguments::Argument},
        parameters::{
            ParameterKind,
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};

pub(in crate::sema) fn possible(
    facts: &mut Declarations,
    left: (Predicate, DefId),
    right: (Predicate, DefId),
) -> Result<(bool, usize), infer::Error> {
    let mut context = Context::default();
    let mut sized = Vec::new();
    let a = instantiate(facts, &mut context, left, &mut sized)?;
    let b = instantiate(facts, &mut context, right, &mut sized)?;
    let possible = match context.equate_predicate_shapes(&facts.types, &a, &b) {
        Ok(()) => sized_possible(facts, &mut context, &sized)?,
        Err(infer::Error::Mismatch | infer::Error::Occurs) => false,
        Err(error) => return Err(error),
    };
    Ok((possible, context.steps))
}

fn instantiate(
    facts: &mut Declarations,
    context: &mut Context,
    (predicate, owner): (Predicate, DefId),
    sized: &mut Vec<TermId>,
) -> Result<infer::predicates::Predicate, infer::Error> {
    let ids = facts
        .generics
        .parameter_ids(&facts.types, owner)
        .map_err(|_| infer::Error::InvalidIdentity)?;
    let mut bindings = Vec::with_capacity(ids.len());
    for id in ids {
        let argument = context.parameter_variable(&facts.types, id)?;
        if let Argument::Type(term) = argument
            && matches!(
                facts.types.parameter(id).map(|p| p.kind),
                Some(ParameterKind::Type { sized: true })
            )
        {
            sized.push(term);
        }
        bindings.push((id, argument));
    }
    let mut substitution = Instantiation::new(context, &facts.types, &bindings)?;
    for requirement in facts.generics.predicates_in_scope(owner) {
        if let Clause::Predicate(Predicate {
            ty,
            bound: Bound::Sized | Bound::Copy | Bound::Clone,
        }) = requirement.value
        {
            sized.push(context.instantiate_with(&mut facts.types, ty, &mut substitution)?);
        }
    }
    context.instantiate_predicate(&mut facts.types, predicate, &mut substitution)
}

fn sized_possible(
    facts: &mut Declarations,
    context: &mut Context,
    requirements: &[TermId],
) -> Result<bool, infer::Error> {
    let mut known = std::collections::HashMap::new();
    let mut path = Vec::new();
    for &term in requirements {
        context.account_work(1)?;
        let ty = match context.resolved(&mut facts.types, term) {
            Ok(ty) => ty,
            Err(infer::Error::Unresolved | infer::Error::Normalization) => continue,
            Err(error) => return Err(error),
        };
        if definitely_unsized(&facts.types, context, ty, &mut known, &mut path)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Unknown projected conditions remain possible. A conservative false Sized
/// property is never proof that two implementations are disjoint.
fn definitely_unsized(
    types: &crate::types::Store,
    context: &mut Context,
    mut ty: crate::types::TypeId,
    known: &mut std::collections::HashMap<crate::types::TypeId, bool>,
    path: &mut Vec<crate::types::TypeId>,
) -> Result<bool, infer::Error> {
    use crate::types::{Kind, Primitive, sizedness::Rule};
    let answer = loop {
        context.account_work(1)?;
        if let Some(&answer) = known.get(&ty) {
            break answer;
        }
        path.push(ty);
        match types.kind(ty).ok_or(infer::Error::InvalidIdentity)? {
            Kind::Slice(_) | Kind::Primitive(Primitive::Str) => break true,
            Kind::Tuple(fields) => {
                let Some(&last) = types
                    .list(fields)
                    .ok_or(infer::Error::InvalidIdentity)?
                    .last()
                else {
                    break false;
                };
                ty = last;
            }
            Kind::Adt(item, arguments) => match types.nominal_size_rule(item) {
                Some(Rule::Unsized) => break true,
                Some(Rule::Argument(slot)) => {
                    let Some(crate::types::arguments::Argument::Type(value)) = types
                        .arguments(arguments)
                        .and_then(|mut arguments| arguments.nth(slot as usize))
                    else {
                        return Err(infer::Error::InvalidIdentity);
                    };
                    ty = value;
                }
                None | Some(Rule::Sized | Rule::Projection(_)) => break false,
            },
            _ => break false,
        }
    };
    for ty in path.drain(..) {
        known.insert(ty, answer);
    }
    Ok(answer)
}
