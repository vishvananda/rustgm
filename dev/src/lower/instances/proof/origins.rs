//! Original bodies move with their actual pending rows, without cloning payloads.
use crate::{
    body::{
        drops::Requirements,
        ir::{Body, Program},
    },
    sema::model::Declarations,
    support::retained::{self, Growth},
};
pub(in crate::lower::instances) struct Originals {
    bodies: Vec<Body>,
    requirements: Option<Vec<Requirements>>,
}
impl Originals {
    pub(super) fn take(facts: &Declarations, program: &mut Program) -> Result<Self, &'static str> {
        let count = program.bodies.len();
        facts.charge_caller_inputs(count.saturating_mul(4).saturating_add(1), 0)?;
        let requirements = if let Some(pending) = &mut program.drops {
            if pending.bodies.len() != count {
                return Err("original cleanup rows have different body coverage");
            }
            for (body, row) in program.bodies.iter().zip(&pending.bodies) {
                if row.locals.len() != body.locals.len()
                    || row.values.len() != body.values.len()
                    || row.projections.len() != body.projections.len()
                {
                    return Err("original cleanup row changed its actual body extents");
                }
            }
            // Both row containers can remain live during assembly. Their inner
            // payloads move once; no copy of a Requirements collection is made.
            let growth = Growth::vector::<Requirements>(0, 0, count);
            facts.charge_caller_inputs(
                growth.work,
                growth
                    .bytes
                    .saturating_add(retained::vector(&pending.bodies)),
            )?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(count)
                .map_err(|_| "original cleanup row assembly allocation failed")?;
            Some(std::mem::replace(&mut pending.bodies, replacement))
        } else {
            None
        };
        Ok(Self {
            bodies: std::mem::take(&mut program.bodies),
            requirements,
        })
    }
    pub(in crate::lower::instances) fn bodies(&self) -> &[Body] {
        &self.bodies
    }
    pub(in crate::lower::instances) fn reverse(
        &mut self,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        facts.charge_caller_inputs(
            self.bodies
                .len()
                .saturating_add(self.requirements.as_ref().map_or(0, Vec::len)),
            0,
        )?;
        self.bodies.reverse();
        if let Some(rows) = &mut self.requirements {
            rows.reverse();
        }
        Ok(())
    }
    pub(in crate::lower::instances) fn pop(
        &mut self,
    ) -> Result<Option<(Body, Option<Requirements>)>, &'static str> {
        if self
            .requirements
            .as_ref()
            .is_some_and(|r| r.len() != self.bodies.len())
        {
            return Err("original cleanup row ownership changed before consumption");
        }
        let Some(body) = self.bodies.pop() else {
            return Ok(None);
        };
        let row = self
            .requirements
            .as_mut()
            .map(|rows| rows.pop().ok_or("original cleanup row is missing"))
            .transpose()?;
        Ok(Some((body, row)))
    }
    pub(in crate::lower::instances) fn shrink(
        &mut self,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        if self.bodies.len() < self.bodies.capacity() / 2 {
            let rows = self.requirements.as_ref().map_or(0, Vec::len);
            facts.charge_caller_inputs(
                self.bodies.len().saturating_add(rows),
                self.bodies
                    .len()
                    .saturating_mul(size_of::<Body>())
                    .saturating_add(rows.saturating_mul(size_of::<Requirements>())),
            )?;
            self.bodies.shrink_to_fit();
            if let Some(rows) = &mut self.requirements {
                rows.shrink_to_fit();
            }
        }
        Ok(())
    }
}

/// A pending row is data, not const qualification for the separate parent path.
pub(super) fn parent_history_gap(
    facts: &Declarations,
    body: &Body,
    input: &crate::body::regions::Contract,
    requirements: Option<&Requirements>,
) -> Result<bool, crate::borrow::Error> {
    if !facts.uses_source_copy()
        || !(body.const_context || input.requires_defining_cleanup_preparation())
    {
        return Ok(false);
    }
    let Some(row) = requirements else {
        return Ok(false);
    };
    facts
        .charge_caller_inputs(
            row.locals
                .len()
                .saturating_add(row.values.len())
                .saturating_add(row.projections.len())
                .saturating_add(row.sites.len()),
            0,
        )
        .map_err(|message| crate::borrow::Error {
            node: Some(body.source.into()),
            message: message.into(),
        })?;
    Ok(!crate::borrow::loans::templates::no_cleanup(row))
}

#[cfg(test)]
#[path = "../../../../tests/invariants/origrow.rs"]
mod tests;
