//! Genuine private source-provider setup shared by the two namespace controls.
use crate::support::crates::generics::source::UnpublishedNamespace;
use std::sync::Arc;

pub(crate) fn namespace_fixture(
    name: &str,
) -> Result<Arc<UnpublishedNamespace>, Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture(&format!(
        "pa31/tests/spec/private-module-namespace/{name}.rs"
    ))?;
    let report = crate::test_source::source_provider(&source, "module_namespace")?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    Ok(super::inspect_source_namespace(
        &report, &mut facts, &identity, prepared,
    )?)
}
