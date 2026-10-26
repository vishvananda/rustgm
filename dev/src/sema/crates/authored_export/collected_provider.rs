//! Actual source completion keeps rows and advisory contracts under one owner.
use super::{Declarations, DefId, OwnerKind, Report, SourceWork, Stage, g, s};
use crate::{
    resolve::{model::DefKind, package::Error, unit::NodeKey},
    support::{crates::CrateIdentity, retained},
};
#[path = "collected_inventory.rs"]
mod inventory;
pub(crate) use inventory::{BoundReceipt, CollectedArtifact, CollectedReceipt};

pub(crate) struct CollectedScope {
    definition: DefId,
    source: NodeKey,
    header: bool,
}
impl CollectedScope {
    pub(super) fn header_source(definition: DefId, source: NodeKey) -> Self {
        Self {
            definition,
            source,
            header: true,
        }
    }
    pub(crate) fn source(&self) -> NodeKey {
        self.source
    }
    pub(super) fn definition(&self) -> DefId {
        self.definition
    }
    pub(super) fn header(&self) -> bool {
        self.header
    }
}

/// Producer-local Stage IDs retire before this arena-independent data can move.
pub(crate) struct CollectedStage {
    stage: Stage,
    provider: CrateIdentity,
    contracts: s::HeaderAttributeColumns,
    contract_bytes: usize,
    names: s::NamespaceColumns,
    names_bytes: usize,
    live: usize,
}
/// No unchecked constructor or mutable row/contract access is exposed.
pub(crate) struct CollectedRows {
    rows: s::SourceRows,
    work: SourceWork,
    provider: CrateIdentity,
    contracts: s::HeaderAttributeColumns,
    contract_bytes: usize,
    names: s::NamespaceColumns,
    names_bytes: usize,
}
impl Stage {
    pub(crate) fn collected_scope(
        &self,
        report: &Report,
        facts: &Declarations,
        index: usize,
    ) -> Result<Option<CollectedScope>, Error> {
        if let Some((definition, source)) = self.selected_header(report, facts, index)? {
            return Ok(Some(CollectedScope {
                definition,
                source,
                header: true,
            }));
        }
        let definition = DefId(
            u32::try_from(index)
                .map_err(|_| Error::unlocated("collected scope identity overflow"))?,
        );
        if self.definitions.get(index).is_none_or(Option::is_none)
            || report
                .resolution
                .definitions
                .get(index)
                .is_none_or(|declaration| declaration.kind != DefKind::SelfType)
        {
            return Ok(None);
        }
        let owner = facts
            .associated
            .owner(definition)
            .and_then(|owner| facts.associated.get(owner))
            .ok_or_else(|| {
                Error::unlocated("collected Self formal lost its actual associated owner")
            })?;
        if owner.self_definition != definition {
            return Err(Error::unlocated(
                "collected Self formal changed its actual owner identity",
            ));
        }
        match owner.kind {
            OwnerKind::Trait { .. } => return Ok(None),
            OwnerKind::Impl { .. } => {}
            OwnerKind::ImportedImpl(_) => {
                return Err(Error::unlocated(
                    "collected source formal cannot use an imported owner",
                ));
            }
        }
        let owner = owner.source_owner().map_err(Error::unlocated)?;
        if owner.self_definition != definition
            || report.resolution.definitions[index].node != Some(owner.source)
        {
            return Err(Error::unlocated(
                "collected impl formal scope changed its actual source owner",
            ));
        }
        Ok(Some(CollectedScope {
            definition,
            source: owner.source,
            header: false,
        }))
    }
    pub(crate) fn collect_provider_headers(
        mut self,
        report: &Report,
        facts: &Declarations,
        mut check: impl FnMut(&mut Stage, crate::resolve::unit::NodeKey) -> Result<(), Error>,
    ) -> Result<CollectedStage, Error> {
        let before = self.work();
        let overhead = size_of::<CollectedRows>()
            .saturating_sub(size_of::<s::SourceRows>())
            .saturating_sub(size_of::<SourceWork>())
            .saturating_sub(size_of::<s::HeaderAttributeColumns>())
            .saturating_sub(size_of::<s::NamespaceColumns>());
        self.reserve(overhead.saturating_add(size_of::<s::NamespaceColumns>()))
            .map_err(Error::unlocated)?;
        let names = self.namespace_columns(report).map_err(Error::unlocated)?;
        let mut validation = g::Work {
            steps: self.work().steps,
            ..g::Work::default()
        };
        let names_bytes = names.storage_bytes(&mut validation);
        self.charge(validation.steps.saturating_sub(self.work().steps))
            .map_err(Error::unlocated)?;
        let names_bytes = names_bytes.map_err(Error::unlocated)?;
        let expected = self.selected_type_slots().map_err(Error::unlocated)?;
        let mut observed = 0usize;
        let contracts =
            self.collect_formal_columns(report, facts, |stage, definition, source| {
                check(stage, source)?;
                observed = observed.saturating_add(
                    stage.require_named_parameters(report, facts, definition, source)?,
                );
                Ok(())
            })?;
        if observed != expected {
            return Err(Error::unlocated(
                "collected formals omitted complete actual type-slot coverage",
            ));
        }
        let mut validation = g::Work {
            steps: self.work().steps,
            ..g::Work::default()
        };
        let start = validation.steps;
        let result = contracts.validate(
            &self.rows,
            before.reserved_bytes.saturating_add(names_bytes),
            &mut validation,
        );
        self.charge(validation.steps.saturating_sub(start))
            .map_err(Error::unlocated)?;
        let contract_bytes = result.map_err(Error::unlocated)?;
        let live = contract_bytes
            .saturating_add(names_bytes)
            .saturating_add(overhead);
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
            names,
            names_bytes,
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
            names: self.names,
            names_bytes: self.names_bytes,
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
