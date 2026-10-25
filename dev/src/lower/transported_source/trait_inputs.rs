//! Positive source sidecars remain unpaid beside their actual numbered rows.
use super::{Declarations, Report, Source, SourceInspection, g, s};
use crate::{
    resolve::package::Error, sema::crates::authored_export::SourceWork, support::retained,
};

/// Complete source observations, never an imported contract or execution token.
#[cfg(test)]
#[path = "../../../tests/invariants/trait_inspections.rs"]
mod tests;

pub(crate) struct TraitInspection {
    source: SourceInspection,
    contracts: s::HeaderAttributeColumns,
    contract_bytes: usize,
}
impl TraitInspection {
    fn new(
        source: SourceInspection,
        contracts: s::HeaderAttributeColumns,
        contract_bytes: usize,
    ) -> Result<Self, &'static str> {
        let inspection = Self {
            source,
            contracts,
            contract_bytes,
        };
        let minimum = size_of::<s::HeaderAttributeColumns>()
            .saturating_add(retained::vector(&inspection.contracts.functions))
            .saturating_add(retained::vector(&inspection.contracts.traits));
        if inspection.contracts.functions.len() != inspection.source.rows.functions.len()
            || inspection.contracts.traits.len() != inspection.source.rows.traits.len()
            || inspection.contract_bytes < minimum
            || inspection.contract_bytes > inspection.source.work.reserved_bytes
        {
            return Err("numbered trait inspection changed its complete sidecar ownership");
        }
        Ok(inspection)
    }
    #[cfg(test)]
    pub(crate) fn into_parts(self) -> (SourceInspection, s::HeaderAttributeColumns, usize) {
        (self.source, self.contracts, self.contract_bytes)
    }
}
impl Source {
    pub(in crate::lower) fn complete_trait_numbered(
        mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<TraitInspection, Error> {
        let before = self.work();
        let overhead = size_of::<TraitInspection>()
            .saturating_sub(size_of::<SourceInspection>())
            .saturating_sub(size_of::<s::HeaderAttributeColumns>());
        self.stage.reserve(overhead).map_err(Error::unlocated)?;
        // This same selected-header producer owns C's defaultness fence. Its
        // actual qualified hooks must be integrated before this entry is gated.
        let contracts = self.collect_header_columns(report, facts)?;
        let mut validation = g::Work {
            steps: self.work().steps,
            ..g::Work::default()
        };
        let start = validation.steps;
        let result = contracts.validate(&self.stage.rows, before.reserved_bytes, &mut validation);
        self.stage
            .charge(validation.steps.saturating_sub(start))
            .map_err(Error::unlocated)?;
        let contract_bytes = result.map_err(Error::unlocated)?;
        let live = contract_bytes.saturating_add(overhead);
        if live
            > self
                .work()
                .reserved_bytes
                .saturating_sub(before.reserved_bytes)
        {
            return Err(Error::unlocated(
                "trait sidecar exceeded its selected allocation prepayment",
            ));
        }
        self.stage.charge(3).map_err(Error::unlocated)?;
        let source = self.complete_with_extra(live).map_err(Error::unlocated)?;
        TraitInspection::new(source, contracts, contract_bytes).map_err(Error::unlocated)
    }
    pub(super) fn complete_with_extra(
        self,
        extra: usize,
    ) -> Result<SourceInspection, &'static str> {
        let (rows, mut work) = self.finish_rows()?;
        validate(&rows, &mut work, extra)?;
        Ok(SourceInspection { rows, work })
    }
}
fn validate(rows: &s::SourceRows, work: &mut SourceWork, extra: usize) -> Result<(), &'static str> {
    let mut validation = g::Work {
        steps: work.steps,
        ..g::Work::default()
    };
    let result = (|| {
        let headers = s::HeaderIndex::new(rows, extra, &mut validation)?;
        let numbering_peak = if rows.body_units.is_empty() {
            0
        } else {
            headers.numbering(&mut validation)?.peak_bytes()
        };
        let domains = headers.domains(&mut validation)?;
        let plan = domains.original_inputs(&mut validation)?;
        let previous_peak = work.reserved_bytes.saturating_add(work.scratch_bytes);
        let retained = work.reserved_bytes.max(plan.retained_bytes());
        let peak = previous_peak.max(plan.peak_bytes()).max(numbering_peak);
        // Existing Stage reservations already include the live sidecar. The
        // validator's extra base covers index coexistence without paying twice.
        work.reserved_bytes = retained;
        work.scratch_bytes = peak.saturating_sub(retained);
        drop(plan);
        drop(domains);
        drop(headers);
        Ok(())
    })();
    // Rejected lengths, indexes and work limits still consume attempted work.
    work.steps = validation.steps;
    result
}
