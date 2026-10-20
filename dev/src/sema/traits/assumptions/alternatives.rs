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
        Err(infer::Error::Normalization) => {
            let head = match context.result_identity(store, goal.subject())? {
                Some(crate::types::outlives::ResultId::Associated(item)) => Head::Projection(item),
                Some(crate::types::outlives::ResultId::Opaque(id)) => Head::Opaque(id),
                None => return Err(infer::Error::InvalidIdentity),
            };
            Ok(Some(Subject::Rigid(head)))
        }
        Err(error) => Err(error),
    }
}

pub(super) fn choose<C, E>(
    store: &mut Store,
    context: &mut Context,
    goal: &Goal,
    candidates: impl Iterator<Item = C>,
    mut prepare: impl FnMut(&mut Store, &mut Context, C) -> Result<(Predicate, E), infer::Error>,
    mut equivalent: impl FnMut(&mut Store, &mut Context, &E, &E) -> Result<bool, infer::Error>,
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
                match &selected {
                    Some((previous, prior))
                        if *previous != predicate
                            && !equivalent(store, context, prior, &evidence)? =>
                    {
                        return Ok(Selection::Deferred);
                    }
                    // Opening equivalent clauses may erase only unused local slots.
                    // Keep one actual premise and its owned instantiation interval.
                    Some(_) => {}
                    None => selected = Some((predicate, evidence)),
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
