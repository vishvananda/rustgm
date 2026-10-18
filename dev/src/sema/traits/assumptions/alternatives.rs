//! Every assumption alternative uses an isolated trial before committing one match.
use super::{Context, Goal, Head, Selection, Store, Subject, infer};
use crate::types::parameters::predicates::Predicate;

pub(super) fn subject(
    store: &Store,
    context: &mut Context,
    goal: &Goal,
) -> Result<Option<Subject>, infer::Error> {
    match Subject::query(store, context, goal.subject()) {
        Ok(subject) => Ok(subject),
        Err(infer::Error::Normalization) => Ok(Some(Subject::Rigid(Head::Projection(
            context
                .projection_item(store, goal.subject())?
                .ok_or(infer::Error::InvalidIdentity)?,
        )))),
        Err(error) => Err(error),
    }
}

pub(super) fn choose<C, E>(
    store: &mut Store,
    context: &mut Context,
    goal: &Goal,
    candidates: impl Iterator<Item = C>,
    mut prepare: impl FnMut(&mut Store, &mut Context, C) -> Result<(Predicate, E), infer::Error>,
) -> Result<Selection<E>, infer::Error> {
    let mut selected = None;
    let mut needs_normalization = false;
    for candidate in candidates {
        let (predicate, evidence) = prepare(store, context, candidate)?;
        let mut probe = Context::default();
        let imported = probe.import_predicate(store, context, goal)?;
        let result = probe.equate_trait_predicate(store, &imported, predicate);
        context.account_work(probe.steps)?;
        match result {
            Ok(()) => {
                if selected.replace((predicate, evidence)).is_some() {
                    return Ok(Selection::Deferred);
                }
            }
            Err(infer::Error::Mismatch | infer::Error::Occurs) => {}
            Err(infer::Error::Normalization) => needs_normalization = true,
            Err(infer::Error::Unresolved) => return Ok(Selection::Deferred),
            Err(error) => return Err(error),
        }
    }
    if let Some((predicate, evidence)) = selected {
        context.equate_trait_predicate(store, goal, predicate)?;
        Ok(Selection::Proven(evidence))
    } else if needs_normalization {
        Err(infer::Error::Normalization)
    } else {
        Ok(Selection::Absent)
    }
}
