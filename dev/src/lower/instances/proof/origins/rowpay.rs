//! Structural row accounting borrows its actual report, never missing headers.
use crate::{
    body::copy_requirements::OriginalIndexBudget,
    resolve::{expansion::Report, unit::NodeKey},
    sema::model::Declarations,
};

pub(super) struct Payer<'report> {
    report: &'report Report,
    kind: Kind,
}
enum Kind {
    Source,
    Attributes,
    Legacy(OriginalIndexBudget),
}
impl<'report> Payer<'report> {
    pub(super) fn new(report: &'report Report, facts: &Declarations) -> Result<Self, &'static str> {
        authenticate(report, facts)?;
        let kind = if report.package.source_core().is_some() {
            Kind::Source
        } else if report.lint_producer().is_some() {
            Kind::Attributes
        } else {
            Kind::Legacy(OriginalIndexBudget::new(1_000_000, 64 * 1024 * 1024))
        };
        Ok(Self { report, kind })
    }
    pub(super) fn charge(
        &self,
        facts: &Declarations,
        work: usize,
        bytes: usize,
    ) -> Result<(), &'static str> {
        authenticate(self.report, facts)?;
        match &self.kind {
            Kind::Source => facts.charge_caller_inputs(work, bytes),
            Kind::Attributes => {
                let module = self
                    .report
                    .package
                    .modules
                    .first()
                    .ok_or("row report has no root")?;
                self.report
                    .lint_producer()
                    .ok_or("original rows lost their actual report payer")?
                    .diagnostic_payload(
                        NodeKey {
                            file: module.file,
                            node: module.root,
                        },
                        work,
                        bytes,
                    )
                    .map_err(|e| e.message)
            }
            Kind::Legacy(budget) => budget.charge(work, bytes),
        }
    }
}
fn authenticate(report: &Report, facts: &Declarations) -> Result<(), &'static str> {
    if report.package.modules.is_empty() {
        return Err("original row report has no actual root");
    }
    facts
        .require_lint_context(report)
        .map_err(|_| "original rows changed their actual lint context")?;
    if let Some(catalog) = facts.source_bindings.as_deref() {
        catalog.require_provider_report(report)?;
    } else if report.package.source_core().is_some() || report.package.source_client().is_some() {
        return Err("original rows require their actual source catalog");
    }
    Ok(())
}
