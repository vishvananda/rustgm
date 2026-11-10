//! Lookup retains each nonempty binder and rejects closed, impossible scoped debts.
use super::applicability::inference;
use crate::{
    sema::{
        model::Declarations,
        traits::{implementations::Availability, selection},
    },
    types::{
        Store,
        infer::{self, Context, Instantiation},
        lifetime_proof::Obligations,
        outlives::Queries,
        parameters::{clauses::Clause, predicates::Bound},
        region_graph::Error as RegionError,
        relations::Constraints,
    },
};

pub(super) struct Goal {
    pub predicate: infer::predicates::Predicate,
    pub bound_regions: Option<u32>,
}
impl Goal {
    pub fn ordinary(predicate: infer::predicates::Predicate) -> Self {
        Self {
            predicate,
            bound_regions: None,
        }
    }
    /// A possibility check never exports evidence or supplies caller assumptions.
    pub fn possible(
        &self,
        facts: &mut Declarations,
        context: &mut Context,
        proof: &Obligations,
    ) -> Result<bool, &'static str> {
        if self.bound_regions.is_none() {
            return Ok(true);
        }
        let predicate = match context.resolved_predicate(&mut facts.types, &self.predicate) {
            Ok(predicate) => predicate,
            Err(infer::Error::Unresolved | infer::Error::Normalization) => return Ok(true),
            Err(error) => return Err(inference(error)),
        };
        if predicate.contains_parameters(&facts.types)?
            || predicate.has_regions_outside(&facts.types, 1)?
            || predicate.contains_opaques(&facts.types)?
        {
            return Ok(true);
        }
        let payload = proof.allocation_bytes().saturating_add(1);
        context
            .account_work(payload.saturating_mul(4))
            .map_err(inference)?;
        // Caller-dependent debts remain possible here. The selected callable
        // must pay them under its actual declared inputs, including f01's check.
        if !proof.self_contained(&facts.types)? {
            return Ok(true);
        }
        let limit = payload.saturating_mul(64).clamp(1024, 1_000_000);
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        let mut graph = Constraints::new(&facts.types, &facts.lifetimes.variances, 0, limit)
            .map_err(|_| "method universal lifetime graph exhausted")?;
        let required = proof.require(&mut graph, &mut queries);
        context.account_work(queries.steps()).map_err(inference)?;
        required.map_err(|error| {
            crate::sema::census::fold_cause(
                &error,
                "method universal lifetime work exhausted",
                "invalid method universal lifetime proof",
            )
        })?;
        let answer = graph.check_aliases_counted();
        context.account_work(answer.steps).map_err(inference)?;
        match answer.result {
            Ok(()) => Ok(true),
            Err(
                RegionError::Unsatisfied { .. }
                | RegionError::Verification { .. }
                | RegionError::Membership { .. },
            ) => Ok(false),
            Err(RegionError::Budget) => Err("method universal lifetime work exhausted"),
            Err(RegionError::Identity | RegionError::UnsupportedOutput) => {
                Err("invalid method universal lifetime proof")
            }
        }
    }
}

pub(super) fn quantified(
    store: &mut Store,
    context: &mut Context,
    clause: Clause,
    substitution: &mut Instantiation,
) -> Result<Option<Goal>, &'static str> {
    let Clause::ForAll(id) = clause else {
        return Ok(None);
    };
    let scoped = store
        .quantified(id)
        .ok_or("invalid quantified method condition")?;
    if scoped.bound_regions == 0 {
        return Ok(None);
    }
    if scoped.predicate.has_regions_outside(store, 1)?
        || matches!(scoped.predicate.bound, Bound::Outlives(_))
    {
        return Err("quantified clause requires a scoped consumer");
    }
    let clause = crate::sema::traits::effects::in_context(
        store,
        clause,
        crate::support::constness::Constness::Ordinary,
    )?;
    let Clause::ForAll(id) = clause else {
        return Err("method condition lost its binder");
    };
    let scoped = store
        .quantified(id)
        .ok_or("invalid ordinary quantified method condition")?;
    Ok(Some(Goal {
        predicate: context
            .instantiate_predicate(store, scoped.predicate, substitution)
            .map_err(inference)?,
        bound_regions: Some(scoped.bound_regions),
    }))
}

pub(super) fn select_available(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: selection::Caller<'_>,
    availability: Availability,
) -> Result<selection::Selection, String> {
    let Some(count) = goal.bound_regions else {
        return selection::select_available(facts, context, &goal.predicate, caller, availability);
    };
    let assumptions = caller.assumptions;
    let environment = caller.environment;
    let first_region = caller.first_region;
    let retain_sized = caller.retain_sized;
    let nesting = caller.nesting;
    let placeholders = caller.placeholders;
    let progress = selection::begin_quantified_borrowed(
        facts,
        context,
        &goal.predicate,
        caller,
        (availability, count),
    )?;
    match progress.retain(|| selection::OwnedCaller {
        assumptions: std::sync::Arc::new(assumptions.clone()),
        environment: environment.cloned(),
        first_region,
        retain_sized,
        nesting,
        placeholders: placeholders.cloned(),
    }) {
        selection::Progress::Ready(selected) => Ok(selected),
        selection::Progress::Pending(..) => {
            Err("method condition requires pending declarations".into())
        }
        selection::Progress::Inferred { .. } => {
            Err("non-refining applicability returned partial inputs".into())
        }
    }
}
