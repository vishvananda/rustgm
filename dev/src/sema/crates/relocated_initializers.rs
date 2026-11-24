//! A static may retain an exact raw pointer to an already published promotion.
use super::{Declarations, Error, error};
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::{
        members::VariantId,
        values::{
            Constant, Evaluator, Failure, Instance, LocationId, Origin, Reader, View,
            WritePermission,
        },
    },
    support::crates::{MAX_COMPONENTS, owner_catalog::DeclarationRef},
    types::{GenericArgs, Kind, TypeId},
};

struct Chain {
    records: Vec<VariantId>,
    pointee: TypeId,
    work: usize,
}

fn chain(evaluator: &mut Evaluator<'_>, ty: TypeId, limit: usize) -> Result<Chain, &'static str> {
    let (_, work) = evaluator
        .storage_layout(ty, limit)
        .map_err(Failure::message)?;
    let facts = evaluator.facts();
    let mut current = ty;
    let mut records = Vec::new();
    let mut steps = work;
    loop {
        steps = steps.saturating_add(1);
        if steps > limit {
            return Err("source static relocation exceeds its complete storage budget");
        }
        match facts.types.kind(current) {
            Some(Kind::Pointer { pointee, .. }) => {
                return Ok(Chain {
                    records,
                    pointee,
                    work: steps,
                });
            }
            Some(Kind::Adt(owner, _)) => {
                let [variant] = facts.members.variants(owner) else {
                    return Err("source static relocation requires a single-field record");
                };
                if !facts
                    .aggregates
                    .get(owner)
                    .is_some_and(|record| record.kind == crate::types::aggregate::Kind::Struct)
                    || facts
                        .members
                        .fields(*variant)
                        .is_none_or(|fields| fields.len() != 1)
                {
                    return Err("source static relocation requires a single-field record");
                }
                records.push(*variant);
                current = Reader::new(facts).projection_layout(current, 0)?.ty;
            }
            _ => return Err("source static relocation requires a raw pointer terminal"),
        }
    }
}

fn address(
    reader: Reader<'_>,
    value: Constant,
    pointee: TypeId,
) -> Result<LocationId, &'static str> {
    let location = value
        .location()
        .ok_or("source static relocation has no actual address")?;
    let stored = reader.location(location)?;
    if stored.ty != pointee
        || !matches!(stored.origin, Origin::Allocation)
        || stored.value.is_none()
        || value.metadata().is_some()
        || value.write_permission() != Some(WritePermission::SharedDerived)
    {
        return Err("source static relocation changed the actual shared allocation");
    }
    Ok(location)
}

pub(super) fn checked_pointer(
    report: &Report,
    facts: &mut Declarations,
    ty: TypeId,
    mut value: Constant,
) -> Result<(LocationId, usize), &'static str> {
    let chain = chain(&mut Evaluator::new(report, facts), ty, MAX_COMPONENTS)?;
    let reader = Reader::new(facts);
    for variant in chain.records {
        if !matches!(reader.view(value)?, View::Product { length: 1, variant: Some(actual) } if actual == variant)
        {
            return Err("source static relocation changed its checked record fields");
        }
        value = reader.field(value, 0)?;
    }
    let location = address(reader, value, chain.pointee)?;
    Ok((location, chain.work))
}

pub(super) fn import(
    report: &Report,
    facts: &mut Declarations,
    ty: TypeId,
    target: DefId,
    anchor: DeclarationRef,
    remaining: &mut usize,
) -> Result<Constant, Error> {
    let mut work = 0;
    crate::sema::stored_fields::prepare_premise_free(facts, ty, &mut work)
        .map_err(Error::unlocated)?;
    *remaining = remaining
        .checked_sub(work)
        .ok_or_else(|| error("source static relocation exceeds its complete storage budget"))?;
    let mut evaluator = Evaluator::new(report, facts);
    let chain = chain(&mut evaluator, ty, *remaining).map_err(error)?;
    *remaining -= chain.work;
    let facts = evaluator.facts();
    let reader = Reader::new(facts);
    let mut value = reader
        .definition(target)
        .or_else(|| {
            reader.associated(Instance {
                definition: target,
                arguments: GenericArgs::EMPTY,
            })
        })
        .ok_or_else(|| error("imported source static relocation lost its completed constant"))?;
    let location = address(reader, value, chain.pointee).map_err(error)?;
    if facts
        .crates
        .as_deref()
        .and_then(|linkage| linkage.promoted_allocation(location))
        .is_none_or(|binding| !binding.imported || binding.anchor != anchor)
    {
        return Err(error(
            "imported source static relocation changed its provider's actual allocation",
        ));
    }
    for variant in chain.records.into_iter().rev() {
        value = evaluator
            .fields(&[value], Some(variant))
            .map_err(|failure| error(failure.message()))?;
    }
    Ok(value)
}
