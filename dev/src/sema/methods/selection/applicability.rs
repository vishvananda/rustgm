//! Bind each candidate once; applicability never authorizes a completed body.
use super::{Caller, Receiver};
use crate::{
    resolve::model::DefId,
    sema::{
        model::Declarations,
        traits::{
            assumptions::Assumptions,
            selection::{self, Selection},
        },
    },
    types::{
        infer::{self, Context, Instantiation, TermId, arguments::Argument, properties::Query},
        parameters::{
            ParameterKind,
            clauses::Clause,
            environment::Environment,
            predicates::{Bound, Predicate},
        },
    },
};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../../../tests/invariants/supertrait_lookups.rs"]
mod tests;

pub(super) struct Conditions {
    pub builtins: Vec<(TermId, Bound)>,
    pub traits: Vec<super::lookup_goals::Goal>,
}

pub(in crate::sema::methods) fn applicable(
    facts: &mut Declarations,
    context: &mut Context,
    prefix: TermId,
    owner: DefId,
    receiver: Option<Receiver>,
    caller: Caller<'_>,
) -> Result<bool, &'static str> {
    let Some(mut conditions) = prepare(facts, context, prefix, owner, receiver, caller)? else {
        return Ok(false);
    };
    loop {
        if !trait_requirements(facts, context, &conditions.traits, caller)? {
            return Ok(false);
        }
        let before = conditions.traits.len();
        if !builtin_requirements(facts, context, &mut conditions, caller.environment)? {
            return Ok(false);
        }
        if conditions.traits.len() == before {
            return Ok(true);
        }
    }
}

pub(super) fn prepare(
    facts: &mut Declarations,
    context: &mut Context,
    prefix: TermId,
    owner: DefId,
    receiver: Option<Receiver>,
    caller: Caller<'_>,
) -> Result<Option<Conditions>, &'static str> {
    let target = facts
        .associated
        .schema(owner)
        .ok_or("missing candidate schema")?
        .self_type;
    let subject = target;
    let target = receiver.as_ref().map_or(target, |r| r.ty);
    // A projected implementation header is normalized as rustc's probe does:
    // each associated-result leaf becomes a projection-equality condition.
    let projected = |ty| facts.types.contains_projections(ty) == Some(true);
    let (subject_projected, target_projected) = (projected(subject), projected(target));
    let ids = if facts.associated.trait_owner(owner).is_some()
        || facts.associated.operation(owner).is_some()
        || facts.associated.comparison(owner).is_some()
        || facts
            .iteration
            .as_deref()
            .is_some_and(|r| r.owns_trait(owner))
        || facts
            .cloning
            .as_deref()
            .is_some_and(|r| r.owns_trait(owner))
    {
        facts.generics.callable_parameters(&facts.types, owner)?.1
    } else {
        facts.generics.parameter_ids(&facts.types, owner)?
    };
    let mut bindings = Vec::with_capacity(ids.len());
    let mut obligations = Vec::new();
    for id in ids {
        let argument = context
            .parameter_variable(&facts.types, id)
            .map_err(inference)?;
        if let Argument::Type(term) = argument
            && matches!(
                facts.types.parameter(id).map(|p| p.kind),
                Some(ParameterKind::Type { sized: true })
            )
        {
            obligations.push((term, Bound::Sized));
        }
        bindings.push((id, argument));
    }
    let mut substitution =
        Instantiation::new(context, &facts.types, &bindings).map_err(inference)?;
    let target = context
        .instantiate_with(&mut facts.types, target, &mut substitution)
        .map_err(inference)?;
    let mut header = Vec::new();
    if let Some(actual) = receiver.and_then(|r| r.subject) {
        let subject = context
            .instantiate_with(&mut facts.types, subject, &mut substitution)
            .map_err(inference)?;
        if !relate_header(
            facts,
            context,
            subject,
            actual,
            subject_projected,
            &mut header,
        )? {
            return Ok(None);
        }
    }
    if !relate_header(
        facts,
        context,
        target,
        prefix,
        target_projected,
        &mut header,
    )? {
        return Ok(None);
    }
    let Some(mut conditions) = declaration_conditions(
        facts,
        context,
        owner,
        &mut substitution,
        caller,
        Conditions {
            builtins: obligations,
            traits: Vec::new(),
        },
    )?
    else {
        return Ok(None);
    };
    conditions
        .traits
        .extend(header.into_iter().map(super::lookup_goals::Goal::ordinary));
    if !builtin_requirements(facts, context, &mut conditions, caller.environment)? {
        return Ok(None);
    }
    Ok(Some(conditions))
}

/// Relate an instantiated candidate header to its actual shape. A projected
/// header postpones each associated-result leaf as a condition for selection,
/// rather than comparing an unnormalized projection structurally.
fn relate_header(
    facts: &Declarations,
    context: &mut Context,
    header: TermId,
    actual: TermId,
    projected: bool,
    goals: &mut Vec<infer::predicates::Predicate>,
) -> Result<bool, &'static str> {
    let related = if projected {
        context
            .equate_shape_goals(&facts.types, header, actual)
            .map(|postponed| goals.extend(postponed))
    } else {
        context.equate_shape(&facts.types, header, actual)
    };
    match related {
        Ok(()) => Ok(true),
        Err(infer::Error::Mismatch | infer::Error::Occurs) => Ok(false),
        Err(error) => Err(inference(error)),
    }
}

fn declaration_conditions(
    facts: &mut Declarations,
    context: &mut Context,
    owner: DefId,
    substitution: &mut Instantiation,
    caller: Caller<'_>,
    mut conditions: Conditions,
) -> Result<Option<Conditions>, &'static str> {
    let mut mapped = Vec::new();
    let mut scope = ScopeConditions::new(&facts.generics, owner);
    for predicate in facts.generics.predicates_in_scope(owner) {
        if let Clause::UnstableFeature(id) = predicate.value {
            let work = std::cell::Cell::new(0_usize);
            let proven =
                facts.prove_feature(id, caller.environment.map(Arc::as_ref), &|steps, _| {
                    work.set(work.get().saturating_add(steps));
                    Ok(())
                });
            facts.body_work.canonical_steps =
                facts.body_work.canonical_steps.saturating_add(work.get());
            if !proven? {
                return Ok(None);
            }
            continue;
        }
        let clause = predicate.value;
        if let Some(goal) =
            super::lookup_goals::quantified(&mut facts.types, context, clause, substitution)?
        {
            conditions.traits.push(goal);
            continue;
        }
        let Some(predicate) = scope.condition(&facts.types, &mut facts.body_work, clause)? else {
            continue;
        };
        // Applicability checks the abstract marker and Sized requirements. The
        // chosen callable retains and proves its complete canonical trait goal.
        if facts
            .cloning
            .as_deref()
            .is_some_and(|r| r.owns_trait(owner))
            && facts.generics.trait_self_bounds.get(&owner) == Some(&predicate)
        {
            continue;
        }
        if matches!(
            predicate.bound,
            Bound::Trait(_)
                | Bound::ConstTrait(_)
                | Bound::MaybeConstTrait(_)
                | Bound::ProjectionEqual(_)
                | Bound::Auto(_)
        ) {
            if let Some(predicate) = scope.ordinary_trait_condition(
                &mut facts.types,
                &mut facts.body_work,
                clause,
                predicate,
                &mut mapped,
            )? {
                conditions.traits.push(super::lookup_goals::Goal::ordinary(
                    context
                        .instantiate_predicate(&mut facts.types, predicate, substitution)
                        .map_err(inference)?,
                ));
            }
            continue;
        }
        if matches!(predicate.bound, Bound::Outlives(_)) {
            continue;
        }
        let term = context
            .instantiate_with(&mut facts.types, predicate.ty, substitution)
            .map_err(inference)?;
        conditions.builtins.push((term, predicate.bound));
    }
    Ok(Some(conditions))
}

/// Reads one lookup's scope clauses as its conditions. Elaboration lifts a
/// sugar bound's consequences into its binder, so a condition read from an
/// empty binder can equal a flat clause of the same scope, such as the
/// callable's implied `MetaSized`. Such a condition is skipped; the flat clause
/// keeps its own position.
struct ScopeConditions<'a> {
    generics: &'a crate::sema::generics::Generics,
    owner: DefId,
    /// The scope's flat clauses, collected by its first read from an empty binder.
    flat: Option<crate::support::hash::HashSet<Predicate>>,
}

impl<'a> ScopeConditions<'a> {
    const fn new(generics: &'a crate::sema::generics::Generics, owner: DefId) -> Self {
        Self {
            generics,
            owner,
            flat: None,
        }
    }

    /// Erase lookup-only constness and retain the first conditional/plain twin.
    fn ordinary_trait_condition(
        &self,
        store: &mut crate::types::Store,
        work: &mut crate::sema::model::BodyWork,
        clause: Clause,
        predicate: Predicate,
        mapped: &mut Vec<Predicate>,
    ) -> Result<Option<Predicate>, &'static str> {
        // Method lookup ignores constness, as rustc 1.98's probe does: a
        // `[const]` condition is its ordinary bound here, and the chosen
        // callable proves its complete clause under the use's effect.
        // prepare is the only caller, for the type-qualified (paths.rs)
        // and method-call (receiver.rs) lookups.
        let written = predicate;
        let predicate = crate::sema::traits::effects::in_context(
            store,
            Clause::Predicate(written),
            crate::support::constness::Constness::Ordinary,
        )?
        .ordinary()?;
        // A trait's `[const]` supertrait is also in scope as its self
        // bound's elaborated ordinary consequence. Of a `[const]` condition
        // and an equal ordinary one, lookup keeps the first in scope order,
        // reading each clause as its condition; no other duplicate is
        // compared. Each visit is charged.
        let conditional = matches!(written.bound, Bound::MaybeConstTrait(_));
        let mut steps = 0_usize;
        let twin = if conditional {
            self.generics
                .predicates_in_scope(self.owner)
                .take_while(|earlier| {
                    steps += 1;
                    earlier.value != clause
                })
                .any(|earlier| lookup_condition(store, earlier.value) == Ok(predicate))
        } else if matches!(predicate.bound, Bound::Trait(_)) {
            steps = mapped.len();
            mapped.contains(&predicate)
        } else {
            false
        };
        work.canonical_steps = work.canonical_steps.saturating_add(steps);
        #[cfg(test)]
        crate::sema::copy_trace::record(|| {
            let (size, maybe) = self.generics.predicates_in_scope(self.owner).fold(
                (0_usize, 0_usize),
                |(size, maybe), located| {
                    let bound = lookup_condition(store, located.value).map(|p| p.bound);
                    let found = matches!(bound, Ok(Bound::MaybeConstTrait(_)));
                    (size + 1, maybe + usize::from(found))
                },
            );
            format!(
                "supertrait-lookup scope={size} conditional={maybe} charge={steps} kept={} {predicate:?}",
                !twin
            )
        });
        if twin {
            return Ok(None);
        }
        if conditional {
            mapped.push(predicate);
        }
        Ok(Some(predicate))
    }

    /// The clause's lookup condition, or `None` when it is read from an empty
    /// binder and equals a flat clause. The first such read charges a step per
    /// scope clause, and each read charges one more.
    fn condition(
        &mut self,
        store: &crate::types::Store,
        work: &mut crate::sema::model::BodyWork,
        clause: Clause,
    ) -> Result<Option<Predicate>, &'static str> {
        let predicate = lookup_condition(store, clause)?;
        if !matches!(clause, Clause::ForAll(_)) {
            return Ok(Some(predicate));
        }
        let (generics, owner) = (self.generics, self.owner);
        let mut steps = 1_usize;
        let flat = self.flat.get_or_insert_with(|| {
            generics
                .predicates_in_scope(owner)
                .filter_map(|located| {
                    steps += 1;
                    match located.value {
                        Clause::Predicate(ordinary) => Some(ordinary),
                        Clause::ForAll(_) | Clause::UnstableFeature(_) => None,
                    }
                })
                .collect()
        });
        work.canonical_steps = work.canonical_steps.saturating_add(steps);
        let duplicate = flat.contains(&predicate);
        #[cfg(test)]
        crate::sema::copy_trace::record(|| {
            let scope = generics.predicates_in_scope(owner).count();
            format!(
                "empty-binder-lookup scope={scope} charge={steps} kept={} {predicate:?}",
                !duplicate
            )
        });
        Ok((!duplicate).then_some(predicate))
    }
}

/// Callable sugar such as `F: FnMut(T) -> U` publishes its bound and output
/// equality inside a binder even when it names no lifetime. An empty binder
/// whose body captures nothing outside it quantifies nothing, so lookup reads
/// it as its body, the same predicate. A binder that quantifies a lifetime, or
/// encloses an outer capture, still requires a scoped consumer.
fn lookup_condition(
    store: &crate::types::Store,
    clause: Clause,
) -> Result<Predicate, &'static str> {
    if let Clause::ForAll(id) = clause {
        let scoped = store
            .quantified(id)
            .ok_or("invalid quantified method condition")?;
        if scoped.bound_regions == 0 && scoped.predicate.escaping_binders(store)? == 0 {
            return Ok(scoped.predicate);
        }
    }
    clause.ordinary()
}

pub(super) fn builtin_requirements(
    facts: &mut Declarations,
    context: &mut Context,
    conditions: &mut Conditions,
    environment: Option<&Arc<Environment>>,
) -> Result<bool, &'static str> {
    if conditions.builtins.is_empty() {
        return Ok(true);
    }
    let environment = if let Some(environment) = environment {
        Arc::clone(environment)
    } else {
        Arc::new(Environment::new(&facts.types, &[])?)
    };
    let mut properties = Query::new(environment);
    let mut result = Ok(true);
    conditions.builtins.retain(|&(term, bound)| {
        if result != Ok(true) {
            return true;
        }
        let value = match properties.try_demand(
            context,
            &mut facts.types,
            &facts.aggregates,
            &facts.derives,
            term,
        ) {
            Ok(Some(value)) => value,
            Err(error) => {
                result = Err(error);
                return true;
            }
            Ok(None) => {
                // The structural filter cannot read a projected property. Reuse
                // ordinary non-refining trait selection, which owns normalization,
                // caller premises and pending declarations. No probe evidence is
                // published; the selected callable still proves its complete use.
                conditions.traits.push(super::lookup_goals::Goal::ordinary(
                    infer::predicates::Predicate::builtin(term, bound),
                ));
                return false;
            }
        };
        result = match bound {
            Bound::Sized => Ok(value.sized),
            Bound::Copy => Ok(value.abilities.copy),
            Bound::Clone => Ok(value.abilities.clone_),
            Bound::Outlives(_) => Err("unexpected candidate region predicate"),
            Bound::Trait(_) | Bound::ConstTrait(_) | Bound::MaybeConstTrait(_) | Bound::Auto(_) => {
                Err("user trait candidate obligations require trait selection")
            }
            Bound::ProjectionEqual(_) => Err("projection equality requires a normalization proof"),
        };
        // A possibility read before trait equations settle is not final.
        // Retain this structural condition for the post-selection check.
        true
    });
    result
}
fn trait_requirements(
    facts: &mut Declarations,
    context: &mut Context,
    goals: &[super::lookup_goals::Goal],
    caller: Caller<'_>,
) -> Result<bool, &'static str> {
    if goals.is_empty() {
        return Ok(true);
    }
    let empty = Assumptions::default();
    let assumptions = caller.traits.unwrap_or(&empty);
    let mut first_region = facts
        .body_regions
        .probe_frontier()
        .map_err(|_| "method trait region identity space exhausted")?;
    let mut pending: Vec<_> = goals.iter().collect();
    while !pending.is_empty() {
        let before = pending.len();
        let mut next = Vec::new();
        for goal in pending {
            match super::lookup_goals::select_available(
                facts,
                context,
                goal,
                selection::Caller {
                    assumptions,
                    environment: caller.environment,
                    first_region,

                    retain_sized: false,
                    nesting: selection::Nesting::ROOT,
                    placeholders: None,
                },
                caller.availability,
            )
            .map_err(|error| {
                crate::sema::census::fold_cause(
                    &error,
                    "method trait selection work exhausted",
                    "invalid method trait selection",
                )
            })? {
                Selection::Refuted(refutation) => {
                    refutation.reject()?;
                    return Ok(false);
                }
                Selection::Absent => return Ok(false),
                Selection::Deferred => next.push(goal),
                Selection::Proven(proof) => {
                    first_region = first_region.max(proof.proof.regions.end);
                    if !goal.possible(facts, context, &proof.proof)? {
                        return Ok(false);
                    }
                }
            }
        }
        if next.len() == before {
            // A candidate is possible; the selected ordinary callable must prove
            // its bounds after all arguments and result constraints are known.
            return Ok(true);
        }
        pending = next;
    }
    Ok(true)
}
pub(in crate::sema::methods) fn inference(error: infer::Error) -> &'static str {
    match error {
        infer::Error::Budget => "method candidate work budget exceeded",
        infer::Error::Overflow => "method candidate arena exceeds 32 bits",
        _ => "invalid method candidate inference graph",
    }
}
