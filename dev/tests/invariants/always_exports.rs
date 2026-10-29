//! New callable profiles must retain the actual Always declaration boundary.
use super::*;

#[test]
fn module_callables_preserve_always_contract_fence() -> TestResult {
    let source = input("exported-full-provider")?;
    let report = report(&source, "always_export")?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual source provider")?
        .current
        .clone();
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let error = crate::lower::instances::build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        crate::lower::instances::SourcePublication::ModuleCallables,
    )
    .err()
    .ok_or("always-comptime declaration was published as ordinary const")?;
    assert!(
        error
            .to_string()
            .contains("always-comptime source contracts require a wider transport profile"),
        "{error}"
    );
    Ok(())
}
