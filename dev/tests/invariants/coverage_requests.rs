//! Coverage checks real syntax and literal authority, not code instrumentation.
use crate::{pipeline, test_source};
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[test]
fn coverage_forms_targets_duplicates_and_unrequested_local_tokens_remain_checked() -> TestResult {
    let foundation =
        test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    for (name, declaration, accepted) in [
        ("function", "#[coverage(off)]fn selected(){}", true),
        ("module", "#[coverage(on)]mod selected{}", true),
        ("nominal", "#[coverage(off)]struct Selected;", false),
        ("word", "#[coverage]fn selected(){}", false),
        ("number", "#[coverage(3)]fn selected(){}", false),
        ("extra", "#[coverage(off,on)]fn selected(){}", false),
        (
            "duplicate",
            "#[coverage(off)]#[coverage(off)]fn selected(){}",
            false,
        ),
    ] {
        let source = String::from("#![feature(coverage_attribute)]") + &foundation + declaration;
        let result = test_source::source_provider(&source, name).and_then(|report| {
            pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    let public = String::from("#![feature(coverage_attribute)]")
        + &foundation
        + "#[coverage(off)]pub fn covered(x:u8)->u8{x}";
    let report = test_source::source_provider(&public, "coverage_publication")?;
    let mut analysis = pipeline::analyze_source(&report)?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual source provider")?
        .provider
        .clone();
    let (_, interface) = crate::lower::instances::build_library_prepared(
        &report,
        &mut analysis.facts,
        identity,
        analysis.prepared,
    )?;
    assert_eq!(
        interface
            .source_descriptor()
            .ok_or("actual source descriptor")?
            .profile(),
        crate::support::crates::SourceProfile::Source2024Functions
    );
    let scalar = public.replace("pub fn covered(x:u8)->u8{x}", "pub fn covered()->u8{7u8}");
    let report = test_source::source_provider(&scalar, "coverage_scalar_boundary")?;
    let mut analysis = pipeline::analyze_source(&report)?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual scalar provider")?
        .provider
        .clone();
    let error = crate::lower::instances::build_library_prepared(
        &report,
        &mut analysis.facts,
        identity,
        analysis.prepared,
    )
    .err()
    .ok_or("legacy scalar pool boundary silently widened")?;
    assert!(
        error
            .to_string()
            .contains("source scalar pool requires a wider transport profile"),
        "{error}"
    );
    let report = test_source::source_provider(
        &(foundation + "#[coverage(off)]fn unrequested(){}"),
        "coverage_unrequested",
    )?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("unrequested physical coverage accepted")?;
    assert!(error.to_string().contains("coverage_attribute"), "{error}");
    Ok(())
}
