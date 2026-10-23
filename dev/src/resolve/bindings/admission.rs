//! Source feature uses are checked once over each phase's actual configured tree.
use super::{policy::Feature, positions::{self, Position}};
use crate::{
    resolve::{
        package::{Error, Package},
        unit::{FileId, Unit},
    },
    syntax::{ast::{Kind, NodeId}, attribute::Style},
};

#[derive(Default, Debug, PartialEq, Eq)]
struct Work {
    units: usize,
    templates: usize,
    nodes: usize,
    edges: usize,
    attributes: usize,
}

enum Use {
    NeverType,
    ExpressionAttribute,
}
impl Use {
    fn feature(&self) -> Feature {
        match self {
            Self::NeverType => Feature::NeverType,
            Self::ExpressionAttribute => Feature::ExpressionAttributes,
        }
    }
    fn message(&self) -> &'static str {
        match self {
            Self::NeverType => "this source never type requires never_type",
            Self::ExpressionAttribute => "source expression attributes require stmt_expr_attributes",
        }
    }
}

/// Ordinary programs return before allocating or inspecting any syntax.
/// Loading and final expansion publish different trees; each calls this once.
pub(in crate::resolve) fn validate(package: &Package) -> Result<(), Error> {
    inspect(package, &mut Work::default())
}

fn inspect(package: &Package, work: &mut Work) -> Result<(), Error> {
    let Some(source) = package.source_core.as_deref() else { return Ok(()); };
    let mut pending = Vec::new();
    let mut children = Vec::new();
    for (index, unit) in package.units.iter().enumerate() {
        // Include templates are provenance, not additional published syntax.
        // Their installed copies are visited below their real parent edges.
        if unit.include_context.is_some() {
            work.templates += 1;
            continue;
        }
        let file = FileId(u32::try_from(index)
            .map_err(|_| Error::at(FileId(0), 0, "source file space exhausted"))?);
        work.units += 1;
        pending.push((unit.configured.ast.root,
            if index == 0 { Position::CrateRoot } else { Position::Other }));
        while let Some((node, position)) = pending.pop() {
            if !unit.configured.active[node.get() as usize] { continue; }
            work.nodes += 1;
            let kind = unit.configured.ast.node(node).kind;
            if matches!(kind, Kind::NeverType) && position != Position::FunctionResult {
                require(source, Use::NeverType, unit, file, node)?;
            }
            attributes(source, unit, file, node, position, work)?;
            children.clear();
            unit.configured.ast.push_children(node, &mut children);
            work.edges += children.len();
            pending.extend(children.iter().rev().map(|child| {
                (*child, positions::child(&unit.configured.ast, node, position, *child))
            }));
        }
    }
    Ok(())
}

fn require(
    source: &super::SourceCore,
    usage: Use,
    unit: &Unit,
    file: FileId,
    node: NodeId,
) -> Result<(), Error> {
    if source.features.has(usage.feature()) { return Ok(()); }
    Err(Error::at(file, unit.configured.ast.node(node).span.start, usage.message()))
}

fn attributes(
    source: &super::SourceCore,
    unit: &Unit,
    file: FileId,
    node: NodeId,
    position: Position,
    work: &mut Work,
) -> Result<(), Error> {
    let Kind::Attributed { .. } = unit.configured.ast.node(node).kind else { return Ok(()); };
    let Some(index) = unit.owner_indices[node.get() as usize] else {
        return Err(Error::at(file, unit.configured.ast.node(node).span.start,
            "source attribute owner has no configured identity"));
    };
    let owner = &unit.configured.owners[index.get() as usize - 1];
    if !owner.enabled || owner.node != node {
        return Err(Error::at(file, unit.configured.ast.node(node).span.start,
            "source attribute has a different configured owner"));
    }
    for (attribute, forest) in unit.retained_attributes(owner.attributes.clone()) {
        work.attributes += 1;
        if attribute.name(forest, unit.source.text()) == Some("feature") {
            if position != Position::CrateRoot || attribute.style != Style::Inner {
                return Err(Error::at(file, attribute.span.start,
                    "source features may be requested only at the actual crate root"));
            }
        } else if position == Position::Expression {
            require(source, Use::ExpressionAttribute, unit, file, node)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/feature_admission.rs"]
mod tests;
