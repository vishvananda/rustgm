//! Demand-only configured context data; requests and macro allowances remain separate.
use super::positions::{self, Position};
use crate::{
    resolve::{
        package::Error,
        unit::{FileId, Unit},
    },
    support::retained::Growth,
    syntax::ast::{Kind, NodeId},
};

pub(crate) struct AttributePositions<'unit> {
    unit: &'unit Unit,
    positions: Vec<Option<Position>>,
}
impl<'unit> AttributePositions<'unit> {
    pub(crate) fn prepare(
        unit: &'unit Unit,
        file: FileId,
        mut pay: impl FnMut(usize, usize) -> Result<(), Error>,
    ) -> Result<Self, Error> {
        let ast = &unit.configured.ast;
        if unit.configured.active.len() != ast.nodes.len()
            || unit.owner_indices.len() != ast.nodes.len()
            || unit.include_context.is_some()
        {
            return Err(Error::at(
                file,
                0,
                "attribute positions lack their configured unit",
            ));
        }
        let mut positions = Vec::new();
        pay(
            ast.nodes.len(),
            ast.nodes
                .len()
                .saturating_mul(size_of::<Option<Position>>()),
        )?;
        positions
            .try_reserve_exact(ast.nodes.len())
            .map_err(|_| Error::at(file, 0, "attribute position table allocation failed"))?;
        positions.resize(ast.nodes.len(), None);
        let mut pending = Vec::new();
        append(
            &mut pending,
            (
                ast.root,
                if file.0 == 0 {
                    Position::CrateRoot
                } else {
                    Position::Other
                },
            ),
            &mut pay,
        )?;
        while let Some((node, position)) = pending.pop() {
            pay(1, 0)?;
            let index = node.get() as usize;
            let slot = positions
                .get_mut(index)
                .ok_or_else(|| Error::at(file, 0, "attribute position node is outside its unit"))?;
            if !unit.configured.active[index] {
                continue;
            }
            if let Some(previous) = *slot {
                if previous != position {
                    return Err(Error::at(
                        file,
                        0,
                        "attribute node has incompatible configured contexts",
                    ));
                }
                continue;
            }
            *slot = Some(position);
            ast.try_for_each_child(node, &mut |child| {
                append(
                    &mut pending,
                    (child, positions::child(ast, node, position, child)),
                    &mut pay,
                )
            })?;
        }
        Ok(Self { unit, positions })
    }
    pub(crate) fn is_for(&self, unit: &Unit) -> bool {
        std::ptr::eq(self.unit, unit)
    }
    pub(crate) fn requires_expression_feature(
        &self,
        unit: &Unit,
        node: NodeId,
    ) -> Result<bool, &'static str> {
        if !self.is_for(unit) {
            return Err("attribute positions belong to a different configured unit");
        }
        let owner = unit
            .owner_indices
            .get(node.get() as usize)
            .copied()
            .flatten()
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
            .ok_or("attribute position lacks its configured owner")?;
        if owner.node != node
            || !owner.enabled
            || !matches!(unit.configured.ast.node(node).kind, Kind::Attributed { .. })
        {
            return Err("attribute position has a different configured owner");
        }
        self.positions
            .get(node.get() as usize)
            .copied()
            .flatten()
            .map(|position| position == Position::Expression)
            .ok_or("attribute owner was not visited in its configured unit")
    }
}
fn append(
    pending: &mut Vec<(NodeId, Position)>,
    value: (NodeId, Position),
    pay: &mut impl FnMut(usize, usize) -> Result<(), Error>,
) -> Result<(), Error> {
    let growth = Growth::vector::<(NodeId, Position)>(pending.len(), pending.capacity(), 1);
    pay(growth.work.saturating_add(1), growth.bytes)?;
    pending
        .try_reserve(1)
        .map_err(|_| Error::unlocated("attribute position traversal allocation failed"))?;
    pending.push(value);
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/placement_queries.rs"]
mod tests;
