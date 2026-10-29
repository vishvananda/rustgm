//! A derive occurrence copies parsed field types, never their rendered spelling.
use super::choice::Charge;
use crate::{
    lex::token::LexError,
    literal::storage::{Storage, StoredValue},
    syntax::{
        ast::{Kind, List, Node, NodeId, Syntax},
        interpolation::ParsedSyntax,
        token_tree::error,
    },
};
#[cfg(test)]
#[path = "../../../tests/invariants/constraint_images.rs"]
mod tests;

pub(super) fn field(
    source: &Syntax<StoredValue>,
    root: NodeId,
    charge: &Charge<'_>,
) -> Result<ParsedSyntax, LexError> {
    let mut copied = Syntax::default();
    copied.root = node(source, root, &mut copied, charge, 0)?;
    Ok(ParsedSyntax {
        ast: copied,
        literals: Storage::default(),
    })
}

fn node(
    source: &Syntax<StoredValue>,
    id: NodeId,
    target: &mut Syntax<StoredValue>,
    charge: &Charge<'_>,
    depth: usize,
) -> Result<NodeId, LexError> {
    charge(1, 4 * size_of::<Node>())?;
    if depth >= 256 || target.nodes.len() >= 1_000_000 {
        return Err(error(
            0,
            "derived field type exceeds the bounded syntax profile",
        ));
    }
    let mut actual = *source
        .nodes
        .get(id.get() as usize)
        .ok_or_else(|| error(0, "derived field type lost its parsed node"))?;
    match &mut actual.kind {
        Kind::Name(..) | Kind::CrateRoot(_) | Kind::NeverType => {}
        Kind::Path { segments, .. }
        | Kind::Type {
            elements: segments, ..
        } => {
            *segments = list(source, *segments, target, charge, depth + 1)?;
        }
        Kind::Applied {
            base,
            arguments,
            output,
            ..
        } => {
            *base = node(source, *base, target, charge, depth + 1)?;
            *arguments = list(source, *arguments, target, charge, depth + 1)?;
            if let Some(output) = output {
                *output = node(source, *output, target, charge, depth + 1)?;
            }
        }
        Kind::FunctionType(index) => *index = callable(source, *index, target, charge, depth + 1)?,
        Kind::Constraint {
            target: binding,
            value,
            bounds,
        } => {
            *binding = node(source, *binding, target, charge, depth + 1)?;
            if let Some(value) = value {
                *value = node(source, *value, target, charge, depth + 1)?;
            }
            *bounds = list(source, *bounds, target, charge, depth + 1)?;
        }
        Kind::FieldDefinition {
            visibility: crate::syntax::item::Visibility::Private,
            ty,
            ..
        } => {
            *ty = node(source, *ty, target, charge, depth + 1)?;
        }
        Kind::Group(value) => *value = node(source, *value, target, charge, depth + 1)?,
        Kind::Qualified {
            ty,
            trait_path,
            path,
        } => {
            *ty = node(source, *ty, target, charge, depth + 1)?;
            if let Some(trait_path) = trait_path {
                *trait_path = node(source, *trait_path, target, charge, depth + 1)?;
            }
            *path = node(source, *path, target, charge, depth + 1)?;
        }
        _ => {
            return Err(error(
                actual.span.start,
                "this parsed field type requires a wider source derive profile",
            ));
        }
    }
    let copied = NodeId::checked(
        u32::try_from(target.nodes.len())
            .map_err(|_| error(actual.span.start, "derived field node overflow"))?,
    )
    .ok_or_else(|| error(actual.span.start, "derived field node overflow"))?;
    target.nodes.push(actual);
    Ok(copied)
}

fn callable(
    source: &Syntax<StoredValue>,
    index: u32,
    target: &mut Syntax<StoredValue>,
    charge: &Charge<'_>,
    depth: usize,
) -> Result<u32, LexError> {
    charge(1, 4 * size_of::<crate::syntax::item::FunctionType>())?;
    let mut function = *source
        .function_types
        .get(index as usize)
        .ok_or_else(|| error(0, "derived callable field lost its parsed signature"))?;
    if function.abi != crate::syntax::item::Abi::Rust
        || function.binders.start != function.binders.end
    {
        return Err(error(
            0,
            "derived callable ABI or explicit binders require a wider field profile",
        ));
    }
    function.binders = list(source, function.binders, target, charge, depth + 1)?;
    function.parameters = list(source, function.parameters, target, charge, depth + 1)?;
    if let Some(result) = &mut function.result {
        *result = node(source, *result, target, charge, depth + 1)?;
    }
    let result = u32::try_from(target.function_types.len())
        .map_err(|_| error(0, "derived callable field arena overflow"))?;
    target.function_types.push(function);
    Ok(result)
}

fn list(
    source: &Syntax<StoredValue>,
    range: List,
    target: &mut Syntax<StoredValue>,
    charge: &Charge<'_>,
    depth: usize,
) -> Result<List, LexError> {
    let children = source
        .edges
        .get(range.start as usize..range.end as usize)
        .ok_or_else(|| error(0, "derived field type has an invalid parsed list"))?;
    charge(
        children.len(),
        children.len().saturating_mul(8 * size_of::<NodeId>()),
    )?;
    let mut copied = Vec::with_capacity(children.len());
    for &child in children {
        copied.push(node(source, child, target, charge, depth)?);
    }
    let start =
        u32::try_from(target.edges.len()).map_err(|_| error(0, "derived type list overflow"))?;
    target.edges.extend(copied);
    let end =
        u32::try_from(target.edges.len()).map_err(|_| error(0, "derived type list overflow"))?;
    Ok(List { start, end })
}
