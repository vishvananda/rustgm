//! Complete source inspection retains a sealed joint row/contract owner.
use super::{Declarations, Report, Source, SourceInspection, s};
#[cfg(test)]
use super::{g, retained};
use crate::{
    resolve::package::Error,
    sema::crates::authored_export::{CollectedRows, SourceWork, validate_collected_rows},
};
#[cfg(test)]
#[path = "../../../tests/invariants/numbered_headers.rs"]
mod tests;

pub(crate) struct TraitInspection {
    collected: CollectedRows,
}
impl TraitInspection {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "genuine private collected-artifact admission precedes full retained role conformance and original/instance proof"
        )
    )]
    pub(crate) fn into_artifact(
        self,
        base: crate::support::crates::Interface,
        work: &mut crate::resolve::crates::ReservationWork,
    ) -> Result<crate::sema::crates::authored_export::CollectedArtifact, Error> {
        self.collected.into_artifact(base, work)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private named provided consumers await complete checked headers/body demand and supported public artifact policy"
        )
    )]
    pub(crate) fn into_named_artifact(
        self,
        base: crate::support::crates::Interface,
        alias: crate::support::crates::Dependency,
        work: &mut crate::resolve::crates::ReservationWork,
    ) -> Result<crate::sema::crates::authored_export::CollectedArtifact, Error> {
        self.collected.into_named_artifact(base, alias, work)
    }
    #[cfg(test)]
    pub(crate) fn into_parts(self) -> (SourceInspection, s::HeaderAttributeColumns, usize) {
        let (rows, work, contracts, bytes) = self.collected.into_unpaid();
        (SourceInspection { rows, work }, contracts, bytes)
    }
}
impl Source {
    pub(in crate::lower) fn complete_trait_numbered(
        self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<TraitInspection, Error> {
        let (collected, _) = self.complete_collected(report, facts, false)?;
        Ok(TraitInspection { collected })
    }
    pub(super) fn complete_provided_contract_payload(
        self,
        report: &Report,
        facts: &Declarations,
        base: crate::support::crates::Interface,
    ) -> Result<crate::support::crates::Interface, Error> {
        if report
            .package
            .source_core()
            .is_none_or(|source| &source.provider != base.identity())
        {
            return Err(Error::unlocated(
                "provided publication changed its actual provider",
            ));
        }
        let (collected, modifiers) = self.complete_collected(report, facts, true)?;
        collected.into_provided_interface(
            base,
            modifiers.ok_or_else(|| Error::unlocated("provided modifier completion is missing"))?,
        )
    }
    fn complete_collected(
        self,
        report: &Report,
        facts: &Declarations,
        contracts: bool,
    ) -> Result<(CollectedRows, Option<s::HeaderModifierColumns>), Error> {
        let Source {
            mut stage,
            openings,
            numbering,
            collector,
        } = self;
        let collector = collector.ok_or_else(|| {
            Error::unlocated("source header completion lacks its actual producer origin")
        })?;
        collector
            .require_root(report, facts, &mut stage)
            .map_err(Error::unlocated)?;
        let check = |stage: &mut crate::sema::crates::authored_export::Stage, declaration| {
            collector
                .require_header(report, declaration, stage)
                .map_err(Error::unlocated)
        };
        let (collected, modifiers) = if contracts {
            let (collected, modifiers) =
                stage.collect_contract_provider_headers(report, facts, check)?;
            (collected, Some(modifiers))
        } else {
            (stage.collect_provider_headers(report, facts, check)?, None)
        };
        drop(collector);
        // Preserve the original release boundary before index validation.
        drop(openings);
        let collected = if let Some(modifiers) = &modifiers {
            collected.finish_with_extra(numbering, modifiers.retained_bytes())
        } else {
            collected.finish(numbering)
        }
        .map_err(Error::unlocated)?;
        Ok((collected, modifiers))
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
    validate_collected_rows(rows, work, extra)
}
