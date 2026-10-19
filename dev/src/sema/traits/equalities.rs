//! Associated equality assumptions match complete inputs without equating their results.
use crate::types::{
    GenericArgs, Kind, Store, TypeId,
    infer::{self, Context, projections::Application},
    parameters::{
        clauses::Clause,
        environment::Environment,
        predicates::{Bound, Predicate},
    },
};

#[cfg(test)]
#[path = "../../../tests/invariants/equality_scopes.rs"]
mod tests;

use super::assumptions::{Openings, ScopedEvidence};

pub(in crate::sema) struct Evidence {
    pub ty: TypeId,
    pub premise: ScopedEvidence,
}
impl Evidence {
    /// Publish only the selected equality's fresh existential interval.
    pub fn append_to(
        self,
        proof: &mut super::obligations::Obligations,
    ) -> Result<TypeId, &'static str> {
        let mut owned = super::obligations::Obligations::new(self.premise.regions.start);
        owned.regions.end = self.premise.regions.end;
        proof.extend(owned)?;
        Ok(self.ty)
    }
}

pub(in crate::sema) enum Selection {
    Absent,
    Ambiguous,
    Proven(Evidence),
}

/// The complete parent application has already been proved in this context.
/// Map only relevant declaration equations; their cache carries no permission.
pub(in crate::sema) fn for_projection(
    facts: &mut crate::sema::model::Declarations,
    context: &mut Context,
    application: &Application,
    environment: Option<&Environment>,
    first: u32,
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
    select(
        &mut facts.types,
        context,
        application,
        environment,
        extra,
        first,
    )
    .map_err(|e| e.to_string())
}

/// The caller separately proves the trait premise. Failed alternatives cannot
/// constrain its inference graph; a selected result retains all input equations.
fn select(
    store: &mut Store,
    context: &mut Context,
    application: &Application,
    environment: Option<&Environment>,
    extra: &[Clause],
    first: u32,
) -> Result<Selection, infer::Error> {
    if !application.arguments.is_empty() {
        return Err(infer::Error::InvalidIdentity);
    }
    let mut selected: Option<(Predicate, TypeId, ScopedEvidence)> = None;
    let mut openings = Openings::default();
    for clause in environment
        .into_iter()
        .flat_map(|environment| environment.projection_candidates(application.item))
        .chain(extra.iter().copied())
    {
        context.account_work(1)?;
        let (equality, evidence) = openings.prepare(store, context, clause, first)?;
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
                if let Some(previous) = &selected
                    && (previous.0, previous.1) != candidate
                    && !previous.2.same_clause(store, context, &evidence)?
                {
                    return Ok(Selection::Ambiguous);
                }
                if selected.is_none() {
                    selected = Some((predicate, result, evidence));
                }
            }
            // Equal outputs do not identify opaque inputs. This rule supplies
            // no equation for the current parent; keep looking for an exact
            // complete match, or leave its result opaque if none exists.
            Err(infer::Error::Mismatch | infer::Error::Occurs | infer::Error::Normalization) => {}
            Err(error) => return Err(error),
        }
    }
    let Some((predicate, result, premise)) = selected else {
        return Ok(Selection::Absent);
    };
    context.equate_trait_predicate(store, &application.predicate, predicate)?;
    Ok(Selection::Proven(Evidence {
        ty: result,
        premise,
    }))
}
