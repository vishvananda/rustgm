//! Ordinary declaration facts acquire body-local mappings only at original demand.
use super::{Canonical, Declarations, Error, Graph, error, s};
use crate::{
    resolve::{crates::ReservationWork, expansion::Report, model::DefId},
    sema::crates::{
        ImportedOwners,
        typed_headers::{
            staged_headers::rebuilt_environments::ContractParts,
            unpaid_provided::{Observed, OrdinaryDomain},
        },
    },
    support::retained::Allocation,
    types::parameters::environment::{Environment, Retained},
};
use std::{rc::Rc, sync::Arc};

pub(crate) fn prepare(
    report: &Report,
    facts: &mut Declarations,
    definition: DefId,
    extra_work: usize,
    extra_bytes: usize,
) -> Result<(), Error> {
    let Some((initializer, environment)) = prepare_lazy(
        report,
        facts,
        extra_work,
        extra_bytes,
        |graph, facts, work, allocation| {
            let headers = facts
                .crates
                .as_deref()
                .and_then(|linkage| linkage.originals.as_deref())
                .ok_or_else(|| error("ordinary original lost its current headers"))?;
            let mut steps = work.steps;
            let ready = if headers.get(definition).is_some() {
                graph
                    .function_ready(&graph.origins, facts, headers, definition, &mut steps)
                    .map(|()| None)
            } else {
                graph
                    .initializer_header(&graph.origins, facts, definition, &mut steps)
                    .and_then(|initializer| {
                        initializer
                            .map(Some)
                            .ok_or("ordinary original lost its actual declaration header")
                    })
            };
            work.charge(steps.saturating_sub(work.steps))?;
            let initializer = ready.map_err(error)?;
            let Canonical::Lazy(lazy) = &graph.canonical else {
                return Err(error("ordinary original lost its lazy owner"));
            };
            let domain = lazy.prepare_body(graph, facts, definition, initializer, work)?;
            let environment = environment(graph, facts, definition, &domain, work, allocation)?;
            domain.publish_environment(&environment, work)?;
            Ok((initializer, environment))
        },
    )?
    else {
        return Ok(());
    };
    if initializer.is_none() {
        let headers = facts
            .crates
            .as_deref_mut()
            .and_then(|linkage| linkage.originals.as_deref_mut())
            .ok_or_else(|| error("ordinary original lost its actual accounting owner"))?;
        headers
            .entries
            .get_mut(&definition)
            .ok_or_else(|| error("ordinary original lost its exact publication header"))?
            .environment = Some(environment);
    }
    Ok(())
}

pub(crate) fn prepare_closure(
    report: &Report,
    facts: &mut Declarations,
    identity: crate::types::closure::ClosureId,
    extra_work: usize,
    extra_bytes: usize,
) -> Result<(), Error> {
    let _ = prepare_lazy(
        report,
        facts,
        extra_work,
        extra_bytes,
        |graph, facts, work, allocation| {
            let mut steps = work.steps;
            let closure = graph.closure_header(&graph.origins, facts, identity, &mut steps);
            work.charge(steps.saturating_sub(work.steps))?;
            let closure = closure
                .map_err(error)?
                .ok_or_else(|| error("ordinary closure lost its actual closure header"))?;
            let Canonical::Lazy(lazy) = &graph.canonical else {
                return Err(error("ordinary closure lost its lazy owner"));
            };
            let domain = lazy.prepare_closure_body(graph, facts, closure, work)?;
            let environment =
                closure_environment(graph, facts, closure, &domain, work, allocation)?;
            domain.publish_environment(&environment, work)?;
            Ok(())
        },
    )?;
    Ok(())
}

fn prepare_lazy<T>(
    report: &Report,
    facts: &mut Declarations,
    extra_work: usize,
    extra_bytes: usize,
    action: impl FnOnce(
        &Graph,
        &mut Declarations,
        &mut ReservationWork,
        &mut Allocation,
    ) -> Result<T, Error>,
) -> Result<Option<T>, Error> {
    let Some(headers) = facts
        .crates
        .as_deref()
        .and_then(|linkage| linkage.originals.as_deref())
    else {
        return Ok(None);
    };
    let Some(ImportedOwners::Ordinary(graph)) = headers.source.as_ref() else {
        return Ok(None);
    };
    if !matches!(graph.canonical, Canonical::Lazy(_)) {
        return Ok(None);
    }
    headers.charge(1)?;
    let graph = Rc::clone(graph);
    let before = headers.work.get();
    let base = headers.base;
    let mut work = ReservationWork {
        steps: before.saturating_add(extra_work),
        retained: base
            .saturating_add(headers.bytes)
            .saturating_add(extra_bytes),
        peak: 0,
    };
    let mut allocation = Allocation::default();
    let result = (|| {
        work.charge(0)?;
        work.reserve(0)?;
        authenticate(&graph, report, facts, &mut work)?;
        action(&graph, facts, &mut work, &mut allocation)
    })();
    let headers = facts
        .crates
        .as_deref_mut()
        .and_then(|linkage| linkage.originals.as_deref_mut())
        .ok_or_else(|| error("ordinary original lost its actual accounting owner"))?;
    // Preserve attempted work and failed preallocation without billing existing
    // original bodies twice. Their caller still owns extra_work/extra_bytes.
    headers
        .work
        .set(before.saturating_add(work.steps.saturating_sub(before.saturating_add(extra_work))));
    headers.bytes = headers.bytes.max(
        work.retained
            .saturating_sub(extra_bytes)
            .saturating_sub(base),
    );
    headers.observed.include(allocation);
    result.map(Some)
}

fn closure_environment(
    graph: &Graph,
    facts: &Declarations,
    closure: super::super::unpaid_provided::ClosureHeader,
    domain: &OrdinaryDomain,
    work: &mut ReservationWork,
    allocation: &mut Allocation,
) -> Result<Arc<Environment>, Error> {
    work.charge(2)?;
    if let Some(actual) = domain.environment() {
        return Ok(Arc::clone(actual));
    }
    let view = graph.recipe_view();
    let raw = domain.raw()?;
    let rows = view.rows()?;
    let inputs = rows
        .body_inputs
        .get(rows.original_index(raw).map_err(error)?)
        .filter(|inputs| inputs.body == raw)
        .ok_or_else(|| error("ordinary closure environment lost its actual body inputs"))?;
    let owner_header = facts
        .crates
        .as_deref()
        .and_then(|linkage| linkage.originals.as_deref())
        .and_then(|headers| headers.get(closure.owner));
    match inputs.environment {
        s::EnvironmentOrigin::Declaration(owner)
            if view.definition(owner, work)? == closure.owner =>
        {
            facts
                .generics
                .environments
                .get(&closure.owner)
                .map(Arc::clone)
                .ok_or_else(|| {
                    error("ordinary closure environment lost its declaration construction")
                })
        }
        s::EnvironmentOrigin::ConstTraitDefault { member, trait_ } => {
            if view.definition(member, work)? != closure.owner
                || trait_.artifact != rows.policy.provider
            {
                return Err(error(
                    "const default closure environment changed its actual member or provider",
                ));
            }
            let owner = view.definition(trait_.declaration, work)?;
            if facts.associated.trait_owner(closure.owner) != Some(owner) {
                return Err(error(
                    "const default closure environment changed its actual trait owner",
                ));
            }
            if let Some(actual) = owner_header.and_then(|header| header.environment.as_ref()) {
                return Ok(Arc::clone(actual));
            }
            default_environment(facts, closure.owner, owner, work, allocation)
        }
        s::EnvironmentOrigin::Contract => {
            let mut observed = Observed {
                work,
                allocation: Allocation::default(),
            };
            let result = ContractParts::original(graph, facts, domain, inputs, &mut observed)?
                .build(facts, &mut observed);
            allocation.include(observed.allocation);
            result
        }
        _ => Err(error(
            "ordinary closure environment requires its actual defining construction profile",
        )),
    }
}
fn authenticate(
    graph: &Graph,
    report: &Report,
    facts: &Declarations,
    work: &mut ReservationWork,
) -> Result<(), Error> {
    work.charge(8)?;
    graph.require_store(&facts.types).map_err(error)?;
    let imported = report
        .resolution
        .crates
        .as_deref()
        .ok_or_else(|| error("ordinary original requires its actual SourceClient imports"))?;
    let client = report
        .package
        .source_client()
        .ok_or_else(|| error("ordinary original requires its actual SourceClient report"))?;
    let index = client
        .require_inventory(imported.dependencies())
        .map_err(error)?;
    let catalog = imported
        .origin_catalog()
        .map_err(error)?
        .ok_or_else(|| error("ordinary original lost its actual origin catalog"))?;
    if index != graph.provider_index
        || !std::ptr::eq(catalog.as_ref(), graph.origins.as_ref())
        || !Arc::ptr_eq(&imported.shared_dependencies(), &graph.inventory)
        || !matches!(facts.associated.imported_owner(), Some(ImportedOwners::Ordinary(actual)) if std::ptr::eq(actual.as_ref(), graph))
    {
        return Err(error(
            "ordinary original changed its actual namespace or mapping owner",
        ));
    }
    facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| error("ordinary original lost its actual source role catalog"))?
        .require_provider_report(report)
        .map_err(error)?;
    graph.descriptor(&graph.origins).map_err(error)?;
    Ok(())
}
fn environment(
    graph: &Graph,
    facts: &Declarations,
    definition: DefId,
    domain: &OrdinaryDomain,
    work: &mut ReservationWork,
    allocation: &mut Allocation,
) -> Result<Arc<Environment>, Error> {
    work.charge(2)?;
    let header = facts
        .crates
        .as_deref()
        .and_then(|linkage| linkage.originals.as_deref())
        .and_then(|headers| headers.get(definition));
    if let Some(actual) = domain.environment() {
        if header
            .and_then(|header| header.environment.as_ref())
            .is_some_and(|header| !Arc::ptr_eq(header, actual))
        {
            return Err(error(
                "ordinary environment changed its actual published Arc",
            ));
        }
        return Ok(Arc::clone(actual));
    }
    let view = graph.recipe_view();
    let raw = domain.raw()?;
    let rows = view.rows()?;
    let inputs = rows
        .body_inputs
        .get(rows.original_index(raw).map_err(error)?)
        .filter(|inputs| inputs.body == raw)
        .ok_or_else(|| error("ordinary environment lost its actual body inputs"))?;
    match inputs.environment {
        s::EnvironmentOrigin::Declaration(owner) if view.definition(owner, work)? == definition => {
            let actual = facts
                .generics
                .environments
                .get(&definition)
                .ok_or_else(|| error("ordinary environment lost its declaration construction"))?;
            if header.is_some_and(|header| {
                header
                    .environment
                    .as_ref()
                    .is_none_or(|header| !Arc::ptr_eq(header, actual))
            }) {
                return Err(error(
                    "ordinary declaration environment changed its actual Arc",
                ));
            }
            Ok(Arc::clone(actual))
        }
        s::EnvironmentOrigin::ConstTraitDefault { member, trait_ } => {
            if view.definition(member, work)? != definition
                || trait_.artifact != rows.policy.provider
            {
                return Err(error(
                    "const default environment changed its actual member or provider",
                ));
            }
            let owner = view.definition(trait_.declaration, work)?;
            if header.is_some_and(|header| header.environment.is_some()) {
                return Err(error("const default acquired an unconstructed environment"));
            }
            default_environment(facts, definition, owner, work, allocation)
        }
        s::EnvironmentOrigin::Contract => {
            if header.is_some_and(|header| header.environment.is_some()) {
                return Err(error(
                    "ordinary Contract acquired an unconstructed environment",
                ));
            }
            let mut observed = Observed {
                work,
                allocation: Allocation::default(),
            };
            let result = ContractParts::original(graph, facts, domain, inputs, &mut observed)?
                .build(facts, &mut observed);
            allocation.include(observed.allocation);
            result
        }
        _ => Err(error(
            "ordinary environment requires its actual defining construction profile",
        )),
    }
}

fn default_environment(
    facts: &Declarations,
    member: DefId,
    owner: DefId,
    work: &mut ReservationWork,
    allocation: &mut Allocation,
) -> Result<Arc<Environment>, Error> {
    if facts.associated.trait_owner(member) != Some(owner) {
        return Err(error(
            "const default environment changed its actual trait owner",
        ));
    }
    let actual = facts
        .generics
        .environments
        .get(&member)
        .ok_or_else(|| error("const default lost its complete declaration environment"))?;
    let clause = facts.defining_self_clause(owner).map_err(error)?;
    let clauses = actual
        .trait_clauses()
        .map_err(error)?
        .count()
        .saturating_add(1);
    work.charge(clauses)?;
    let bound = Environment::trait_leaf_allocation_bound(1, 0, clauses, clauses);
    work.reserve(bound)?;
    let environment = Arc::new(
        actual
            .as_ref()
            .clone()
            .with_clauses(&facts.types, &[clause])
            .map_err(error)?,
    );
    let retained = environment.retain_owned_from(actual, &mut Retained::default());
    if retained.bytes.saturating_add(2 * size_of::<usize>()) > bound {
        return Err(error(
            "const default exceeded its actual environment prepayment",
        ));
    }
    work.charge(retained.visits)?;
    allocation.include(retained);
    allocation.add(2 * size_of::<usize>());
    Ok(environment)
}
