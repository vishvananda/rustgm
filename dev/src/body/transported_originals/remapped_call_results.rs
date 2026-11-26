//! A call's associated result is normalized in its rebuilt caller environment.
use crate::{
    body::ir::{Body, Rvalue},
    sema::{crates::SourceOriginal, model::Declarations},
    support::{
        crates::MAX_BYTES,
        retained::{self, Growth},
    },
    types::{Kind, RegionId, TypeId, substitute::Bound},
};

pub(crate) struct CallResult {
    declared: TypeId,
    actual: TypeId,
    late: Vec<RegionId>,
}

pub(crate) fn collect(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
    work: &mut usize,
) -> Result<Vec<CallResult>, &'static str> {
    let mut result = Vec::new();
    if !view.rows().descriptor().profile().declaration_policies() {
        return Ok(result);
    }
    let mut storage = 0_usize;
    for request in &view.raw().calls {
        *work = work.saturating_add(1);
        let Some((declared, actual, bound_regions)) = request_types(facts, body, request.value)?
        else {
            continue;
        };
        let regions = request.opening.map_or(Ok(&[][..]), |opening| {
            opening.regions.slice(&view.raw().openings)
        })?;
        if bound_regions as usize != regions.len() {
            return Err("source call result lost its actual late opening");
        }
        let growth = Growth::vector::<CallResult>(result.len(), result.capacity(), 1);
        storage = storage.saturating_add(regions.len().saturating_mul(size_of::<RegionId>()));
        if retained::vector(&result)
            .saturating_add(growth.bytes)
            .saturating_add(storage)
            > MAX_BYTES
        {
            return Err("source call result proof storage budget exceeded");
        }
        *work = work.saturating_add(growth.work);
        let late = regions
            .iter()
            .map(|&region| view.region(region, work))
            .collect::<Result<_, _>>()?;
        result
            .try_reserve(1)
            .map_err(|_| "source call result proof allocation failed")?;
        result.push(CallResult {
            declared,
            actual,
            late,
        });
    }
    Ok(result)
}

fn request_types(
    facts: &Declarations,
    body: &Body,
    value: u32,
) -> Result<Option<(TypeId, TypeId, u32)>, &'static str> {
    let Some(Rvalue::Call {
        callee, signature, ..
    }) = body.values.get(value as usize).map(|row| row.kind)
    else {
        return Err("source call result lost its actual call");
    };
    let Some(Kind::Function { result: actual, .. }) = facts.types.kind(signature) else {
        return Err("source call result lost its actual opened signature");
    };
    let Some(Kind::FunctionItem { signature, .. }) = body
        .values
        .get(callee.0 as usize)
        .and_then(|value| facts.types.kind(value.ty))
    else {
        return Err("source call result lost its actual applied item");
    };
    let Some(Kind::Function {
        bound_regions,
        result: declared,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("source call result lost its actual declared signature");
    };
    if (declared == actual && bound_regions == 0)
        || (facts.types.contains_projections(declared) == Some(false)
            && facts.types.contains_projections(actual) == Some(false))
    {
        return Ok(None);
    }
    Ok(Some((declared, actual, bound_regions)))
}

impl CallResult {
    pub(super) fn open(self, facts: &mut Declarations) -> Result<(TypeId, TypeId), &'static str> {
        if self.late.is_empty() {
            return Ok((self.declared, self.actual));
        }
        let mut bound = Bound::new(self.late);
        let opened = bound.apply(&mut facts.types, self.declared);
        facts.body_work.canonical_steps =
            facts.body_work.canonical_steps.saturating_add(bound.steps);
        Ok((opened?, self.actual))
    }
}
