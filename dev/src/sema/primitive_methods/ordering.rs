//! Only the validated comparison nominal uses signed discriminant operations.
use super::{
    Candidate, Constructor, Declarations, Descriptor, Form, Operation, Primitive, Rhs, TypeId,
    infer,
};
use crate::{
    support::comparisons::Comparison,
    types::{
        GenericArgs, ItemId, Kind,
        aggregate::{self, Repr, Representation},
    },
};

pub(in crate::sema) fn register_ordering(
    facts: &mut Declarations,
    item: ItemId,
) -> Result<(), &'static str> {
    validate(facts, item)?;
    facts.associated.declare_comparison_ordering(item)
}
pub(super) fn subject(facts: &Declarations) -> Result<TypeId, &'static str> {
    let item = facts
        .associated
        .comparison_ordering()
        .ok_or("comparison Ordering identity is unavailable")?;
    validate(facts, item)
}
fn validate(facts: &Declarations, item: ItemId) -> Result<TypeId, &'static str> {
    let ty = facts
        .definitions
        .get(item.0 as usize)
        .copied()
        .flatten()
        .ok_or("comparison Ordering has no type")?;
    if facts.types.kind(ty) != Some(Kind::Adt(item, GenericArgs::EMPTY))
        || !facts.enum_casts.contains(&item)
    {
        return Err("comparison Ordering has invalid nominal identity");
    }
    let schema = facts
        .aggregates
        .get(item)
        .ok_or("comparison Ordering has no schema")?;
    let repr = Repr {
        representation: Representation::Rust,
        packed: None,
        align: None,
        tag: Some(Primitive::I8),
    };
    if schema.kind != aggregate::Kind::Enum
        || schema.repr != repr
        || !facts.aggregates.discriminants_ready(item)
    {
        return Err("comparison Ordering has invalid representation");
    }
    let variants = facts.aggregates.variants(schema);
    let members = facts.members.variants(item);
    if variants.len() != 3 || members.len() != 3 {
        return Err("comparison Ordering has invalid variants");
    }
    for ((variant, member), (ordinal, bits)) in
        variants.iter().zip(members).zip([(0, 255), (1, 0), (2, 1)])
    {
        let member = facts
            .members
            .variants
            .get(member.0 as usize)
            .ok_or("comparison Ordering member is unavailable")?;
        if variant.discriminant != bits
            || facts.types.list(variant.fields) != Some(&[])
            || member.owner != item
            || member.ordinal != ordinal
            || member.shape != crate::syntax::item::Shape::Unit
            || member.start != member.end
        {
            return Err("comparison Ordering has invalid fieldless discriminants");
        }
    }
    Ok(ty)
}
pub(super) fn candidate(
    facts: &Declarations,
    context: &mut super::Context,
    left_term: infer::TermId,
    right_term: infer::TermId,
    kind: Comparison,
) -> Result<Option<Candidate>, infer::Error> {
    let Some(item) = facts.associated.comparison_ordering() else {
        return Ok(None);
    };
    let left = context.head(&facts.types, left_term)?;
    let right = context.head(&facts.types, right_term)?;
    let mut canonical = |head, term| -> Result<bool, infer::Error> {
        Ok(match head {
            Some(Constructor::Adt(owner, GenericArgs::EMPTY)) => owner == item,
            Some(Constructor::AdtArgs(owner, _)) if owner == item => {
                context.item_arguments(&facts.types, term)?.is_empty()
            }
            _ => false,
        })
    };
    let left_canonical = canonical(left, left_term)?;
    let right_canonical = canonical(right, right_term)?;
    if !left_canonical && !right_canonical {
        return Ok(None);
    }
    if left.is_none() || right.is_none() {
        return Ok(Some(Candidate::Deferred));
    }
    if !left_canonical || !right_canonical {
        return Ok(Some(Candidate::Unrelated));
    }
    subject(facts).map_err(|_| infer::Error::InvalidIdentity)?;
    // I8 describes the validated discriminant operation, not either source operand.
    Ok(Some(Candidate::Present(Descriptor {
        operation: Operation::Ordering(kind),
        form: Form::Comparison,
        left: Primitive::I8,
        right: Primitive::I8,
        rhs: Rhs::Owned,
    })))
}
