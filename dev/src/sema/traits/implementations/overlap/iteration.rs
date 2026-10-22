//! Closed absence of the canonical blanket prerequisite excludes only that pair.
use super::{Declarations, DefId, Predicate, inputs::Inputs};
use crate::{
    support::iteration::{Blanket, Trait},
    types::{
        ItemId,
        infer::{self, Constructor, Context, predicates::Predicate as Goal},
    },
};
pub(in crate::sema) fn possible(
    facts: &mut Declarations,
    left: (Predicate, DefId),
    right: (Predicate, DefId),
) -> Result<(bool, usize), String> {
    let Some(registry) = facts.iteration.as_deref() else {
        return Ok((true, 0));
    };
    let canonical = registry.canonical;
    let pair = canonical.blankets.iter().find_map(|declaration| {
        if declaration.definition == left.1 {
            Some((*declaration, right))
        } else if declaration.definition == right.1 {
            Some((*declaration, left))
        } else {
            None
        }
    });
    let Some((declaration, source)) = pair else {
        return Ok((true, 0));
    };
    let mut context = Context::default();
    let mut inputs = Inputs::default();
    let goal = inputs.instantiate(facts, &mut context, source)?;
    let subject = match declaration.kind {
        Blanket::Identity => goal.subject(),
        Blanket::MutableReference => match context.head(&facts.types, goal.subject()) {
            Ok(Some(Constructor::Reference { mutable: true, .. })) => context
                .child(&facts.types, goal.subject(), 0)
                .map_err(|e| e.to_string())?,
            Ok(_) | Err(infer::Error::Unresolved | infer::Error::Normalization) => {
                return Ok((true, context.steps));
            }
            Err(error) => return Err(error.to_string()),
        },
    };
    let prerequisite = Goal::trait_application(
        subject,
        ItemId(canonical.get(Trait::Iterator).definition.0),
        Vec::new(),
    );
    let possible = crate::sema::traits::selection::may_overlap(facts, &mut context, &prerequisite)?;
    Ok((possible, context.steps))
}
