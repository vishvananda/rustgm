//! A cursor frontier visits shared type recipes once without flattening closures.
use super::{Argument, Context, Declarations, Error, Headers, RegionId, TypeId, error, s};
use crate::{
    support::crates::generics as g,
    types::{Kind, callable::Convention},
};

pub(super) fn prepare(
    context: &Context<'_>,
    facts: &mut Declarations,
    slots: &[Argument],
    regions: &[RegionId],
    budget: &mut Headers,
) -> Result<Vec<TypeId>, Error> {
    let rows = context.descriptor.rows();
    let mut nodes = vec![None; rows.types.len()];
    let mut visiting = vec![false; rows.types.len()];
    let mut cursors = vec![0_usize; rows.types.len()];
    let mut stack = Vec::with_capacity(rows.types.len());
    for root in 0..rows.types.len() {
        budget.charge(1)?;
        if nodes[root].is_some() {
            continue;
        }
        stack.push(root);
        while let Some(&index) = stack.last() {
            budget.charge(1)?;
            let row = *rows
                .types
                .get(index)
                .ok_or_else(|| error("source recipe refers outside its actual type graph"))?;
            visiting[index] = true;
            let child = next(row, cursors[index], rows)?;
            cursors[index] = cursors[index]
                .checked_add(1)
                .ok_or_else(|| error("source recipe cursor overflow"))?;
            if let Some(child) = child {
                let child = child.0 as usize;
                if nodes.get(child).is_none() {
                    return Err(error("source recipe has an unknown actual type dependency"));
                }
                if nodes[child].is_some() {
                    continue;
                }
                if visiting[child] {
                    return Err(error("source canonical type recipes contain a cycle"));
                }
                stack.push(child);
            } else {
                let mapped = intern(context, facts, slots, regions, &nodes, row, budget)?;
                nodes[index] = Some(mapped);
                visiting[index] = false;
                stack.pop();
            }
        }
    }
    nodes
        .into_iter()
        .map(|node| node.ok_or_else(|| error("source canonical graph is incomplete")))
        .collect()
}
fn next(
    row: s::Type,
    cursor: usize,
    rows: s::ValidatedView<'_>,
) -> Result<Option<g::TypeRef>, Error> {
    Ok(match row {
        s::Type::Scalar(_) | s::Type::Parameter(_) => None,
        s::Type::Reference { referent, .. } => (cursor == 0).then_some(referent),
        s::Type::Function(signature) => {
            let parameters = signature.parameters.slice(&rows.roots).map_err(error)?;
            if cursor == parameters.len() {
                Some(signature.result)
            } else {
                parameters.get(cursor).copied()
            }
        }
        s::Type::FunctionItem(_) | s::Type::ApplicationSignature(_) => {
            return Err(error(
                "source callable application needs its wider explicit remap profile",
            ));
        }
    })
}
fn intern(
    context: &Context<'_>,
    facts: &mut Declarations,
    slots: &[Argument],
    regions: &[RegionId],
    nodes: &[Option<TypeId>],
    row: s::Type,
    budget: &mut Headers,
) -> Result<TypeId, Error> {
    let child = |reference: g::TypeRef| {
        nodes
            .get(reference.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| error("source type recipe has an unresolved actual dependency"))
    };
    let rows = context.descriptor.rows();
    let kind = match row {
        s::Type::Scalar(scalar) => return Ok(super::super::super::scalar(scalar)),
        s::Type::Parameter(slot) => {
            return match slots.get(slot.0 as usize) {
                Some(Argument::Type(ty)) => Ok(*ty),
                _ => Err(error(
                    "source type parameter changed its exact canonical slot kind",
                )),
            };
        }
        s::Type::Reference {
            mutable,
            region,
            referent,
        } => Kind::Reference {
            mutable,
            region: *regions
                .get(region.0 as usize)
                .ok_or_else(|| error("source reference lost its actual regional remap"))?,
            referent: child(referent)?,
        },
        s::Type::Function(signature) => {
            if signature.convention != g::Convention::Rust || signature.variadic {
                return Err(error(
                    "source callable convention/variadic form needs a wider profile",
                ));
            }
            let frame = rows
                .binders
                .get(signature.binder.0 as usize)
                .ok_or_else(|| error("source callable has no exact actual binder"))?;
            context.definition(frame.owner, budget)?;
            let roots = signature.parameters.slice(&rows.roots).map_err(error)?;
            budget.charge(roots.len().saturating_add(1))?;
            budget.reserve(roots.len().saturating_mul(4 * size_of::<TypeId>()))?;
            let parameters = roots
                .iter()
                .map(|&root| child(root))
                .collect::<Result<Vec<_>, _>>()?;
            let result = child(signature.result)?;
            let parameters = facts.types.intern_list(&parameters).map_err(error)?;
            Kind::Function {
                bound_regions: frame.count,
                parameters,
                result,
                unsafe_: signature.unsafe_,
                convention: Convention::RUST,
            }
        }
        s::Type::FunctionItem(_) | s::Type::ApplicationSignature(_) => {
            return Err(error(
                "source application recipe escaped its explicit profile",
            ));
        }
    };
    facts.types.intern(kind).map_err(error)
}
