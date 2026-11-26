//! Attach remapped provider ancestry to the consumer's actual implementation registry.
use super::{
    candidate_order::{Order, ancestry_order},
    graph_pairs,
    proof_attempt::{Attempt, Progress},
    registry_graph::{EdgeId, Forest, Graph, PairKey, Relation, Vertex, VertexId, Witness},
    retention_work::Budget,
};
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::{crates::ImportedOwners, model::Declarations, traits::implementations::EntryKey},
    support::{hash::HashMap, retained::Growth},
    types::GenericArgs,
};

#[derive(Clone, Copy)]
pub(in crate::sema) struct ImportedParent {
    pub child: DefId,
    pub parent: DefId,
    pub arguments: GenericArgs,
}

pub(in crate::sema) fn install_imported(
    facts: &mut Declarations,
    report: &Report,
) -> Result<(), String> {
    let Some(ImportedOwners::Ordinary(graphs)) = facts.associated.imported_owner() else {
        return Ok(());
    };
    if graphs.iter().all(|graph| graph.specialization().is_empty()) {
        return Ok(());
    }
    let graphs = std::rc::Rc::clone(graphs);
    if graph_pairs::registry(facts)?
        .specialization_graph()
        .is_none()
    {
        let graph = Graph::new(facts, report)?;
        facts
            .generics
            .implementations
            .as_deref_mut()
            .ok_or("specialization registry disappeared")?
            .install_specialization_graph(graph)?;
    }
    let budget = graph_pairs::active_graph(facts)?.budget.alias()?;
    budget.charge(1, size_of::<Forest>())?;
    let mut forest = Forest {
        vertices: Vec::new(),
        parents: Vec::new(),
        by_entry: HashMap::default(),
        families: HashMap::default(),
    };
    for provider in graphs.iter() {
        provider.require_store(&facts.types)?;
        for &edge in provider.specialization() {
            register(facts, &mut forest, edge, &budget)?;
        }
    }
    super::forest_order::number(&mut forest, &budget)?;
    super::graph_closure::families(facts, &mut forest, &budget)?;
    forest.validate_seal(graph_pairs::registry(facts)?)?;
    let graph = graph_pairs::active_graph(facts)?;
    if graph.imported.is_some() {
        return Err("the imported specialization forest was already installed".into());
    }
    graph.imported = Some(Box::new(forest));
    Ok(())
}

fn register(
    facts: &mut Declarations,
    forest: &mut Forest,
    edge: ImportedParent,
    budget: &Budget,
) -> Result<(), &'static str> {
    let registry = graph_pairs::registry(facts)?;
    let child = registry.entry_key(edge.child)?;
    let parent = registry.entry_key(edge.parent)?;
    let child = vertex(facts, forest, child, budget)?;
    let parent = vertex(facts, forest, parent, budget)?;
    let (child, parent) = (child.index(), parent.index());
    let relation = Relation {
        child: forest.vertices[child].witness.key,
        parent: forest.vertices[parent].witness.key,
        parent_arguments: edge.arguments,
    };
    check_substitution(
        facts,
        edge,
        forest.vertices[child].witness.predicate,
        forest.vertices[parent].witness.predicate,
        budget,
    )?;
    let child = &mut forest.vertices[child];
    if child.parent.is_some() {
        return Err("imported specialization has two immediate parents");
    }
    let growth = Growth::vector::<Relation>(forest.parents.len(), forest.parents.capacity(), 1);
    budget.charge(1 + growth.work, growth.bytes)?;
    child.parent = Some(EdgeId::new(forest.parents.len())?);
    forest.parents.push(relation);
    Ok(())
}

fn vertex(
    facts: &Declarations,
    forest: &mut Forest,
    key: EntryKey,
    budget: &Budget,
) -> Result<VertexId, &'static str> {
    budget.charge(1, 0)?;
    if let Some(&vertex) = forest.by_entry.get(&key) {
        return Ok(vertex);
    }
    let registry = graph_pairs::registry(facts)?;
    let header = registry.checked_entry(key)?;
    if header.source.is_some()
        || !matches!(
            facts
                .associated
                .owner(key.owner())
                .and_then(|owner| facts.associated.get(owner))
                .map(|owner| owner.kind),
            Some(crate::sema::associated::OwnerKind::ImportedImpl(_))
        )
        || registry.polarity(&facts.types, &facts.associated, key.owner())?
            != crate::sema::traits::implementations::Polarity::Positive
    {
        return Err("specialization endpoint is not an actual imported positive impl");
    }
    let mut growth = Growth::vector::<Vertex>(forest.vertices.len(), forest.vertices.capacity(), 1);
    growth.include(Growth::map::<EntryKey, VertexId>(
        forest.by_entry.len(),
        forest.by_entry.capacity(),
        1,
    ));
    budget.charge(growth.work, growth.bytes)?;
    let vertex = VertexId::new(forest.vertices.len())?;
    forest.by_entry.insert(key, vertex);
    forest.vertices.push(Vertex {
        witness: Witness {
            key,
            predicate: header.predicate,
            policy: None,
        },
        parent: None,
        depth: 0,
        preorder: 0,
        subtree_end: 0,
    });
    Ok(vertex)
}

fn check_substitution(
    facts: &mut Declarations,
    edge: ImportedParent,
    child: crate::types::parameters::predicates::Predicate,
    parent: crate::types::parameters::predicates::Predicate,
    budget: &Budget,
) -> Result<(), &'static str> {
    let mut mapping =
        super::header_conditions::mapping(facts, edge.parent, edge.arguments, budget)?;
    let applied = mapping
        .fold
        .predicate_charged(&mut facts.types, parent, &|steps, bytes| {
            budget.charge(steps, bytes)
        })?;
    mapping.fold.pay()?;
    let (actual, _) = child
        .bound
        .trait_parts()
        .ok_or("specialization child lost its trait")?;
    let (substituted, _) = applied
        .bound
        .trait_parts()
        .ok_or("specialization parent lost its trait")?;
    if child.ty != applied.ty || actual != substituted {
        return Err("imported specialization substitution changes the actual trait header");
    }
    Ok(())
}

/// Imported ancestry orders candidates only after their actual headers have been
/// remapped. As for local pairs, coherence also checks applicability in both
/// directions before accepting that claimed order.
pub(super) fn verify_pair(facts: &mut Declarations, key: PairKey) -> Result<bool, String> {
    let registry = graph_pairs::registry(facts)?;
    let [a, b] = key.entries();
    let Some(graph) = registry.specialization_graph() else {
        return Ok(false);
    };
    let Some(forest) = graph.imported_forest(a) else {
        return Ok(false);
    };
    let Order::Prefer(child) = ancestry_order(forest, registry, a, b)? else {
        return Ok(false);
    };
    let child = if a.owner() == child { a } else { b };
    let expected = super::member_paths::checked(forest, registry, child)?
        .and_then(|vertex| vertex.parent)
        .and_then(|edge| forest.parents.get(edge.index()))
        .copied()
        .filter(|edge| [a, b].contains(&edge.parent));
    let mut proof = Attempt::new(key, graph.budget.alias()?)?;
    match proof.advance(facts)? {
        Progress::Complete(Some(relation)) if relation.child == child => {
            if expected.is_some_and(|edge| edge.parent_arguments != relation.parent_arguments) {
                return Err("imported specialization changed its proved parent arguments".into());
            }
            Ok(true)
        }
        Progress::Pending(_) => {
            Err("imported specialization requires an unfinished obligation".into())
        }
        Progress::Complete(_) => {
            Err("imported specialization does not match header applicability".into())
        }
    }
}
