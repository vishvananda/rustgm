//! Common declared Item equations guide inference without selecting an implementation.
use crate::{
    sema::{
        model::Declarations,
        traits::{
            assumptions::Assumptions,
            heads::Subject,
            implementations::{Availability, Candidates},
        },
    },
    types::{
        ItemId,
        infer::{Constructor, Context, Error, TermId, predicates::Predicate},
    },
};

pub(in crate::sema) fn range_element(
    facts: &mut Declarations,
    context: &mut Context,
    subject: TermId,
    assumptions: Option<&Assumptions>,
) -> Result<Option<TermId>, String> {
    let Some(registry) = facts
        .iteration
        .as_deref()
        .and_then(|r| r.ranges.as_deref())
        .filter(|r| r.complete)
    else {
        return Ok(None);
    };
    let nominal = ItemId(registry.canonical.nominal.0);
    if !matches!(context.head(&facts.types, subject), Ok(Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _))) if item == nominal)
    {
        return Ok(None);
    }
    let iterator = facts
        .iteration
        .as_deref()
        .ok_or("missing iteration registry")?
        .canonical
        .get(crate::support::iteration::Trait::Iterator);
    let item = ItemId(iterator.definition.0);
    let head = Subject::query(&facts.types, context, subject)
        .map_err(|e| e.to_string())?
        .ok_or("missing Range inference head")?;
    if assumptions.is_some_and(|a| a.may_have_subject(item, head)) {
        return Ok(None);
    }
    let goal = Predicate::trait_application(subject, item, Vec::new());
    let Some(inventory) = facts.generics.implementations.as_mut() else {
        return Ok(None);
    };
    let Candidates::Ready(owners) = inventory
        .candidates(&mut facts.types, context, &goal, Availability::Validated)
        .map_err(|e| e.to_string())?
    else {
        return Ok(None);
    };
    if owners.is_empty() {
        return Ok(None);
    }
    let mut applicable = false;
    for owner in owners {
        context.account_work(1).map_err(|e| e.to_string())?;
        let Some(original) = facts
            .iteration
            .as_deref()
            .and_then(|r| r.ranges.as_deref())
            .and_then(|r| {
                r.originals
                    .iter()
                    .find(|o| o.declaration.definition == owner)
            })
            .copied()
        else {
            // An additional possible header must keep the ordinary unresolved query.
            return Ok(None);
        };
        original.validate(facts)?;
        let mut probe = Context::default();
        let actual = probe
            .import(&facts.types, context, subject)
            .map_err(|e| e.to_string())?;
        let formal = probe
            .known(&facts.types, original.subject)
            .map_err(|e| e.to_string())?;
        let compatible = probe.equate(&facts.types, actual, formal);
        context
            .account_work(probe.steps)
            .map_err(|e| e.to_string())?;
        match compatible {
            Ok(()) => applicable = true,
            Err(Error::Mismatch | Error::Occurs) => {}
            Err(Error::Normalization | Error::Unresolved) => return Ok(None),
            Err(error) => return Err(error.to_string()),
        }
    }
    applicable
        .then(|| {
            context
                .child(&facts.types, subject, 0)
                .map_err(|e| e.to_string())
        })
        .transpose()
}

#[cfg(test)]
#[path = "../../../../tests/invariants/progression_inference.rs"]
mod tests;
