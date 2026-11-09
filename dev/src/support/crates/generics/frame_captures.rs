//! Version thirteen: a method of an inherent impl captures the impl's lifetimes
//! as the leading slots of its original and public frames.
use super::{HeaderIndex, SourceRows};
use crate::support::{
    constness::Constness,
    crates::{
        generics::{BinderRef, TypeRef, Work, source as s},
        owner_catalog::DeclarationRef,
    },
};

/// Admits a provided method whose impl is inherent and declares lifetimes.
/// The method's frames lead with exactly the impl's captured slots, its own
/// late slots follow them, and its receiver base is the impl's Self with each
/// captured slot of the impl frame read as the same slot of the method's
/// original frame. Every consumer re-proves the method in its own rebuilt
/// environment; nothing here grants a region relation.
pub(super) fn method(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    function: &s::FunctionHeader,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let Some(parent) = function.parent else {
        return Ok(false);
    };
    let Some(s::HeaderRef::Implementation(index)) = headers.header(parent, work)? else {
        return Ok(false);
    };
    let implementation = &rows.implementations[index as usize];
    work.charge(6)?;
    // A method that claims captures under an impl that declares none is a
    // forged frame, not some other method family.
    if implementation.trait_application.is_some()
        || implementation.region_captures == 0 && function.regions.captures == 0
    {
        return Ok(false);
    }
    let scopes = headers.scopes();
    if implementation.self_definition != parent
        || implementation.constant
        || implementation.negative
        || implementation.unsafe_
        || !implementation
            .region_bounds
            .slice(&rows.region_bounds)?
            .is_empty()
        || function.constness == Constness::Conditional
        || function.regions.captures != implementation.region_captures
        || !function.regions.early.slice(&rows.region_slots)?.is_empty()
        || !function
            .regions
            .bounds
            .slice(&rows.region_bounds)?
            .is_empty()
        || scopes
            .complete_slots(implementation.scope, work)?
            .checked_sub(scopes.complete_type_slots(implementation.scope, work)?)
            != Some(implementation.region_captures)
        || scopes
            .complete_slots(function.scope, work)?
            .checked_sub(scopes.complete_type_slots(function.scope, work)?)
            != Some(function.regions.total)
    {
        return Err("source captured method changed its impl's exact lifetime frame");
    }
    if let Some(receiver) = function.receiver {
        work.charge(2)?;
        if receiver.references > 1
            || receiver.formal as usize >= function.formals.slice(&rows.formals)?.len()
        {
            return Err("source method receiver differs from its actual implementation Self");
        }
        let frames = Frames {
            implementation: implementation.self_definition,
            original: function.original.binder,
            captures: implementation.region_captures,
        };
        same_self(rows, frames, implementation.self_type, receiver.base, work)?;
    }
    Ok(true)
}

#[derive(Clone, Copy)]
struct Frames {
    /// The impl's declaration-header frame owner, which binds its captures.
    implementation: DeclarationRef,
    original: BinderRef,
    captures: u32,
}

/// Compares the impl's Self, written in its own frame, with a receiver base
/// written in the method's original frame. Shapes must agree node for node;
/// a region agrees when both are `'static` or both name the same captured slot.
fn same_self(
    rows: &SourceRows,
    frames: Frames,
    expected: TypeRef,
    actual: TypeRef,
    work: &mut Work,
) -> Result<(), &'static str> {
    let error = "source method receiver differs from its actual implementation Self";
    let mut pending = vec![(expected, actual)];
    while let Some((expected, actual)) = pending.pop() {
        work.charge(1)?;
        if expected == actual {
            continue;
        }
        let (Some(left), Some(right)) = (
            rows.types.get(expected.0 as usize),
            rows.types.get(actual.0 as usize),
        ) else {
            return Err(error);
        };
        match (*left, *right) {
            (
                s::Type::Nominal {
                    declaration: a,
                    arguments: left,
                },
                s::Type::Nominal {
                    declaration: b,
                    arguments: right,
                },
            ) => {
                let (left, right) = (left.slice(&rows.arguments)?, right.slice(&rows.arguments)?);
                if a != b || left.len() != right.len() {
                    return Err(error);
                }
                work.charge(left.len())?;
                for (left, right) in left.iter().zip(right) {
                    match (*left, *right) {
                        (s::Argument::Type(left), s::Argument::Type(right)) => {
                            pending.push((left, right));
                        }
                        (s::Argument::Region(left), s::Argument::Region(right)) => {
                            same_region(rows, frames, left, right)?;
                        }
                        _ => return Err(error),
                    }
                }
            }
            (
                s::Type::Reference {
                    mutable: a,
                    region: left,
                    referent: expected,
                },
                s::Type::Reference {
                    mutable: b,
                    region: right,
                    referent: actual,
                },
            ) if a == b => {
                same_region(rows, frames, left, right)?;
                pending.push((expected, actual));
            }
            (s::Type::Tuple(left), s::Type::Tuple(right)) => {
                let (left, right) = (left.slice(&rows.roots)?, right.slice(&rows.roots)?);
                if left.len() != right.len() {
                    return Err(error);
                }
                pending.extend(left.iter().copied().zip(right.iter().copied()));
            }
            // Version thirteen: the length terms agree exactly; a slot is the
            // same captured const parameter, as a type parameter is.
            (
                s::Type::Array {
                    element: expected,
                    length: left,
                },
                s::Type::Array {
                    element: actual,
                    length: right,
                },
            ) => {
                let lengths = rows.array_lengths();
                let (Some(left), Some(right)) =
                    (lengths.get(left.0 as usize), lengths.get(right.0 as usize))
                else {
                    return Err("source array type has no length row");
                };
                if left != right {
                    return Err(error);
                }
                pending.push((expected, actual));
            }
            _ => return Err(error),
        }
    }
    Ok(())
}

fn same_region(
    rows: &SourceRows,
    frames: Frames,
    expected: s::RegionRef,
    actual: s::RegionRef,
) -> Result<(), &'static str> {
    let (Some(left), Some(right)) = (
        rows.regions.get(expected.0 as usize),
        rows.regions.get(actual.0 as usize),
    ) else {
        return Err("source captured method receiver names an unknown region");
    };
    match (*left, *right) {
        (s::Region::Static, s::Region::Static) => Ok(()),
        // The binder validator gives each impl exactly one header frame.
        (
            s::Region::Bound {
                binder: a,
                depth: 0,
                slot: left,
            },
            s::Region::Bound {
                binder: b,
                depth: 0,
                slot: right,
            },
        ) if rows.binders.get(a.0 as usize).is_some_and(|frame| {
            frame.owner == frames.implementation && frame.kind == s::BinderKind::DeclarationHeader
        }) && b == frames.original
            && left == right
            && left < frames.captures =>
        {
            Ok(())
        }
        _ => Err("source captured method receiver changed its impl's lifetime slot"),
    }
}
