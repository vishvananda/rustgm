//! Publication reads the provider's sealed immediate parent.
use super::{member_paths, registry_graph::State};
use crate::{resolve::model::DefId, sema::model::Declarations, types::GenericArgs};

pub(in crate::sema) fn immediate_parent(
    facts: &Declarations,
    child: DefId,
) -> Result<Option<(DefId, GenericArgs)>, &'static str> {
    let Some(registry) = facts.generics.implementations.as_deref() else {
        return Ok(None);
    };
    let Some(graph) = registry.specialization_graph() else {
        return Ok(None);
    };
    graph.current(facts)?;
    let State::Sealed(forest) = &graph.state else {
        return Err("source specialization publication requires the sealed forest");
    };
    let key = registry.entry_key(child)?;
    let Some(edge) = member_paths::checked(forest, registry, key)?.and_then(|vertex| vertex.parent)
    else {
        return Ok(None);
    };
    let edge = forest
        .parents
        .get(edge.index())
        .ok_or("source specialization parent disappeared")?;
    if edge.child != key {
        return Err("source specialization parent belongs to another implementation");
    }
    Ok(Some((edge.parent.owner(), edge.parent_arguments)))
}
