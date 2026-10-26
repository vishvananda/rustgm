//! Comparison closure precedes conformance; only completed member checks seal execution.
use super::{
    graph_pairs,
    registry_graph::{Forest, State},
    retention_work::Budget,
};
use crate::{
    resolve::{expansion::Report, model::DefKind},
    sema::{
        model::Declarations,
        traits::{
            heads::Head,
            implementations::{EntryKey, Identity},
        },
    },
    support::{hash::HashSet, retained::Growth},
};
pub(in crate::sema) fn prepare(facts: &mut Declarations, report: &Report) -> Result<(), String> {
    graph_pairs::drain(facts, report)?;
    let Some(graph) = graph_pairs::registry(facts)?.specialization_graph() else {
        return Ok(());
    };
    graph.current(facts)?;
    let State::Building(builder) = &graph.state else {
        return Ok(());
    };
    graph.budget.charge(
        builder.sources.len(),
        builder.sources.len().saturating_mul(size_of::<EntryKey>()),
    )?;
    let keys: Vec<_> = builder.sources.keys().copied().collect();
    for key in keys {
        let graph = graph_pairs::registry(facts)?
            .specialization_graph()
            .ok_or("specialization graph disappeared")?;
        if graph.witness(facts, report, key)?.is_some() {
            return Err("specialization comparison omitted its source witness".into());
        }
    }
    let graph = graph_pairs::active_graph(facts)?;
    let budget = graph.budget.alias()?;
    let minimal = graph.requests.min_checks();
    let State::Building(builder) = std::mem::replace(&mut graph.state, State::Failed) else {
        return Err("specialization builder changed during closure".into());
    };
    // Failed construction never publishes a partial forest. Work stays in the
    // same budget while the Graph remains installed and conservatively failed.
    budget.charge(1, size_of::<Forest>())?;
    let mut forest = super::forest_edges::build(*builder, &budget)?;
    super::forest_order::number(&mut forest, &budget)?;
    families(facts, &mut forest, &budget)?;
    functions_only(facts, report, &forest, &budget)?;
    if minimal {
        super::minimal_admission::validate(facts, report, &forest, &budget)?;
    }
    graph_pairs::active_graph(facts)?.state = State::Conforming(Box::new(forest));
    Ok(())
}
fn families(
    facts: &Declarations,
    forest: &mut Forest,
    budget: &Budget,
) -> Result<(), &'static str> {
    for vertex in &forest.vertices {
        budget.charge(1, 0)?;
        let predicate = vertex.witness.predicate;
        let (application, _) = predicate
            .bound
            .trait_parts()
            .ok_or("specialization requires an actual trait application")?;
        let identity = Identity::User(
            facts
                .types
                .trait_application(application)
                .ok_or("missing specialization trait application")?
                .item,
        );
        let kind = facts
            .types
            .kind(predicate.ty)
            .ok_or("missing specialization subject")?;
        let head =
            (!matches!(kind, crate::types::Kind::Parameter(_))).then(|| Head::canonical(kind));
        let growth = Growth::map::<Identity, HashSet<Option<Head>>>(
            forest.families.len(),
            forest.families.capacity(),
            usize::from(!forest.families.contains_key(&identity)),
        );
        budget.charge(growth.work, growth.bytes)?;
        let heads = forest.families.entry(identity).or_default();
        let growth = Growth::map::<Option<Head>, ()>(
            heads.len(),
            heads.capacity(),
            usize::from(!heads.contains(&head)),
        );
        budget.charge(growth.work, growth.bytes)?;
        heads.insert(head);
    }
    Ok(())
}
fn functions_only(
    facts: &Declarations,
    report: &Report,
    forest: &Forest,
    budget: &Budget,
) -> Result<(), &'static str> {
    for vertex in &forest.vertices {
        let owner = facts
            .associated
            .declaration_owner(vertex.witness.key.owner())
            .ok_or("specialization source owner disappeared")?;
        let members = facts
            .associated
            .owner_members(owner)
            .ok_or("specialization source members are incomplete")?;
        budget.charge(members.len(), 0)?;
        for member in members {
            if report
                .resolution
                .definitions
                .get(member.0 as usize)
                .is_none_or(|actual| actual.kind != DefKind::Function)
            {
                return Err("specialization of associated values or types is not implemented");
            }
        }
    }
    Ok(())
}
pub(in crate::sema) fn seal(facts: &mut Declarations, report: &Report) -> Result<(), String> {
    let registry = graph_pairs::registry(facts)?;
    let Some(graph) = registry.specialization_graph() else {
        return Ok(());
    };
    graph.current(facts)?;
    let State::Conforming(forest) = &graph.state else {
        return Err("specialization execution requires completed conformance closure".into());
    };
    graph.budget.charge(forest.vertices.len(), 0)?;
    forest.validate_seal(registry)?;
    for vertex in &forest.vertices {
        if !registry.is_validated(vertex.witness.key.owner()) {
            return Err("specialization header conformance is unfinished".into());
        }
        vertex
            .witness
            .policy
            .validate(report, &graph.requests, |steps, bytes| {
                graph.budget.charge(steps, bytes)
            })
            .map_err(|error| error.to_string())?;
    }
    finality(facts, report, forest, &graph.budget)?;
    let graph = graph_pairs::active_graph(facts)?;
    let State::Conforming(forest) = std::mem::replace(&mut graph.state, State::Failed) else {
        return Err("specialization conformance closure changed during sealing".into());
    };
    graph.state = State::Sealed(forest);
    Ok(())
}
fn finality(
    facts: &Declarations,
    report: &Report,
    forest: &Forest,
    budget: &Budget,
) -> Result<(), String> {
    let registry = graph_pairs::registry(facts)?;
    for relation in &forest.parents {
        let child = registry.checked_entry(relation.child)?;
        for &(declaration, actual) in &child.methods {
            budget.charge(1, 0)?;
            if actual.is_none() {
                continue;
            }
            // An omitted inherited member is final at this immediate boundary;
            // an older defining ancestor's default flag cannot reopen it.
            let Some(parent) = registry.direct_method(relation.parent, declaration)? else {
                return Err("specializing method overrides a final inherited member".into());
            };
            let key = report
                .resolution
                .definitions
                .get(parent.0 as usize)
                .filter(|definition| definition.kind == DefKind::Function)
                .and_then(|definition| definition.node)
                .ok_or("specialization parent method has no actual local function owner")?;
            if report
                .active_defaultness(key)
                .map_err(|error| error.to_string())?
                .is_none()
            {
                return Err("specializing method overrides a final parent member".into());
            }
        }
    }
    Ok(())
}

/// Verification alone never produces executable analyzed facts or a usable artifact.
pub(in crate::sema) fn reject_execution(
    facts: &Declarations,
    report: &Report,
) -> Result<(), crate::resolve::package::Error> {
    use crate::resolve::package::Error;
    let Some(registry) = facts.generics.implementations.as_deref() else {
        return Ok(());
    };
    let Some(graph) = registry.specialization_graph() else {
        return Ok(());
    };
    graph.current(facts).map_err(Error::unlocated)?;
    let State::Sealed(forest) = &graph.state else {
        return Err(Error::unlocated(
            "specialization verification is unfinished",
        ));
    };
    let source = forest
        .vertices
        .first()
        .ok_or_else(|| Error::unlocated("specialization verifier has no authenticated source"))?
        .witness
        .policy
        .source;
    Err(Error::at(
        source.file,
        report
            .package
            .unit(source.file)
            .configured
            .ast
            .node(source.node)
            .span
            .start,
        "specialization graph execution is not implemented",
    ))
}
