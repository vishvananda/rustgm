//! One topological canonical type remap preserves exact formal declaration IDs.
use super::{Declarations, Error, Headers, error};
use crate::{
    resolve::crates::Imported,
    support::crates::{CrateIdentity, generics as g},
    types::{
        ItemId, Kind, TypeId,
        parameters::{Parameter, ParameterKind},
    },
};
use std::cell::Cell;
#[cfg(test)]
#[path = "../../../../tests/invariants/header_graphs.rs"]
mod tests;
#[derive(Default)]
pub(super) struct Graph {
    nodes: usize,
    edges: usize,
    visit: Cell<usize>,
}
pub(super) struct Mapped {
    pub nodes: Vec<TypeId>,
    pub parameters: Vec<Option<TypeId>>,
    graphs: Vec<Graph>,
    scalars: usize,
    round: Cell<usize>,
}
impl Mapped {
    pub(super) fn environment_bound(
        &self,
        rows: &g::Rows,
        function: &g::Function,
    ) -> Result<(usize, usize), Error> {
        let round = self
            .round
            .get()
            .checked_add(1)
            .ok_or_else(|| error("imported header accounting round overflow"))?;
        self.round.set(round);
        let mut nodes = self.scalars;
        let mut edges = 0_usize;
        for clause in function.clauses.slice(&rows.clauses).map_err(error)? {
            let owner = match rows.types[clause.subject.0 as usize] {
                g::Type::Scalar(_) => continue,
                g::Type::Parameter(reference) => {
                    let g::Declaration::TypeParameter { owner, .. } =
                        rows.declarations[reference.0 as usize]
                    else {
                        return Err(error("header accounting found a foreign parameter kind"));
                    };
                    owner
                }
                g::Type::Function(signature) => rows.binders[signature.binder.0 as usize].owner,
            };
            let graph = &self.graphs[owner.0 as usize];
            if graph.visit.replace(round) != round {
                nodes = nodes.saturating_add(graph.nodes);
                edges = edges.saturating_add(graph.edges);
            }
        }
        Ok((nodes, edges))
    }
    fn record(&mut self, rows: &g::Rows, node: g::Type) -> Result<(), Error> {
        let (owner, edges) = match node {
            g::Type::Scalar(_) => {
                self.scalars += 1;
                return Ok(());
            }
            g::Type::Parameter(reference) => {
                let g::Declaration::TypeParameter { owner, .. } =
                    rows.declarations[reference.0 as usize]
                else {
                    return Err(error("header accounting found an invalid type parameter"));
                };
                (owner, 0)
            }
            g::Type::Function(signature) => (
                rows.binders[signature.binder.0 as usize].owner,
                (signature.parameters.end - signature.parameters.start) as usize + 1,
            ),
        };
        let graph = &mut self.graphs[owner.0 as usize];
        graph.nodes = graph.nodes.saturating_add(1);
        graph.edges = graph.edges.saturating_add(edges);
        Ok(())
    }
}
pub(super) fn import(
    imported: &Imported,
    facts: &mut Declarations,
    owner: &CrateIdentity,
    rows: &g::Rows,
    headers: &mut Headers,
) -> Result<Mapped, Error> {
    let mut mapped = Mapped {
        nodes: Vec::with_capacity(rows.types.len()),
        parameters: vec![None; rows.declarations.len()],
        graphs: (0..rows.declarations.len())
            .map(|_| Graph::default())
            .collect(),
        scalars: 0,
        round: Cell::new(0),
    };
    for node in &rows.types {
        headers.charge(1)?;
        mapped.record(rows, *node)?;
        facts.crates.get_or_insert_default().import_work.type_nodes += 1;
        let ty = match *node {
            g::Type::Scalar(value) => super::super::scalar(value),
            g::Type::Parameter(reference) => {
                let g::Declaration::TypeParameter {
                    owner: declaration,
                    ordinal,
                    sized,
                    ..
                } = rows.declarations[reference.0 as usize]
                else {
                    return Err(error(
                        "imported type parameter changed its declaration kind",
                    ));
                };
                let definition = imported
                    .generic_definition(owner, declaration)
                    .map_err(error)?;
                let id = facts
                    .types
                    .intern_parameter(Parameter {
                        owner: ItemId(definition.0),
                        index: ordinal,
                        kind: ParameterKind::Type { sized },
                    })
                    .map_err(error)?;
                facts.types.intern(Kind::Parameter(id)).map_err(error)?
            }
            g::Type::Function(signature) => {
                callable(facts, rows, &mapped.nodes, signature, headers)?
            }
        };
        if let g::Type::Parameter(reference) = *node {
            mapped.parameters[reference.0 as usize] = Some(ty);
            let binding = imported
                .generic_definition(owner, reference)
                .map_err(error)?;
            let slot = facts
                .definitions
                .get_mut(binding.0 as usize)
                .ok_or_else(|| error("imported rigid binding has no real declaration slot"))?;
            if slot.replace(ty).is_some() {
                return Err(error("imported rigid binding was already typed"));
            }
        }
        mapped.nodes.push(ty);
    }
    Ok(mapped)
}

fn callable(
    facts: &mut Declarations,
    rows: &g::Rows,
    nodes: &[TypeId],
    signature: g::Signature,
    headers: &mut Headers,
) -> Result<TypeId, Error> {
    let roots = signature.parameters.slice(&rows.roots).map_err(error)?;
    headers.charge(roots.len().saturating_add(1))?;
    let work = &mut facts.crates.get_or_insert_default().import_work;
    work.signatures += 1;
    work.roots += roots.len() + 1;
    let parameters = roots
        .iter()
        .map(|root| {
            nodes
                .get(root.0 as usize)
                .copied()
                .ok_or_else(|| error("unknown imported callable parameter"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let result = *nodes
        .get(signature.result.0 as usize)
        .ok_or_else(|| error("unknown imported callable result"))?;
    let parameters = facts.types.intern_list(&parameters).map_err(error)?;
    facts
        .types
        .intern(Kind::Function {
            bound_regions: rows.binders[signature.binder.0 as usize].count,
            parameters,
            result,
            unsafe_: signature.unsafe_,
            convention: crate::types::callable::Convention::RUST,
        })
        .map_err(error)
}
