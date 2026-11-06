//! Nominal coercion legality consumes actual full applications and one impl proof.
use crate::{
    sema::{model::Declarations, nominals::Proof},
    support::retained::Growth,
    types::{
        Kind, TypeId,
        aggregate::SemanticFields,
        unsizing::{CoercionField, CoercionSchema},
    },
};

pub(super) fn schema(
    facts: &mut Declarations,
    source: TypeId,
    target: TypeId,
    proof: &mut Proof,
) -> Result<CoercionSchema, &'static str> {
    let mut fields = fields(facts, source, target)?;
    let mut changed = None;
    for (index, field) in fields.iter_mut().enumerate() {
        facts.charge_caller_inputs(2, 0)?;
        let from = proof.normalize(facts, field.source)?;
        let to = proof.normalize(facts, field.target)?;
        field.source = from;
        field.target = to;
        let phantom = facts.aggregates.phantom_argument(&facts.types, from)?;
        if phantom.is_some() {
            if facts
                .aggregates
                .phantom_argument(&facts.types, to)?
                .is_none()
            {
                return Err("nominal coercion changed its actual PhantomData owner");
            }
            field.phantom = true;
            continue;
        }
        // Field equality includes every lifetime and complete application input.
        // A subtype relation would silently admit a second runtime field change.
        if from != to && changed.replace(index).is_some() {
            return Err("nominal CoerceUnsized implementation changes more than one field");
        }
    }
    let changed =
        changed.ok_or("nominal CoerceUnsized implementation requires one changed field")?;
    Ok(CoercionSchema {
        source,
        target,
        changed: u32::try_from(changed)
            .map_err(|_| "nominal coercion changed field index overflow")?,
        fields,
    })
}

fn fields(
    facts: &mut Declarations,
    source: TypeId,
    target: TypeId,
) -> Result<Vec<CoercionField>, &'static str> {
    let (Some(Kind::Adt(owner, _)), Some(Kind::Adt(other, _))) =
        (facts.types.kind(source), facts.types.kind(target))
    else {
        return Err("nominal CoerceUnsized requires two complete nominal applications");
    };
    let definition = facts
        .aggregates
        .get(owner)
        .ok_or("nominal CoerceUnsized owner fields are pending")?;
    if owner != other || definition.kind != crate::types::aggregate::Kind::Struct {
        return Err("nominal CoerceUnsized requires the same actual struct owner");
    }
    let [variant] = facts.aggregates.variants(definition) else {
        return Err("nominal CoerceUnsized struct field group changed");
    };
    let count = facts
        .types
        .list(variant.fields)
        .ok_or("nominal CoerceUnsized field list is missing")?
        .len();
    let pay =
        crate::sema::header_attributes::caller_payments(facts.must_use_contracts.as_deref(), true)
            .map_err(|_| "nominal CoerceUnsized lacks its completed source payer")?;
    let growth = Growth::vector::<CoercionField>(0, 0, count);
    pay.charge(
        growth.work.saturating_add(1),
        growth
            .bytes
            .saturating_add(2 * size_of::<SemanticFields<'_>>() + size_of::<Vec<CoercionField>>()),
    )?;
    let mut pairs = Vec::new();
    pairs
        .try_reserve(count)
        .map_err(|_| "nominal CoerceUnsized field scratch allocation failed")?;
    let mut from = facts.aggregates.semantic_fields(&facts.types, source)?;
    let mut to = facts.aggregates.semantic_fields(&facts.types, target)?;
    for index in 0..count {
        let a = from.field_paid(&mut facts.types, 0, index, &|w, b| pay.charge(w, b))?;
        let b = to.field_paid(&mut facts.types, 0, index, &|w, b| pay.charge(w, b))?;
        pairs.push(CoercionField {
            source: a,
            target: b,
            phantom: false,
        });
    }
    Ok(pairs)
}
