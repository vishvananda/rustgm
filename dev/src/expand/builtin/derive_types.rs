//! A derive occurrence copies parsed field types, never their rendered spelling.
use super::{choice::Charge, derive_constants};
use crate::{
    lex::token::LexError,
    literal::storage::{Storage, StoredValue},
    syntax::{
        ast::{Kind, List, MacroContext, Node, NodeId, Syntax},
        interpolation::ParsedSyntax,
        token_tree::{Forest, Tree, error},
    },
};
#[cfg(test)]
#[path = "../../../tests/invariants/invoked_fields.rs"]
mod call_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/embedded_counts.rs"]
mod constant_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/pool_relocations.rs"]
mod literal_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/forest_types.rs"]
mod macro_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/derived_object_fields.rs"]
mod object_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/constraint_images.rs"]
mod tests;

/// The actual owner of byte-backed literals is retained only for the selected copy.
pub(super) struct Source<'a> {
    pub ast: &'a Syntax<StoredValue>,
    owner: Option<(&'a str, &'a Storage)>,
}

#[cfg(test)]
pub(super) fn field(
    source: &Syntax<StoredValue>,
    root: NodeId,
    charge: &Charge<'_>,
) -> Result<ParsedSyntax, LexError> {
    selected(
        &Source {
            ast: source,
            owner: None,
        },
        root,
        charge,
    )
}

/// Only the actual source derive producer supplies this literal owner.
pub(super) fn owned_field(
    source: &Syntax<StoredValue>,
    text: &str,
    literals: &Storage,
    root: NodeId,
    charge: &Charge<'_>,
) -> Result<ParsedSyntax, LexError> {
    selected(
        &Source {
            ast: source,
            owner: Some((text, literals)),
        },
        root,
        charge,
    )
}

fn selected(
    source: &Source<'_>,
    root: NodeId,
    charge: &Charge<'_>,
) -> Result<ParsedSyntax, LexError> {
    let mut copied = ParsedSyntax {
        ast: Syntax::default(),
        literals: Storage::default(),
    };
    copied.ast.root = node(source, root, &mut copied, charge, 0)?;
    Ok(copied)
}

#[expect(
    clippy::too_many_lines,
    reason = "one typed field-node dispatch preserves syntax edges, source authority and payer order"
)]
pub(super) fn node(
    source: &Source<'_>,
    id: NodeId,
    target: &mut ParsedSyntax,
    charge: &Charge<'_>,
    depth: usize,
) -> Result<NodeId, LexError> {
    charge(1, 4 * size_of::<Node>())?;
    if depth >= 256 || target.ast.nodes.len() >= 1_000_000 {
        return Err(error(
            0,
            "derived field type exceeds the bounded syntax profile",
        ));
    }
    let mut actual = *source
        .ast
        .nodes
        .get(id.get() as usize)
        .ok_or_else(|| error(0, "derived field type lost its parsed node"))?;
    match &mut actual.kind {
        Kind::Name(..) | Kind::CrateRoot(_) | Kind::NeverType | Kind::DeferredLiteral(_) => {}
        Kind::Literal(index) => {
            let at = actual.span.start;
            *index = derive_constants::literal(source.ast, *index, &mut target.ast, charge, at)?;
        }
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
        Kind::Call {
            callee: base,
            arguments,
        }
        | Kind::MethodCall {
            receiver: base,
            arguments,
            ..
        } => {
            *base = node(source, *base, target, charge, depth + 1)?;
            *arguments = list(source, *arguments, target, charge, depth + 1)?;
        }
        Kind::GenericMethodCall {
            receiver,
            method,
            arguments,
        } => {
            *receiver = node(source, *receiver, target, charge, depth + 1)?;
            *method = node(source, *method, target, charge, depth + 1)?;
            *arguments = list(source, *arguments, target, charge, depth + 1)?;
        }
        Kind::Macro {
            path,
            capture,
            context: MacroContext::Type,
        } => {
            *path = node(source, *path, target, charge, depth + 1)?;
            *capture = macro_capture(source, *capture, target, charge)?;
        }
        Kind::FunctionType(index) => {
            *index = callable(source, *index, actual.span.start, target, charge, depth + 1)?;
        }
        // Object fields retain the original bounds; the derived impl's own
        // trait and lifetime checks still decide whether the field is legal.
        Kind::TraitType {
            opaque: false,
            bounds,
        } if source.owner.is_some() => {
            *bounds = list(source, *bounds, target, charge, depth + 1)?;
        }
        Kind::Bound {
            optional: false,
            constness: crate::support::constness::Constness::Ordinary,
            quantified,
            binders,
            value,
        } if source.owner.is_some() && (*quantified || binders.start == binders.end) => {
            *binders = list(source, *binders, target, charge, depth + 1)?;
            *value = node(source, *value, target, charge, depth + 1)?;
        }
        Kind::Generic(index) if source.owner.is_some() => {
            *index = bound_lifetime(source, *index, actual.span.start, target, charge)?;
        }
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
        Kind::Unary { .. } | Kind::Binary { .. } | Kind::Cast { .. } | Kind::Block { .. } => {
            derive_constants::children(source, &mut actual, target, charge, depth + 1)?;
        }
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
        u32::try_from(target.ast.nodes.len())
            .map_err(|_| error(actual.span.start, "derived field node overflow"))?,
    )
    .ok_or_else(|| error(actual.span.start, "derived field node overflow"))?;
    derive_constants::pending(actual, copied, &mut target.ast, charge)?;
    target.ast.nodes.push(actual);
    Ok(copied)
}

/// Copy an actual higher-ranked lifetime row and its exact declaration span.
fn bound_lifetime(
    source: &Source<'_>,
    index: u32,
    at: u32,
    target: &mut ParsedSyntax,
    charge: &Charge<'_>,
) -> Result<u32, LexError> {
    use crate::syntax::item::{GenericFlavor, GenericParameter};

    let lifetime = *source
        .ast
        .generics
        .get(index as usize)
        .ok_or_else(|| error(at, "derived object binder lost its parsed parameter"))?;
    if lifetime.flavor != GenericFlavor::Lifetime
        || lifetime.ty.is_some()
        || lifetime.default.is_some()
        || lifetime.bounds.start != lifetime.bounds.end
    {
        return Err(error(at, "derived object binder requires a plain lifetime"));
    }
    let growth = crate::support::retained::Growth::vector::<GenericParameter>(
        target.ast.generics.len(),
        target.ast.generics.capacity(),
        1,
    );
    charge(growth.work.saturating_add(1), growth.bytes)?;
    let result = u32::try_from(target.ast.generics.len())
        .map_err(|_| error(at, "derived object binder arena overflow"))?;
    let start = u32::try_from(target.ast.edges.len())
        .map_err(|_| error(at, "derived object binder list overflow"))?;
    target
        .ast
        .generics
        .try_reserve(1)
        .map_err(|_| error(at, "derived object binder allocation failed"))?;
    target.ast.generics.push(GenericParameter {
        bounds: List { start, end: start },
        ..lifetime
    });
    Ok(result)
}

/// Copy only this occurrence's forest. Persistent opaque handles remain owned
/// by the actual expansion registry; typed transfer maps their occurrence spans.
fn macro_capture(
    source: &Source<'_>,
    index: u32,
    target: &mut ParsedSyntax,
    charge: &Charge<'_>,
) -> Result<u32, LexError> {
    let capture = source
        .ast
        .captures
        .get(index as usize)
        .ok_or_else(|| error(0, "derived type macro lost its parsed capture"))?;
    let growth = crate::support::retained::Growth::vector::<Forest>(
        target.ast.captures.len(),
        target.ast.captures.capacity(),
        1,
    );
    charge(
        growth
            .work
            .saturating_add(capture.nodes().len())
            .saturating_add(1),
        growth
            .bytes
            .saturating_add(capture.nodes().len().saturating_mul(size_of::<Tree>())),
    )?;
    let result = u32::try_from(target.ast.captures.len())
        .map_err(|_| error(0, "derived type macro capture overflow"))?;
    target
        .ast
        .captures
        .try_reserve(1)
        .map_err(|_| error(0, "derived type macro capture allocation failed"))?;
    target.ast.captures.push(capture.map_spans(&mut Ok)?);
    Ok(result)
}

fn callable(
    source: &Source<'_>,
    index: u32,
    at: u32,
    target: &mut ParsedSyntax,
    charge: &Charge<'_>,
    depth: usize,
) -> Result<u32, LexError> {
    charge(1, 4 * size_of::<crate::syntax::item::FunctionType>())?;
    let mut function = *source
        .ast
        .function_types
        .get(index as usize)
        .ok_or_else(|| error(at, "derived callable field lost its parsed signature"))?;
    if function.binders.start != function.binders.end
        || (function.abi != crate::syntax::item::Abi::Rust && source.owner.is_none())
    {
        return Err(error(
            at,
            "derived callable ABI or explicit binders require a wider field profile",
        ));
    }
    if let crate::syntax::item::Abi::Extern(Some(literal)) = &mut function.abi {
        *literal = abi_literal(source, *literal, target, charge, at)?;
    }
    function.binders = list(source, function.binders, target, charge, depth + 1)?;
    function.parameters = list(source, function.parameters, target, charge, depth + 1)?;
    if let Some(result) = &mut function.result {
        *result = node(source, *result, target, charge, depth + 1)?;
    }
    let result = u32::try_from(target.ast.function_types.len())
        .map_err(|_| error(at, "derived callable field arena overflow"))?;
    target.ast.function_types.push(function);
    Ok(result)
}

/// Admit only the selected C ABI string, retaining its literal node and owner.
fn abi_literal(
    source: &Source<'_>,
    id: NodeId,
    target: &mut ParsedSyntax,
    charge: &Charge<'_>,
    at: u32,
) -> Result<NodeId, LexError> {
    let rejected = || {
        error(
            at,
            "derived callable ABI or explicit binders require a wider field profile",
        )
    };
    let (text, literals) = source.owner.ok_or_else(rejected)?;
    let actual = source
        .ast
        .nodes
        .get(id.get() as usize)
        .ok_or_else(rejected)?;
    let Kind::Literal(index) = actual.kind else {
        return Err(rejected());
    };
    let value = *source
        .ast
        .literals
        .get(index as usize)
        .ok_or_else(rejected)?;
    let crate::literal::Value::String(bytes) = literals
        .view(text, value)
        .map_err(|message| error(at, message))?
    else {
        return Err(rejected());
    };
    if bytes.as_ref() != b"C" {
        return Err(rejected());
    }
    let decoded = if matches!(
        value,
        StoredValue::String(crate::literal::storage::Bytes::Decoded { .. })
    ) {
        bytes.len()
    } else {
        0
    };
    let mut growth = crate::support::retained::Growth::vector::<StoredValue>(
        target.ast.literals.len(),
        target.ast.literals.capacity(),
        1,
    );
    growth.include(target.literals.decoded_growth(decoded));
    charge(
        growth.work.saturating_add(decoded).saturating_add(1),
        growth.bytes.saturating_add(4 * size_of::<Node>()),
    )?;
    target
        .ast
        .literals
        .try_reserve(1)
        .map_err(|_| error(at, "derived ABI literal allocation failed"))?;
    target
        .ast
        .nodes
        .try_reserve(1)
        .map_err(|_| error(at, "derived ABI node allocation failed"))?;
    let copied = target
        .literals
        .import(text, literals, value, 0)
        .map_err(|message| error(at, message))?;
    let index = u32::try_from(target.ast.literals.len())
        .map_err(|_| error(at, "derived ABI literal arena overflow"))?;
    let node = NodeId::checked(
        u32::try_from(target.ast.nodes.len())
            .map_err(|_| error(at, "derived ABI node overflow"))?,
    )
    .ok_or_else(|| error(at, "derived ABI node overflow"))?;
    target.ast.literals.push(copied);
    target.ast.nodes.push(Node {
        span: actual.span,
        kind: Kind::Literal(index),
    });
    Ok(node)
}

fn list(
    source: &Source<'_>,
    range: List,
    target: &mut ParsedSyntax,
    charge: &Charge<'_>,
    depth: usize,
) -> Result<List, LexError> {
    let children = source
        .ast
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
    let start = u32::try_from(target.ast.edges.len())
        .map_err(|_| error(0, "derived type list overflow"))?;
    target.ast.edges.extend(copied);
    let end = u32::try_from(target.ast.edges.len())
        .map_err(|_| error(0, "derived type list overflow"))?;
    Ok(List { start, end })
}
