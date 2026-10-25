//! Indirect grammar records are part of an actual parent snapshot, not just its tag.
use super::{
    Error,
    demand::{self, Work},
};
use crate::{
    resolve::unit::{FileId, Unit},
    syntax::{
        ast::{Kind, NodeId},
        item,
    },
};
use std::cell::Cell;

#[derive(PartialEq, Eq)]
pub(super) enum Auxiliary {
    Item(item::Item),
    Generic(item::GenericParameter),
    Function(item::FunctionType),
    Variant(item::Variant),
    Use(item::UseTree),
}
fn value(unit: &Unit, node: NodeId) -> Option<Auxiliary> {
    match unit.configured.ast.nodes.get(node.get() as usize)?.kind {
        Kind::Item(index) => unit
            .configured
            .ast
            .items
            .get(index as usize)
            .copied()
            .map(Auxiliary::Item),
        Kind::Generic(index) => unit
            .configured
            .ast
            .generics
            .get(index as usize)
            .copied()
            .map(Auxiliary::Generic),
        Kind::FunctionType(index) => unit
            .configured
            .ast
            .function_types
            .get(index as usize)
            .copied()
            .map(Auxiliary::Function),
        Kind::Variant(index) => unit
            .configured
            .ast
            .variants
            .get(index as usize)
            .copied()
            .map(Auxiliary::Variant),
        Kind::UseTree(index) => unit
            .configured
            .ast
            .use_trees
            .get(index as usize)
            .copied()
            .map(Auxiliary::Use),
        _ => None,
    }
}
pub(super) fn capture(
    unit: &Unit,
    node: NodeId,
    work: &Cell<Work>,
    file: FileId,
) -> Result<Option<Box<Auxiliary>>, Error> {
    let value = value(unit, node);
    if value.is_some() {
        demand::charge(work, file, 1, size_of::<Auxiliary>())?;
    }
    Ok(value.map(Box::new))
}
pub(super) fn matches(unit: &Unit, node: NodeId, expected: Option<&Auxiliary>) -> bool {
    value(unit, node).as_ref() == expected
}
