//! Selected inline literals have their own arena identity; byte pools are not copied by index.
use super::choice::Charge;
use crate::{
    lex::token::LexError,
    literal::storage::StoredValue,
    syntax::{
        ast::{BlockFlavor, Kind, List, Node, NodeId, Syntax},
        token_tree::error,
    },
};
pub(super) fn literal(
    source: &Syntax<StoredValue>,
    index: u32,
    target: &mut Syntax<StoredValue>,
    charge: &Charge<'_>,
    at: u32,
) -> Result<u32, LexError> {
    charge(1, 0)?;
    let value = *source
        .literals
        .get(index as usize)
        .ok_or_else(|| error(at, "derived field lost its parsed literal"))?;
    if !matches!(
        value,
        StoredValue::Integer(_)
            | StoredValue::Boolean(_)
            | StoredValue::Character(_)
            | StoredValue::Byte(_)
    ) {
        return Err(error(
            at,
            "derived constant literal requires its actual byte-pool owner",
        ));
    }
    let growth = crate::support::retained::Growth::vector::<StoredValue>(
        target.literals.len(),
        target.literals.capacity(),
        1,
    );
    charge(growth.work.saturating_add(1), growth.bytes)?;
    let copied = u32::try_from(target.literals.len())
        .map_err(|_| error(at, "derived literal identity overflow"))?;
    target
        .literals
        .try_reserve(1)
        .map_err(|_| error(at, "derived literal allocation failed"))?;
    target.literals.push(value);
    Ok(copied)
}

pub(super) fn pending(
    actual: Node,
    id: NodeId,
    target: &mut Syntax<StoredValue>,
    charge: &Charge<'_>,
) -> Result<(), LexError> {
    if !matches!(actual.kind, Kind::DeferredLiteral(_)) {
        return Ok(());
    }
    let growth = crate::support::retained::Growth::vector::<NodeId>(
        target.pending.literals.len(),
        target.pending.literals.capacity(),
        1,
    );
    charge(growth.work.saturating_add(1), growth.bytes)?;
    target.pending.literals.try_reserve(1).map_err(|_| {
        error(
            actual.span.start,
            "derived deferred literal allocation failed",
        )
    })?;
    target.pending.literals.push(id);
    Ok(())
}
pub(super) fn children(
    source: &Syntax<StoredValue>,
    actual: &mut Node,
    target: &mut Syntax<StoredValue>,
    charge: &Charge<'_>,
    depth: usize,
) -> Result<(), LexError> {
    use super::derive_types::node;
    match &mut actual.kind {
        Kind::Unary { operand, .. } => *operand = node(source, *operand, target, charge, depth)?,
        Kind::Binary { left, right, .. } => {
            *left = node(source, *left, target, charge, depth)?;
            *right = node(source, *right, target, charge, depth)?;
        }
        Kind::Cast { value, ty } => {
            *value = node(source, *value, target, charge, depth)?;
            *ty = node(source, *ty, target, charge, depth)?;
        }
        Kind::Block {
            flavor: BlockFlavor::Plain | BlockFlavor::Const,
            statements,
            tail,
        } if statements.start == statements.end => {
            *statements = List { start: 0, end: 0 };
            if let Some(tail) = tail {
                *tail = node(source, *tail, target, charge, depth)?;
            }
        }
        _ => {
            return Err(error(
                actual.span.start,
                "derived constant expression requires its wider typed subtree profile",
            ));
        }
    }
    Ok(())
}
#[cfg(test)]
#[path = "../../../tests/invariants/array_derivations.rs"]
mod source_tests;
