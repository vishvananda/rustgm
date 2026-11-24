//! A static may retain an exact raw pointer to an already published promotion.
use super::{Declarations, Error, error};
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::{
        members::VariantId,
        values::{
            Constant, Evaluator, Instance, LocationId, Origin, Reader, View, WritePermission,
        },
    },
    support::crates::{MAX_COMPONENTS, owner_catalog::DeclarationRef},
    types::{GenericArgs, Kind, TypeId, aggregate::ShapeReadiness},
};

pub(super) struct Relocation {
    pub definition: DefId,
    pub ty: TypeId,
    pub target: DefId,
    pub anchor: DeclarationRef,
}

struct Chain {
    records: Vec<VariantId>,
    pointee: TypeId,
}

fn spend(remaining: &mut usize, work: usize) -> Result<(), &'static str> {
    *remaining = remaining
        .checked_sub(work)
        .ok_or("source static relocation exceeds its complete storage budget")?;
    Ok(())
}

fn chain(
    facts: &mut Declarations,
    ty: TypeId,
    remaining: &mut usize,
) -> Result<Option<Chain>, &'static str> {
    let mut current = ty;
    let mut records = Vec::new();
    loop {
        spend(remaining, 1)?;
        match facts.types.kind(current) {
            Some(Kind::Pointer { pointee, .. }) => return Ok(Some(Chain { records, pointee })),
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
                let before = facts.aggregates.instantiation_steps();
                let instantiated = facts.aggregates.instantiate(&mut facts.types, current);
                spend(
                    remaining,
                    facts
                        .aggregates
                        .instantiation_steps()
                        .saturating_sub(before),
                )?;
                instantiated?;
                let ShapeReadiness::Ready(shape) =
                    facts.aggregates.try_shape(&facts.types, current)?
                else {
                    return Ok(None);
                };
                let mut variants = shape.variants();
                let shape = variants
                    .next()
                    .filter(|_| variants.len() == 0)
                    .ok_or("source static relocation lost its single-field shape")?;
                let Some(&[field]) = facts.types.list(shape.fields) else {
                    return Err("source static relocation lost its single-field shape");
                };
                records.push(*variant);
                current = field;
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
    facts: &mut Declarations,
    ty: TypeId,
    mut value: Constant,
) -> Result<(LocationId, usize), &'static str> {
    let mut remaining = MAX_COMPONENTS;
    let chain = chain(facts, ty, &mut remaining)?
        .ok_or("source static relocation requires completed provider storage")?;
    let reader = Reader::new(facts);
    for variant in chain.records {
        if !matches!(reader.view(value)?, View::Product { length: 1, variant: Some(actual) } if actual == variant)
        {
            return Err("source static relocation changed its checked record fields");
        }
        value = reader.field(value, 0)?;
    }
    let location = address(reader, value, chain.pointee)?;
    Ok((location, MAX_COMPONENTS - remaining))
}

pub(super) fn finish(
    report: &Report,
    facts: &mut Declarations,
    relocation: &Relocation,
    remaining: &mut usize,
) -> Result<Constant, Error> {
    let mut work = 0;
    crate::sema::stored_fields::prepare_premise_free(facts, relocation.ty, &mut work)
        .map_err(Error::unlocated)?;
    spend(remaining, work).map_err(error)?;
    import(report, facts, relocation, remaining)?
        .ok_or_else(|| error("source static relocation requires completed consumer storage"))
}

pub(super) fn import(
    report: &Report,
    facts: &mut Declarations,
    relocation: &Relocation,
    remaining: &mut usize,
) -> Result<Option<Constant>, Error> {
    let Some(chain) = chain(facts, relocation.ty, remaining).map_err(error)? else {
        return Ok(None);
    };
    let mut evaluator = Evaluator::new(report, facts);
    let facts = evaluator.facts();
    let reader = Reader::new(facts);
    let mut value = reader
        .definition(relocation.target)
        .or_else(|| {
            reader.associated(Instance {
                definition: relocation.target,
                arguments: GenericArgs::EMPTY,
            })
        })
        .ok_or_else(|| error("imported source static relocation lost its completed constant"))?;
    let location = address(reader, value, chain.pointee).map_err(error)?;
    if facts
        .crates
        .as_deref()
        .and_then(|linkage| linkage.promoted_allocation(location))
        .is_none_or(|binding| !binding.imported || binding.anchor != relocation.anchor)
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
    Ok(Some(value))
}
