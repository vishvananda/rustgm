//! Real reports and stores are compiled once, then moved into unpaid sessions.
use crate::{resolve::expansion::Report, sema::model::Declarations, support::crates};
use std::sync::Arc;
pub(crate) type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) fn actual(provided: bool) -> Result<(Report, Declarations, Arc<crates::Dependencies>)> {
    let path = if provided {
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs"
    } else {
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs"
    };
    let source = crate::test_source::fixture(path)?;
    compile(&source, provided)
}
pub(crate) fn extended(suffix: &str) -> Result<(Report, Declarations, Arc<crates::Dependencies>)> {
    extended_with(suffix, |_, _, _, _| Ok(()))
}
pub(crate) fn extended_with(
    suffix: &str,
    observe: impl FnOnce(
        &Report,
        &Declarations,
        &crates::generics::source::SourceRows,
        &crates::Interface,
    ) -> Result,
) -> Result<(Report, Declarations, Arc<crates::Dependencies>)> {
    let mut source = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    source.push_str(suffix);
    compile_with(&source, true, observe)
}
fn compile(
    source: &str,
    provided: bool,
) -> Result<(Report, Declarations, Arc<crates::Dependencies>)> {
    compile_with(source, provided, |_, _, _, _| Ok(()))
}
fn compile_with(
    source: &str,
    provided: bool,
    observe: impl FnOnce(
        &Report,
        &Declarations,
        &crates::generics::source::SourceRows,
        &crates::Interface,
    ) -> Result,
) -> Result<(Report, Declarations, Arc<crates::Dependencies>)> {
    let report = crate::test_source::source_provider(source, "unpaid_reservations")?;
    let owner = report
        .package
        .source_core()
        .ok_or("actual source policy")?
        .provider
        .clone();
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let mut rows = crate::lower::instances::inspect_source_transport_with_numbering(
        &report,
        &mut facts,
        &owner,
        analysis.prepared,
    )?
    .rows;
    let interface = crates::Interface::new(owner, Vec::new(), Vec::new())?;
    observe(&report, &facts, &rows, &interface)?;
    let payload = if provided {
        crates::generics::Payload::from_source_provided(
            interface.identity(),
            rows,
            interface.base_retained_bytes(),
            &mut crates::generics::Work::default(),
        )?
    } else {
        rows.body_units.clear();
        crates::generics::Payload::from_source(
            interface.identity(),
            rows,
            interface.base_retained_bytes(),
            &mut crates::generics::Work::default(),
        )?
    };
    let inventory =
        crates::Dependencies::new(vec![interface.with_generic_payload(payload)?], Vec::new())?;
    Ok((report, facts, Arc::new(inventory)))
}
