//! Construct genuine callable schemes and receiver facts before any header installation.
use super::{Declarations, Error, Headers, error, mapped_source, s};
use crate::{
    sema::lifetimes::FunctionRegions,
    support::constness::Constness,
    types::{Kind, TypeId},
};

pub(super) fn prepare(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    raw: &s::FunctionHeader,
    scope: &s::ParameterScope,
    budget: &mut Headers,
) -> Result<FunctionRegions, Error> {
    let scheme = from_rows(
        context.descriptor.rows(),
        raw,
        Some(scope),
        budget,
        |reference, _| graph.region(reference),
        |declaration, budget| context.definition(declaration, budget),
    )?;
    budget.observed.vector(&scheme.early);
    budget.observed.vector(&scheme.late);
    budget.observed.vector(&scheme.bounds);
    Ok(scheme)
}
pub(in crate::sema::crates::typed_headers) fn from_rows<B: super::super::recipe_cursor::Budget>(
    rows: s::ValidatedView<'_>,
    raw: &s::FunctionHeader,
    scope: Option<&s::ParameterScope>,
    budget: &mut B,
    mut region: impl FnMut(s::RegionRef, &mut B) -> Result<crate::types::RegionId, Error>,
    mut definition: impl FnMut(
        crate::support::crates::owner_catalog::DeclarationRef,
        &mut B,
    ) -> Result<crate::resolve::model::DefId, Error>,
) -> Result<FunctionRegions, Error> {
    let captures = raw.regions.captures;
    let wide = rows.descriptor().profile().declaration_policies();
    if captures != 0 && !associated_captures(rows.descriptor(), raw)
        || (!wide && raw.regions.early.start != raw.regions.early.end)
    {
        return Err(error(
            "source callable requires its wider captured/early region scheme",
        ));
    }
    let early = raw.regions.early.slice(&rows.region_slots).map_err(error)?;
    let late = raw.regions.late.slice(&rows.region_slots).map_err(error)?;
    let bounds = raw
        .regions
        .bounds
        .slice(&rows.region_bounds)
        .map_err(error)?;
    budget.reserve(
        early
            .len()
            .saturating_mul(size_of::<(crate::resolve::model::DefId, u16)>())
            .saturating_add(late.len().saturating_mul(size_of::<u16>()))
            .saturating_add(
                bounds
                    .len()
                    .saturating_mul(size_of::<crate::types::outlives::Pair>()),
            ),
    )?;
    let mut scheme = FunctionRegions {
        total: raw.regions.total,
        captures,
        early: Vec::with_capacity(early.len()),
        late: Vec::with_capacity(late.len()),
        has_declared_late: false,
        bounds: Vec::with_capacity(bounds.len()),
    };
    decode_early(rows, raw, early, &mut scheme, budget, &mut definition)?;
    for slot in late {
        budget.charge(1)?;
        let slot = checked_region_slot(rows, raw, *slot, captures, scheme.total)?;
        if slot.definition.is_some() {
            if !wide {
                return Err(error(
                    "source named late lifetime requires its wider callable profile",
                ));
            }
            scheme.has_declared_late = true;
        }
        scheme.late.push(u16::try_from(slot.index).map_err(|_| {
            error("source callable late slot exceeds canonical lifetime identity space")
        })?);
    }
    if wide
        && !scheme.has_declared_late
        && (captures as usize)
            .saturating_add(early.len())
            .saturating_add(late.len())
            < scheme.total as usize
        && let Some(scope) = scope
    {
        scheme.has_declared_late = has_unused_declared_late(rows, scope, &scheme, budget)?;
    }
    for bound in bounds {
        budget.charge(1)?;
        scheme.bounds.push((
            region(bound.longer, budget)?,
            region(bound.shorter, budget)?,
        ));
    }
    Ok(scheme)
}

fn checked_region_slot(
    rows: s::ValidatedView<'_>,
    raw: &s::FunctionHeader,
    reference: s::SlotRef,
    captures: u32,
    total: u32,
) -> Result<s::ParameterSlot, Error> {
    let slot = rows
        .slots
        .get(reference.0 as usize)
        .copied()
        .ok_or_else(|| error("source callable late slot is outside its actual table"))?;
    if slot.owner != raw.declaration
        || slot.kind != s::ParameterKind::Region
        || slot.index < captures
        || slot.index >= total
    {
        return Err(error(
            "source callable late slot changed actual owner/kind/ordinal",
        ));
    }
    Ok(slot)
}

fn decode_early<B: super::super::recipe_cursor::Budget>(
    rows: s::ValidatedView<'_>,
    raw: &s::FunctionHeader,
    early: &[s::SlotRef],
    scheme: &mut FunctionRegions,
    budget: &mut B,
    definition: &mut impl FnMut(
        crate::support::crates::owner_catalog::DeclarationRef,
        &mut B,
    ) -> Result<crate::resolve::model::DefId, Error>,
) -> Result<(), Error> {
    for &reference in early {
        budget.charge(1)?;
        let slot = checked_region_slot(rows, raw, reference, scheme.captures, scheme.total)?;
        let binding = slot
            .definition
            .ok_or_else(|| error("source callable early slot omitted its declared lifetime"))?;
        let index = u16::try_from(slot.index).map_err(|_| {
            error("source callable early slot exceeds canonical lifetime identity space")
        })?;
        scheme.early.push((definition(binding, budget)?, index));
    }
    Ok(())
}

fn has_unused_declared_late<B: super::super::recipe_cursor::Budget>(
    rows: s::ValidatedView<'_>,
    scope: &s::ParameterScope,
    scheme: &FunctionRegions,
    budget: &mut B,
) -> Result<bool, Error> {
    for slot in scope.declared.slice(&rows.slots).map_err(error)? {
        budget.charge(1)?;
        if slot.kind != s::ParameterKind::Region
            || slot.index < scheme.captures
            || slot.definition.is_none()
        {
            continue;
        }
        budget.charge(scheme.early.len())?;
        if !scheme
            .early
            .iter()
            .any(|&(_, index)| u32::from(index) == slot.index)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// A captured callable opens the verified leading slots from its true owner.
pub(in crate::sema::crates::typed_headers) fn associated_captures(
    descriptor: s::Descriptor<'_>,
    raw: &s::FunctionHeader,
) -> bool {
    captures_of(descriptor, raw, descriptor.profile().declaration_policies())
}

/// Inherent captures may authorize an existing const inherent original.
pub(in crate::sema::crates::typed_headers) fn inherent_captures(
    descriptor: s::Descriptor<'_>,
    raw: &s::FunctionHeader,
) -> bool {
    captures_of(descriptor, raw, false)
}

fn captures_of(descriptor: s::Descriptor<'_>, raw: &s::FunctionHeader, trait_frames: bool) -> bool {
    let rows = descriptor.rows();
    descriptor.profile().body_nodes()
        && raw.parent.is_some_and(|parent| match rows.header(parent) {
            Ok(Some(s::HeaderRef::Implementation(index))) => rows
                .implementations
                .get(index as usize)
                .is_some_and(|owner| {
                    owner.self_definition == parent
                        && owner.region_captures == raw.regions.captures
                        && (trait_frames || owner.trait_application.is_none())
                }),
            Ok(Some(s::HeaderRef::Trait(index))) if trait_frames => {
                rows.traits.get(index as usize).is_some_and(|owner| {
                    owner.declaration == parent && owner.region_captures == raw.regions.captures
                })
            }
            _ => false,
        })
}

pub(super) fn transported_const_impl(descriptor: s::Descriptor<'_>, owner: &s::ImplHeader) -> bool {
    // The version-fourteen validator authenticates const trait and inherent impls.
    descriptor.profile().declaration_policies()
        && owner.constant
        && (owner.polarity == crate::sema::associated::ImplPolarity::Reservation
            || owner.polarity.is_positive())
}

pub(in crate::sema::crates::typed_headers) fn const_impl_body(
    descriptor: s::Descriptor<'_>,
    raw: &s::FunctionHeader,
) -> bool {
    let rows = descriptor.rows();
    raw.parent.is_some_and(|parent| {
        matches!(rows.header(parent), Ok(Some(s::HeaderRef::Implementation(index)))
        if rows.implementations.get(index as usize).is_some_and(|owner| {
            owner.self_definition == parent && transported_const_impl(descriptor, owner)
        }))
    })
}

pub(super) fn callable_effect(
    descriptor: s::Descriptor<'_>,
    raw: &s::FunctionHeader,
    budget: &mut Headers,
) -> Result<bool, Error> {
    let constant = raw.constness == Constness::Const;
    let implementation_const = if constant && raw.parent.is_some() {
        budget.charge(1)?;
        const_impl_body(descriptor, raw)
    } else {
        false
    };
    // Sealed conditional rows name actual required methods or Drop defaults.
    let conditional = raw.constness == Constness::Conditional
        && descriptor.profile().body_nodes()
        && raw.parent.is_some()
        && (raw.body == s::FunctionBody::Absent
            || (descriptor.profile().declaration_policies()
                && matches!(raw.body, s::FunctionBody::Original(_))));
    if raw.regions.captures != 0 && !associated_captures(descriptor, raw)
        || (raw.regions.early.start != raw.regions.early.end
            && !descriptor.profile().declaration_policies())
        || (raw.constness != Constness::Ordinary
            && !conditional
            && !(constant
                && descriptor.profile().unconditional_callables()
                && (raw.parent.is_none()
                    || implementation_const
                    || inherent_captures(descriptor, raw))))
    {
        return Err(error(
            "source callable captures/effects require their wider header profile",
        ));
    }
    Ok(constant)
}

pub(super) fn receiver(
    graph: &mapped_source::Graph,
    facts: &Declarations,
    raw: &s::FunctionHeader,
    signature: TypeId,
    budget: &mut Headers,
) -> Result<Option<(TypeId, u32)>, Error> {
    receiver_from(&facts.types, raw, signature, budget, |reference, _| {
        graph.ty(reference)
    })
}
pub(in crate::sema::crates::typed_headers) fn receiver_from<
    B: super::super::recipe_cursor::Budget,
>(
    store: &crate::types::Store,
    raw: &s::FunctionHeader,
    signature: TypeId,
    budget: &mut B,
    mut mapped: impl FnMut(crate::support::crates::generics::TypeRef, &mut B) -> Result<TypeId, Error>,
) -> Result<Option<(TypeId, u32)>, Error> {
    let Some(receiver) = raw.receiver else {
        return Ok(None);
    };
    budget.charge(1)?;
    if raw.parent.is_none() || receiver.formal != 0 {
        return Err(error(
            "source receiver requires its actual first associated formal",
        ));
    }
    let Some(Kind::Function { parameters, .. }) = store.kind(signature) else {
        return Err(error(
            "source receiver lacks its actual original function signature",
        ));
    };
    let actual = *store
        .list(parameters)
        .and_then(|inputs| inputs.first())
        .ok_or_else(|| error("source receiver omitted its actual signature input"))?;
    let mut ty = actual;
    for _ in 0..receiver.references {
        budget.charge(1)?;
        let Some(Kind::Reference { referent, .. }) = store.kind(ty) else {
            return Err(error(
                "source receiver references disagree with its actual signature",
            ));
        };
        ty = referent;
    }
    let base = mapped(receiver.base, budget)?;
    if ty != base {
        return Err(error(
            "source receiver base disagrees with its actual original signature",
        ));
    }
    Ok(Some((actual, receiver.references)))
}
