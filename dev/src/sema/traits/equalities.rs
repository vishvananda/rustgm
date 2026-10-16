//! Associated equality assumptions match complete inputs without equating their results.
use crate::types::{
    GenericArgs, Kind, Store, TypeId,
    infer::{self, Context, projections::Application},
    parameters::{
        environment::Environment,
        predicates::{Bound, Predicate},
    },
};

pub(in crate::sema) enum Selection {
    Absent,
    Ambiguous,
    Proven(TypeId),
}

/// The caller separately proves the trait premise. Failed alternatives cannot
/// constrain its inference graph; a selected result retains all input equations.
pub(in crate::sema) fn select(
    store: &Store,
    context: &mut Context,
    application: &Application,
    environment: Option<&Environment>,
) -> Result<Selection, infer::Error> {
    if !application.arguments.is_empty() {
        return Err(infer::Error::InvalidIdentity);
    }
    let mut selected = None;
    for equality in environment
        .into_iter()
        .flat_map(|environment| environment.projection_candidates(application.item))
    {
        context.account_work(1)?;
        let (Some(Kind::Projection(candidate)), Bound::ProjectionEqual(result)) =
            (store.kind(equality.ty), equality.bound)
        else {
            return Err(infer::Error::InvalidIdentity);
        };
        if candidate.item != application.item || candidate.arguments != GenericArgs::EMPTY {
            continue;
        }
        let predicate = Predicate {
            ty: candidate.subject,
            bound: Bound::Trait(candidate.trait_application),
        };
        let mut probe = Context::default();
        let goal = probe.import_predicate(store, context, &application.predicate)?;
        let compatible = probe.equate_trait_predicate(store, &goal, predicate);
        context.account_work(probe.steps)?;
        match compatible {
            Ok(()) => {
                if selected.replace((predicate, result)).is_some() {
                    return Ok(Selection::Ambiguous);
                }
            }
            Err(infer::Error::Mismatch | infer::Error::Occurs) => {}
            Err(error) => return Err(error),
        }
    }
    let Some((predicate, result)) = selected else {
        return Ok(Selection::Absent);
    };
    context.equate_trait_predicate(store, &application.predicate, predicate)?;
    Ok(Selection::Proven(result))
}
