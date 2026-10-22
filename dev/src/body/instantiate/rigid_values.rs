//! Rigid owner arguments cannot reach an executable value or skipped dependency.
use crate::{
    body::ir::{Body, Rvalue},
    sema::{model::Declarations, values::Instance},
    types::{
        TypeId,
        arguments::{Argument, ConstKind},
    },
};

pub(super) fn check(facts: &mut Declarations, body: &Body) -> Result<(), &'static str> {
    let mut work = 0_usize;
    let result = (|| {
        closed(facts, body.result, &mut work)?;
        for local in &body.locals {
            closed(facts, local.ty, &mut work)?;
        }
        for projection in &body.projections {
            closed(facts, projection.ty, &mut work)?;
        }
        for value in &body.values {
            closed(facts, value.ty, &mut work)?;
            match value.kind {
                Rvalue::AssociatedConstant(instance) => arguments(facts, instance, &mut work)?,
                Rvalue::SourceAssociated(source) => {
                    arguments(facts, source.identity(facts)?.0, &mut work)?;
                }
                Rvalue::Intrinsic { instance, .. } => closed(facts, instance.ty(), &mut work)?,
                Rvalue::Call { signature, .. } | Rvalue::Invoke { signature, .. } => {
                    closed(facts, signature, &mut work)?;
                }
                Rvalue::InlineInstance(_) => {
                    return Err(
                        "rigid constant pattern requires a parameter-independent inline dependency",
                    );
                }
                Rvalue::ConstArgument(id)
                    if !matches!(facts.types.const_kind(id), Some(ConstKind::Value(_))) =>
                {
                    return Err("constant pattern depends on a rigid const parameter");
                }
                _ => {}
            }
        }
        Ok(())
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    result
}
fn closed(facts: &Declarations, ty: TypeId, work: &mut usize) -> Result<(), &'static str> {
    visit(work)?;
    if facts.types.contains_parameters(ty) != Some(false)
        || facts.types.contains_projections(ty) != Some(false)
    {
        return Err("constant pattern depends on a rigid type parameter");
    }
    Ok(())
}
fn arguments(
    facts: &Declarations,
    instance: Instance,
    work: &mut usize,
) -> Result<(), &'static str> {
    for argument in facts
        .types
        .arguments(instance.arguments)
        .ok_or("invalid rigid constant dependency")?
    {
        visit(work)?;
        match argument {
            Argument::Type(ty) => closed(facts, ty, work)?,
            Argument::Const(id)
                if !matches!(facts.types.const_kind(id), Some(ConstKind::Value(_))) =>
            {
                return Err("constant pattern depends on a rigid const parameter");
            }
            Argument::Lifetime(_) | Argument::Const(_) => {}
        }
    }
    Ok(())
}
fn visit(work: &mut usize) -> Result<(), &'static str> {
    *work = work.saturating_add(1);
    if *work > 1_000_000 {
        return Err("rigid constant footprint work budget exceeded");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/rigid_footprints.rs"]
mod tests;
