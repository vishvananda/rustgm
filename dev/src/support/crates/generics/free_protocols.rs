//! Genuine marker declarations and indexed const-call eligibility remain separate.
use super::{HeaderIndex, SourceRows};
use crate::support::{
    constness::Constness,
    crates::{
        generics::{Work, source as s},
        owner_catalog::DeclarationRef,
    },
};
pub(super) fn destruct(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &s::TraitHeader,
    pointee: DeclarationRef,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(12)?;
    if header.flags.bits()
        != (s::TraitFlags::CONST | s::TraitFlags::DENY_IMPL | s::TraitFlags::DYN_INCOMPATIBLE)
        || header.region_captures != 0
        || !header.region_bounds.slice(&rows.region_bounds)?.is_empty()
        || header.written_parameters != 0
        || header.canonical_inputs != 1
        || header.declared_inputs != 2
        || !header.members.slice(&rows.members)?.is_empty()
        || headers.scopes().complete_slots(header.scope, work)? != 1
    {
        return Err("source Destruct requires its exact closed const marker declaration");
    }
    let [literal] = header.written_supertraits.slice(&rows.clause_indices)? else {
        return Err("source Destruct requires one actual PointeeSized supertrait");
    };
    let clause = rows
        .clauses
        .get(*literal as usize)
        .ok_or("source Destruct supertrait input is absent")?;
    let s::Requirement::Trait {
        application,
        effect: Constness::Ordinary,
    } = clause.requirement
    else {
        return Err("source Destruct supertrait changed its ordinary effect");
    };
    let actual = rows
        .trait_applications
        .get(application.0 as usize)
        .ok_or("source Destruct supertrait application is absent")?;
    if actual.declaration
        != (s::DeclarationAddress {
            artifact: s::ArtifactRef(0),
            declaration: pointee,
        })
        || actual.subject != header.self_type
        || !actual.arguments.slice(&rows.arguments)?.is_empty()
    {
        return Err(
            "source Destruct supertrait differs from its canonical Self/PointeeSized identity",
        );
    }
    let Some(s::Type::Parameter(slot)) = rows.types.get(header.self_type.0 as usize) else {
        return Err("source Destruct requires its actual Self slot");
    };
    let slot = rows
        .slots
        .get(slot.0 as usize)
        .ok_or("source Destruct Self slot is absent")?;
    if slot.owner != header.self_definition
        || slot.definition != Some(header.self_definition)
        || slot.ordinal != 0
        || slot.index != 0
        || !matches!(slot.kind, s::ParameterKind::Type { sized: None })
    {
        return Err("source Destruct changed its unsized, unqualified Self parameter");
    }
    Ok(())
}

pub(super) fn freeze(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &s::TraitHeader,
    work: &mut Work,
) -> Result<(), &'static str> {
    let exact = header.flags.bits() == (s::TraitFlags::UNSAFE | s::TraitFlags::AUTO);
    empty_auto(rows, headers, header, exact, &FREEZE, work)
}

/// A role-bound auto marker keeps its local role's exact unsafety
/// (`compiler_items::roles`), and is rejected by name when it changes it.
#[derive(Clone, Copy)]
pub(super) struct Marker {
    unsafe_: bool,
    message: &'static str,
}
/// Send, Sync, Unpin and `UnsafeUnpin`, in `Bindings::markers` order.
pub(super) const MARKERS: [Marker; 4] = [
    Marker {
        unsafe_: true,
        message: "source Send role requires its exact auto declaration flags",
    },
    Marker {
        unsafe_: true,
        message: "source Sync role requires its exact auto declaration flags",
    },
    Marker {
        unsafe_: false,
        message: "source Unpin role requires its exact auto declaration flags",
    },
    Marker {
        unsafe_: true,
        message: "source UnsafeUnpin role requires its exact auto declaration flags",
    },
];

/// Version thirteen: any auto trait, re-proved as rustc's empty auto
/// declaration (E0567, E0568, E0380): no written parameters, supertraits,
/// members, region captures or bounds, and its own unsized Self.
pub(super) fn automatic(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &s::TraitHeader,
    marker: Option<Marker>,
    work: &mut Work,
) -> Result<(), &'static str> {
    let flags = header.flags.bits();
    let exact = flags & s::TraitFlags::AUTO != 0
        && flags & !(s::TraitFlags::UNSAFE | s::TraitFlags::AUTO) == 0;
    empty_auto(rows, headers, header, exact, &AUTOMATIC, work)?;
    match marker {
        Some(marker) if (flags & s::TraitFlags::UNSAFE != 0) != marker.unsafe_ => {
            Err(marker.message)
        }
        _ => Ok(()),
    }
}

/// Version thirteen: an impl may be unsafe or negative only when its trait is
/// an auto declaration of this same packet, whose own header admission
/// already proved it exact and empty. Another artifact's auto trait needs the
/// multi-packet profile.
pub(super) fn automatic_implementation(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    implementation: &s::ImplHeader,
    work: &mut Work,
) -> Result<bool, &'static str> {
    work.charge(2)?;
    let Some(application) = implementation.trait_application else {
        return Ok(false);
    };
    let actual = rows
        .trait_applications
        .get(application.0 as usize)
        .ok_or("source implementation trait application is absent")?;
    if actual.declaration.artifact != s::ArtifactRef(0) {
        return Err("imported foreign auto-trait impl requires its multi-packet profile");
    }
    let Some(s::HeaderRef::Trait(index)) = headers.header(actual.declaration.declaration, work)?
    else {
        return Ok(false);
    };
    Ok(rows
        .traits
        .get(index as usize)
        .ok_or("source implementation trait header is absent")?
        .flags
        .bits()
        & s::TraitFlags::AUTO
        != 0)
}

/// The diagnostics of one empty-auto-declaration check.
struct EmptyAuto {
    shape: &'static str,
    self_type: &'static str,
    slot: &'static str,
    parameter: &'static str,
}
const FREEZE: EmptyAuto = EmptyAuto {
    shape: "source Freeze requires its exact empty unsafe auto declaration",
    self_type: "source Freeze requires its actual Self slot",
    slot: "source Freeze Self slot is absent",
    parameter: "source Freeze changed its unsized, unqualified Self parameter",
};
const AUTOMATIC: EmptyAuto = EmptyAuto {
    shape: "source auto trait requires its exact empty auto declaration",
    self_type: "source auto trait requires its actual Self slot",
    slot: "source auto trait Self slot is absent",
    parameter: "source auto trait changed its unsized, unqualified Self parameter",
};
fn empty_auto(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &s::TraitHeader,
    flags: bool,
    texts: &EmptyAuto,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(8)?;
    if !flags
        || header.region_captures != 0
        || !header.region_bounds.slice(&rows.region_bounds)?.is_empty()
        || header.written_parameters != 0
        || header.canonical_inputs != 0
        || header.declared_inputs != 1
        || !header.members.slice(&rows.members)?.is_empty()
        || !header
            .written_supertraits
            .slice(&rows.clause_indices)?
            .is_empty()
        || headers.scopes().complete_slots(header.scope, work)? != 1
    {
        return Err(texts.shape);
    }
    let Some(s::Type::Parameter(slot)) = rows.types.get(header.self_type.0 as usize) else {
        return Err(texts.self_type);
    };
    let slot = rows.slots.get(slot.0 as usize).ok_or(texts.slot)?;
    if slot.owner != header.self_definition
        || slot.definition != Some(header.self_definition)
        || slot.ordinal != 0
        || slot.index != 0
        || !matches!(slot.kind, s::ParameterKind::Type { sized: None })
    {
        return Err(texts.parameter);
    }
    Ok(())
}

/// Exact defining mode is checked against its canonical function declaration.
pub(super) fn original_context(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    body: &s::Original,
    work: &mut Work,
) -> Result<s::CallContext, &'static str> {
    let Some(s::HeaderRef::Function(index)) = headers.header(body.declaration, work)? else {
        return Err("source original has no actual callable header");
    };
    let header = rows.functions[index as usize];
    if header.constness == Constness::Const {
        constant_calls(rows, headers, body, work)?;
        Ok(s::CallContext::ConstDefinition)
    } else {
        Ok(s::CallContext::Runtime)
    }
}

/// Declared constness is data, never permission to call an ordinary target.
fn constant_calls(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    body: &s::Original,
    work: &mut Work,
) -> Result<(), &'static str> {
    for request in &body.calls {
        work.charge(3)?;
        let application = rows
            .function_applications
            .get(request.target.0 as usize)
            .ok_or("source const call target application is absent")?;
        if application.declaration.artifact != s::ArtifactRef(0) {
            return Err("source const call requires its complete defining target artifact");
        }
        let Some(s::HeaderRef::Function(index)) =
            headers.header(application.declaration.declaration, work)?
        else {
            return Err("source const call target header is absent");
        };
        if rows.functions[index as usize].constness != Constness::Const {
            return Err("source const-definition call requires its actual const target");
        }
    }
    Ok(())
}
