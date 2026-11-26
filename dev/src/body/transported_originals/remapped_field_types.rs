//! Imported nominal fields prove their types after interpreting the declaration.
use crate::{
    body::{
        ir::{Body, Projection, ProjectionKind, Rvalue, ValueList},
        regions::{Contract, Proof},
    },
    sema::{
        members::{Selection, VariantId},
        model::Declarations,
    },
    support::{
        crates::MAX_BYTES,
        retained::{self, Growth},
    },
    types::{
        Kind, TypeId,
        parameters::{
            environment::Environment,
            predicates::{Bound, Predicate},
        },
    },
};
use std::sync::Arc;

pub(crate) fn prove(
    facts: &mut Declarations,
    body: &Body,
    contract: &Contract,
    environment: &Arc<Environment>,
    calls: Vec<super::remapped_call_results::CallResult>,
) -> Result<Option<Proof>, &'static str> {
    let mut goals = Vec::new();
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(body.projections.len());
    for projection in &body.projections {
        let ProjectionKind::Field(Selection::Nominal(field)) = projection.kind else {
            continue;
        };
        let declared = field_type(facts, body, projection, field)?;
        add_goal(&mut goals, declared, projection.ty)?;
    }
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(body.values.len());
    for value in &body.values {
        if let Rvalue::Aggregate { variant, fields } = value.kind {
            aggregate_goals(facts, body, value.ty, variant, fields, &mut goals)?;
        }
    }
    for call in calls {
        let (declared, actual) = call.open(facts)?;
        add_goal(&mut goals, declared, actual)?;
    }
    if goals.is_empty() {
        return Ok(None);
    }
    let first = crate::body::dispatch::imported_region_frontier(facts, contract)?;
    let proof = crate::sema::compiler_items::prove_body_requirements(
        facts,
        Some(Arc::clone(environment)),
        contract.universals,
        first,
        goals.into_iter(),
    )?;
    Ok((!proof.is_empty()).then(|| Proof::new(body.source, proof)))
}

fn add_goal(
    goals: &mut Vec<Predicate>,
    declared: TypeId,
    actual: TypeId,
) -> Result<(), &'static str> {
    if declared != actual {
        let growth = Growth::vector::<Predicate>(goals.len(), goals.capacity(), 1);
        if retained::vector(goals).saturating_add(growth.bytes) > MAX_BYTES {
            return Err("source field relation storage budget exceeded");
        }
        goals
            .try_reserve(1)
            .map_err(|_| "source field relation allocation failed")?;
        goals.push(Predicate {
            ty: declared,
            bound: Bound::ProjectionEqual(actual),
        });
    }
    Ok(())
}

fn aggregate_goals(
    facts: &mut Declarations,
    body: &Body,
    ty: TypeId,
    variant: Option<VariantId>,
    fields: ValueList,
    goals: &mut Vec<Predicate>,
) -> Result<(), &'static str> {
    let Some(Kind::Adt(owner, _)) = facts.types.kind(ty) else {
        return Ok(());
    };
    let variant = if let Some(variant) = variant {
        let declared = facts
            .members
            .variants
            .get(variant.0 as usize)
            .filter(|entry| entry.owner == owner)
            .ok_or("source aggregate proof changed its actual variant owner")?;
        declared.ordinal as usize
    } else {
        0
    };
    let declared = facts
        .aggregates
        .raw_shape(&facts.types, ty)?
        .variant(variant)
        .and_then(|variant| facts.types.list(variant.fields))
        .ok_or("source aggregate proof has no actual declared fields")?;
    let actual = body
        .arguments
        .get(fields.start as usize..fields.end as usize)
        .filter(|actual| actual.len() == declared.len())
        .ok_or("source aggregate proof changed its complete field count")?;
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(declared.len());
    for (&declared, &value) in declared.iter().zip(actual) {
        let actual = body
            .values
            .get(value.0 as usize)
            .ok_or("source aggregate proof has no actual field value")?
            .ty;
        add_goal(goals, declared, actual)?;
    }
    Ok(())
}

fn field_type(
    facts: &Declarations,
    body: &Body,
    projection: &Projection,
    field: crate::sema::members::FieldId,
) -> Result<TypeId, &'static str> {
    let base = body
        .place_type(projection.base)
        .ok_or("source field proof has no actual base type")?;
    let identity = facts
        .members
        .identity(field)
        .ok_or("source field proof has no actual member identity")?;
    if !matches!(facts.types.kind(base), Some(Kind::Adt(item, _)) if item.0 == identity.owner.0) {
        return Err("source field proof changed its actual nominal owner");
    }
    facts
        .aggregates
        .raw_shape(&facts.types, base)?
        .variant(identity.variant as usize)
        .and_then(|variant| facts.types.list(variant.fields))
        .and_then(|fields| fields.get(identity.field as usize))
        .copied()
        .ok_or("source field proof lost its actual field declaration")
}

/// A freshly rebuilt body needs actual storage before inspecting drops and
/// promotion. Its fields can contain projections absent from the nominal type.
pub(super) fn prepare_storage(facts: &mut Declarations, body: &Body) -> Result<(), String> {
    let mut work = 0;
    let mut prepare = |ty| prepare_type(facts, ty, &mut work);
    let result = (|| {
        for ty in std::iter::once(body.result)
            .chain(body.locals.iter().map(|local| local.ty))
            .chain(body.values.iter().map(|value| value.ty))
            .chain(body.projections.iter().map(|projection| projection.ty))
        {
            prepare(ty)?;
        }
        for value in &body.values {
            if let crate::body::ir::Rvalue::Unsize { conversion, .. } = value.kind {
                let _ = conversion.try_map(&mut prepare)?;
            }
        }
        Ok(())
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    result
}

fn prepare_type(facts: &mut Declarations, ty: TypeId, work: &mut usize) -> Result<TypeId, String> {
    if matches!(facts.types.kind(ty), Some(Kind::Adt(..))) {
        facts.aggregates.prepare(&mut facts.types, ty)?;
        if facts.aggregates.has_projected_fields(ty)
            && facts.types.contains_parameters(ty) == Some(false)
            && facts.aggregates.shape(&facts.types, ty).is_err()
        {
            crate::sema::stored_fields::prepare_premise_free(facts, ty, work)?;
        }
    }
    Ok(ty)
}
