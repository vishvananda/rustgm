//! Original bodies move with their actual pending rows, without cloning payloads.
mod rowpay;
use crate::{
    body::{
        drops::Requirements,
        ir::{Body, Program},
    },
    resolve::expansion::Report,
    sema::model::Declarations,
    support::retained,
};
pub(in crate::lower::instances) struct Originals<'report> {
    bodies: Vec<Body>,
    requirements: Option<Box<Rows<'report>>>,
}
struct Rows<'report> {
    values: Vec<Requirements>,
    payer: rowpay::Payer<'report>,
}
impl<'report> Originals<'report> {
    pub(super) fn take(
        report: &'report Report,
        facts: &Declarations,
        program: &mut Program,
    ) -> Result<Self, &'static str> {
        let count = program.bodies.len();
        let requirements = if let Some(pending) = &mut program.drops {
            let payer = rowpay::Payer::new(report, facts)?;
            payer.charge(
                facts,
                count.saturating_mul(4).saturating_add(1),
                size_of::<Rows<'_>>(),
            )?;
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
            // The replacement is reserved exactly, so it pays its actual slots.
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(count)
                .map_err(|_| "original cleanup row assembly allocation failed")?;
            payer.charge(
                facts,
                0,
                retained::vector(&replacement).saturating_add(retained::vector(&pending.bodies)),
            )?;
            Some(Box::new(Rows {
                values: std::mem::replace(&mut pending.bodies, replacement),
                payer,
            }))
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
        if let Some(rows) = &self.requirements {
            rows.payer.charge(
                facts,
                self.bodies.len().saturating_add(rows.values.len()),
                0,
            )?;
        }
        self.bodies.reverse();
        if let Some(rows) = &mut self.requirements {
            rows.values.reverse();
        }
        Ok(())
    }
    pub(in crate::lower::instances) fn pop(
        &mut self,
    ) -> Result<Option<(Body, Option<Requirements>)>, &'static str> {
        if self
            .requirements
            .as_ref()
            .is_some_and(|r| r.values.len() != self.bodies.len())
        {
            return Err("original cleanup row ownership changed before consumption");
        }
        let Some(body) = self.bodies.pop() else {
            return Ok(None);
        };
        let row = self
            .requirements
            .as_mut()
            .map(|rows| rows.values.pop().ok_or("original cleanup row is missing"))
            .transpose()?;
        Ok(Some((body, row)))
    }
    /// Release the consumed slots of both containers once fewer than half are
    /// live. Shrinking never raises retained storage above what the builder
    /// and [`Self::take`] already paid for, so the copy pays work only: one
    /// unit per moved body and row.
    pub(in crate::lower::instances) fn shrink(
        &mut self,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        if self.bodies.len() < self.bodies.capacity() / 2 {
            if let Some(rows) = &self.requirements {
                rows.payer.charge(
                    facts,
                    self.bodies.len().saturating_add(rows.values.len()),
                    0,
                )?;
            }
            self.bodies.shrink_to_fit();
            if let Some(rows) = &mut self.requirements {
                rows.values.shrink_to_fit();
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
#[path = "../../../../tests/invariants/lowering_ledger.rs"]
mod ledger_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/shrunk_originals.rs"]
mod shrink_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/origrow.rs"]
mod tests;
