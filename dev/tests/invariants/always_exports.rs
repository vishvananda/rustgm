//! New callable profiles must retain the actual Always declaration boundary.
use super::*;

#[test]
fn module_callables_preserve_always_contract_fence() -> TestResult {
    let missing = input("exported-missing-const-policy")?;
    let error = report(&missing, "always_missing_policy")
        .and_then(|report| {
            pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(Into::into)
        })
        .err()
        .ok_or("public Always declaration lost its required const API policy")?;
    assert!(error.to_string().contains("missing const"), "{error}");
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
    let definition = function(&report, "always_exported")?;
    let advisory = facts
        .reject_function_advisory_transport(&report, definition)
        .err()
        .ok_or("actual Always declaration lost its authenticated transport fence")?;
    assert!(
        advisory
            .to_string()
            .contains("always-comptime source contracts require a wider transport profile"),
        "{advisory}"
    );
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
            .contains("source effective const eligibility requires its actual marker transport"),
        "{error}"
    );
    Ok(())
}
