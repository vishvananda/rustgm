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

/// The complete parent application has already been proved in this context.
/// Map only relevant declaration equations; their cache carries no permission.
pub(in crate::sema) fn for_projection(
    facts: &mut crate::sema::model::Declarations,
    context: &mut Context,
    application: &Application,
    environment: Option<&Environment>,
) -> Result<Selection, String> {
    let extra = super::results::equalities(
        facts,
        context,
        application.predicate.subject(),
        application.item,
    )?;
    context
        .account_work(extra.steps)
        .map_err(|e| e.to_string())?;
    let extra = extra.clauses.as_deref().unwrap_or(&[]);
    select(&mut facts.types, context, application, environment, extra).map_err(|e| e.to_string())
}

/// The caller separately proves the trait premise. Failed alternatives cannot
/// constrain its inference graph; a selected result retains all input equations.
fn select(
    store: &mut Store,
    context: &mut Context,
    application: &Application,
    environment: Option<&Environment>,
    extra: &[Predicate],
) -> Result<Selection, infer::Error> {
    if !application.arguments.is_empty() {
        return Err(infer::Error::InvalidIdentity);
    }
    let mut selected = None;
    for equality in environment
        .into_iter()
        .flat_map(|environment| environment.projection_candidates(application.item))
        .chain(extra)
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
                let candidate = (predicate, result);
                if selected.is_some_and(|previous| previous != candidate) {
                    return Ok(Selection::Ambiguous);
                }
                selected = Some(candidate);
            }
            // Equal outputs do not identify opaque inputs. This rule supplies
            // no equation for the current parent; keep looking for an exact
            // complete match, or leave its result opaque if none exists.
            Err(infer::Error::Mismatch | infer::Error::Occurs | infer::Error::Normalization) => {}
            Err(error) => return Err(error),
        }
    }
    let Some((predicate, result)) = selected else {
        return Ok(Selection::Absent);
    };
    context.equate_trait_predicate(store, &application.predicate, predicate)?;
    Ok(Selection::Proven(result))
}
