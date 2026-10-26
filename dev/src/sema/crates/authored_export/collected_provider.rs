//! Actual source completion keeps rows and advisory contracts under one owner.
use super::{Declarations, Report, SourceWork, Stage, g, s};
use crate::{
    resolve::package::Error,
    support::{crates::CrateIdentity, retained},
};
#[path = "collected_inventory.rs"]
mod inventory;
pub(crate) use inventory::{CollectedArtifact, CollectedReceipt};

/// Producer-local Stage IDs retire before this arena-independent data can move.
pub(crate) struct CollectedStage {
    stage: Stage,
    provider: CrateIdentity,
    contracts: s::HeaderAttributeColumns,
    contract_bytes: usize,
    live: usize,
}
/// No unchecked constructor or mutable row/contract access is exposed.
pub(crate) struct CollectedRows {
    rows: s::SourceRows,
    work: SourceWork,
    provider: CrateIdentity,
    contracts: s::HeaderAttributeColumns,
    contract_bytes: usize,
}
impl Stage {
    pub(crate) fn collect_provider_headers(
        mut self,
        report: &Report,
        facts: &Declarations,
        check: impl FnMut(&mut Stage, crate::resolve::unit::NodeKey) -> Result<(), Error>,
    ) -> Result<CollectedStage, Error> {
        let before = self.work();
        let overhead = size_of::<CollectedRows>()
            .saturating_sub(size_of::<s::SourceRows>())
            .saturating_sub(size_of::<SourceWork>())
            .saturating_sub(size_of::<s::HeaderAttributeColumns>());
        self.reserve(overhead).map_err(Error::unlocated)?;
        let contracts = self.collect_header_columns_checked(report, facts, check)?;
        let mut validation = g::Work {
            steps: self.work().steps,
            ..g::Work::default()
        };
        let start = validation.steps;
        let result = contracts.validate(&self.rows, before.reserved_bytes, &mut validation);
        self.charge(validation.steps.saturating_sub(start))
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
        let provider = &report
            .package
            .source_core()
            .ok_or_else(|| Error::unlocated("collected source headers lost their actual provider"))?
            .provider;
        self.charge(5).map_err(Error::unlocated)?;
        Ok(CollectedStage {
            stage: self,
            provider: provider.clone(),
            contracts,
            contract_bytes,
            live,
        })
    }
}
impl CollectedStage {
    pub(crate) fn finish(
        self,
        numbering: Option<Vec<Option<s::BodyUnit>>>,
    ) -> Result<CollectedRows, &'static str> {
        let (mut rows, mut work) = self.stage.into_rows()?;
        if let Some(numbering) = numbering {
            let mut units = Vec::with_capacity(numbering.len());
            for unit in numbering {
                units.push(unit.ok_or("source numbering omitted an actual original")?);
            }
            rows.body_units = units;
        }
        validate_collected_rows(&rows, &mut work, self.live)?;
        let collected = CollectedRows {
            rows,
            work,
            provider: self.provider,
            contracts: self.contracts,
            contract_bytes: self.contract_bytes,
        };
        let minimum = size_of::<s::HeaderAttributeColumns>()
            .saturating_add(retained::vector(&collected.contracts.functions))
            .saturating_add(retained::vector(&collected.contracts.traits));
        if collected.contracts.functions.len() != collected.rows.functions.len()
            || collected.contracts.traits.len() != collected.rows.traits.len()
            || collected.contract_bytes < minimum
            || collected.contract_bytes > collected.work.reserved_bytes
            || collected
                .rows
                .declarations
                .first()
                .and_then(|row| row.name.as_deref())
                != Some(collected.provider.name())
        {
            return Err("numbered trait inspection changed its complete sidecar ownership");
        }
        Ok(collected)
    }
}
impl CollectedRows {
    /// Raw development inspection consumes and discards source completion.
    #[cfg(test)]
    pub(crate) fn into_unpaid(
        self,
    ) -> (s::SourceRows, SourceWork, s::HeaderAttributeColumns, usize) {
        (self.rows, self.work, self.contracts, self.contract_bytes)
    }
}

pub(crate) fn validate_collected_rows(
    rows: &s::SourceRows,
    work: &mut SourceWork,
    extra: usize,
) -> Result<(), &'static str> {
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
