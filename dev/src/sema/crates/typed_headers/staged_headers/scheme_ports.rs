//! Construct genuine callable schemes and receiver facts before any header installation.
use super::{Declarations, Error, Headers, error, mapped_source, s};
use crate::{
    sema::lifetimes::FunctionRegions,
    types::{Kind, TypeId},
};

pub(super) fn prepare(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    raw: &s::FunctionHeader,
    budget: &mut Headers,
) -> Result<FunctionRegions, Error> {
    let scheme = from_rows(context.descriptor.rows(), raw, budget, |reference, _| {
        graph.region(reference)
    })?;
    budget.observed.vector(&scheme.late);
    budget.observed.vector(&scheme.bounds);
    Ok(scheme)
}
pub(in crate::sema::crates::typed_headers) fn from_rows<B: super::super::recipe_cursor::Budget>(
    rows: s::ValidatedView<'_>,
    raw: &s::FunctionHeader,
    budget: &mut B,
    mut region: impl FnMut(s::RegionRef, &mut B) -> Result<crate::types::RegionId, Error>,
) -> Result<FunctionRegions, Error> {
    if raw.regions.captures != 0 || raw.regions.early.start != raw.regions.early.end {
        return Err(error(
            "source callable requires its wider captured/early region scheme",
        ));
    }
    let late = raw.regions.late.slice(&rows.region_slots).map_err(error)?;
    let bounds = raw
        .regions
        .bounds
        .slice(&rows.region_bounds)
        .map_err(error)?;
    budget.reserve(
        late.len().saturating_mul(size_of::<u16>()).saturating_add(
            bounds
                .len()
                .saturating_mul(size_of::<crate::types::outlives::Pair>()),
        ),
    )?;
    let mut scheme = FunctionRegions {
        total: raw.regions.total,
        captures: 0,
        early: Vec::new(),
        late: Vec::with_capacity(late.len()),
        has_declared_late: false,
        bounds: Vec::with_capacity(bounds.len()),
    };
    for slot in late {
        budget.charge(1)?;
        let slot = rows
            .slots
            .get(slot.0 as usize)
            .ok_or_else(|| error("source callable late slot is outside its actual table"))?;
        if slot.owner != raw.declaration
            || slot.kind != s::ParameterKind::Region
            || slot.index >= scheme.total
        {
            return Err(error(
                "source callable late slot changed actual owner/kind/ordinal",
            ));
        }
        if slot.definition.is_some() {
            return Err(error(
                "source named late lifetime requires its wider callable profile",
            ));
        }
        scheme.late.push(u16::try_from(slot.index).map_err(|_| {
            error("source callable late slot exceeds canonical lifetime identity space")
        })?);
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
