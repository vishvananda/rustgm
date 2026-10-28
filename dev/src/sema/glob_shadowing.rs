//! Immutable completed resolver causes use the actual private site's lexical policy.
use crate::{
    resolve::{expansion::Report, model::Namespace, package::Error},
    sema::model::Declarations,
    support::diagnostics::WarningCode,
};
pub(super) fn collect(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(producer) = report.lint_producer() else {
        return Ok(());
    };
    for cause in producer.hidden_glob_events() {
        let (key, span, namespace) = cause.site(report, producer)?;
        let ordinal = match namespace {
            Namespace::Type => 0,
            Namespace::Value => 1,
            Namespace::Macro => 2,
            _ => {
                return Err(Error::at(
                    key.file,
                    span.start,
                    "hidden glob namespace is not importable",
                ));
            }
        };
        producer.diagnostic_read(key, 96)?;
        facts.must_use_diagnostic(
            report,
            key,
            span.start,
            ordinal,
            WarningCode::HiddenGlobReexports,
            format!(
                "private item shadows public glob reexport in the {} namespace",
                namespace.name()
            ),
        )?;
    }
    Ok(())
}
