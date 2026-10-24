//! Retained attribute contracts use independently declared source and warning outcomes.
use rustgm::{expand::config::Config, resolve::{self, expansion::Report, load::{InputFile, Sources}},
    sema::model::Declarations, support::source::SourceFile};
use std::path::Path;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> { Ok(None) }
}
struct Case {
    name: &'static str,
    path: &'static str,
    codes: &'static [&'static str],
    error: Option<&'static str>,
    stderr: &'static str,
}
const CASES: &[Case] = &[
    Case { name: "pa10-function-discard-default", path: "pa10/tests/spec/must-use/function-discard-default.rs", codes: &["unused_must_use"], error: None, stderr: "warning[unused_must_use]: file 0: 110: unused must_use result\n" },
    Case { name: "pa10-function-discard-deny", path: "pa10/tests/spec/must-use/function-discard-deny.rs", codes: &[], error: Some("unused must_use result"), stderr: "" },
    Case { name: "pa10-function-discard-warn", path: "pa10/tests/spec/must-use/function-discard-warn.rs", codes: &["unused_must_use"], error: None, stderr: "warning[unused_must_use]: file 0: 152: unused must_use result: retain result\n" },
    Case { name: "pa10-function-discard-allow", path: "pa10/tests/spec/must-use/function-discard-allow.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa10-function-explicit-let", path: "pa10/tests/spec/must-use/function-explicit-let.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa10-function-pointer-discard", path: "pa10/tests/spec/must-use/function-pointer-discard.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa10-direct-item-variable-discard", path: "pa10/tests/spec/must-use/direct-item-variable-discard.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa10-outer-deny-inner-allow", path: "pa10/tests/spec/must-use/outer-deny-inner-allow.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa10-outer-forbid-inner-allow", path: "pa10/tests/spec/must-use/outer-forbid-inner-allow.rs", codes: &[], error: Some("forbidden unused_must_use lint cannot be weakened"), stderr: "" },
    Case { name: "pa10-outer-forbid-inner-warn", path: "pa10/tests/spec/must-use/outer-forbid-inner-warn.rs", codes: &[], error: Some("forbidden unused_must_use lint cannot be weakened"), stderr: "" },
    Case { name: "pa10-duplicate-default-warn", path: "pa10/tests/spec/must-use/duplicate-default-warn.rs", codes: &["unused_attributes"], error: None, stderr: "warning[unused_attributes]: file 0: 32: duplicate must_use attribute\n" },
    Case { name: "pa10-duplicate-deny", path: "pa10/tests/spec/must-use/duplicate-deny.rs", codes: &[], error: Some("duplicate must_use attribute"), stderr: "" },
    Case { name: "pa10-malformed-list", path: "pa10/tests/spec/must-use/malformed-list.rs", codes: &[], error: Some("must_use attribute requires no arguments or one string literal message"), stderr: "" },
    Case { name: "pa10-nonstring-message", path: "pa10/tests/spec/must-use/nonstring-message.rs", codes: &[], error: Some("must_use message requires a string literal"), stderr: "" },
    Case { name: "pa10-unsafe-wrapper", path: "pa10/tests/spec/must-use/unsafe-wrapper.rs", codes: &[], error: Some("must_use attribute cannot use an unsafe wrapper"), stderr: "" },
    Case { name: "pa12-struct-discard-deny", path: "pa12/tests/spec/must-use/struct-discard-deny.rs", codes: &[], error: Some("unused must_use result"), stderr: "" },
    Case { name: "pa12-struct-explicit-let", path: "pa12/tests/spec/must-use/struct-explicit-let.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa12-enum-discard-deny", path: "pa12/tests/spec/must-use/enum-discard-deny.rs", codes: &[], error: Some("unused must_use result"), stderr: "" },
    Case { name: "pa12-enum-explicit-let", path: "pa12/tests/spec/must-use/enum-explicit-let.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa12-nominal-tuple-discard", path: "pa12/tests/spec/must-use/nominal-tuple-discard.rs", codes: &[], error: Some("unused must_use result"), stderr: "" },
    Case { name: "pa12-nominal-empty-array-discard", path: "pa12/tests/spec/must-use/nominal-empty-array-discard.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa19-trait-concrete-discard", path: "pa19/tests/spec/must-use/trait-concrete-discard.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa19-trait-method-discard", path: "pa19/tests/spec/must-use/trait-method-discard.rs", codes: &[], error: Some("unused must_use result"), stderr: "" },
    Case { name: "pa19-inherent-method-discard", path: "pa19/tests/spec/must-use/inherent-method-discard.rs", codes: &[], error: Some("unused must_use result"), stderr: "" },
    Case { name: "pa21-trait-opaque-discard", path: "pa21/tests/spec/must-use/trait-opaque-discard.rs", codes: &[], error: Some("unused must_use result"), stderr: "" },
    Case { name: "pa21-trait-opaque-explicit-let", path: "pa21/tests/spec/must-use/trait-opaque-explicit-let.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa24-closure-expression-warn", path: "pa24/tests/spec/must-use/closure-expression-warn.rs", codes: &["unused_must_use"], error: None, stderr: "warning[unused_must_use]: file 0: 49: unused closure that must be used\n" },
    Case { name: "pa24-closure-variable-warn", path: "pa24/tests/spec/must-use/closure-variable-warn.rs", codes: &["unused_must_use"], error: None, stderr: "warning[unused_must_use]: file 0: 66: unused closure that must be used\n" },
    Case { name: "pa10-inline-function-hint", path: "pa10/tests/spec/inline-attributes/function-hint.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa10-inline-function-always", path: "pa10/tests/spec/inline-attributes/function-always.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa10-inline-function-never", path: "pa10/tests/spec/inline-attributes/function-never.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa10-inline-duplicate-default", path: "pa10/tests/spec/inline-attributes/duplicate-default.rs", codes: &["unused_attributes"], error: None, stderr: "warning[unused_attributes]: file 0: 47: duplicate inline attribute\n" },
    Case { name: "pa10-inline-duplicate-deny", path: "pa10/tests/spec/inline-attributes/duplicate-deny.rs", codes: &[], error: Some("duplicate inline attribute"), stderr: "" },
    Case { name: "pa10-inline-empty-list", path: "pa10/tests/spec/inline-attributes/empty-list.rs", codes: &[], error: Some("inline attribute requires exactly always or never"), stderr: "" },
    Case { name: "pa10-inline-unsafe-wrapper", path: "pa10/tests/spec/inline-attributes/unsafe-wrapper.rs", codes: &[], error: Some("inline attribute cannot use an unsafe wrapper"), stderr: "" },
    Case { name: "pa19-inline-trait-required", path: "pa19/tests/spec/inline-attributes/trait-required.rs", codes: &["unused_attributes"], error: None, stderr: "warning[unused_attributes]: file 0: 46: inline attribute on a function without a body has no effect\n" },
    Case { name: "pa19-inline-trait-provided", path: "pa19/tests/spec/inline-attributes/trait-provided.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa10-bootstrap-import-alias", path: "pa10/tests/spec/test-removal/bootstrap-import-alias.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa10-bootstrap-qualified-reexport", path: "pa10/tests/spec/test-removal/bootstrap-qualified-reexport.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa10-bootstrap-forward-alias-chain", path: "pa10/tests/spec/test-removal/bootstrap-forward-alias-chain.rs", codes: &[], error: None, stderr: "" },
    Case { name: "pa10-removed-name-does-not-reserve-binding", path: "pa10/tests/spec/test-removal/removed-name-does-not-reserve-binding.rs", codes: &[], error: None, stderr: "" },
];
fn source(case: &Case) -> std::io::Result<String> {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(case.path))
}
fn analyze(source: &str) -> TestResult<(Report, Declarations)> {
    let report = resolve::expansion::analyze(InputFile {
        identity: "attributes.rs".into(), path: "attributes.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    }, &mut NoFiles, &Config::default()).and_then(resolve::analyze_expanded)?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    Ok((report, analysis.facts))
}
#[test]
fn actual_owner_results_and_lexical_lint_controls_keep_persistent_diagnostics() -> TestResult {
    for case in CASES {
        let input = source(case)?;
        let result = analyze(&input);
        if let Some(expected) = case.error {
            let error = result.err().ok_or_else(|| format!("{} accepted", case.name))?;
            assert!(error.to_string().contains(expected), "{}: {error}", case.name);
            continue;
        }
        let (report, facts) = result.map_err(|error| format!("{}: {error}", case.name))?;
        let codes: Vec<_> = facts.warnings().map(|warning| warning.code.name()).collect();
        assert_eq!(codes, case.codes, "{}", case.name);
        let diagnostic = || facts.warnings().map(|warning| format!("warning[{}]: {}\n", warning.code.name(), warning.diagnostic)).collect::<String>();
        assert_eq!(diagnostic(), case.stderr, "{}", case.name);
        assert_eq!(diagnostic(), case.stderr, "{} repeated borrowed view", case.name);
        assert_eq!(facts.must_use_work().warnings, codes.len(), "{}", case.name);
        // A removed Test body with missing names never creates an invocation.
        if case.path.contains("/test-removal/") { assert!(report.invocations.is_empty(), "{}", case.name); }
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host linker"]
fn default_warning_and_explicit_discard_programs_keep_native_behavior() -> TestResult {
    let mut inputs = Vec::new();
    for case in CASES.iter().filter(|case| case.error.is_none()) { inputs.push((case, source(case)?)); }
    let cases: Vec<_> = inputs.iter().map(|(case, source)| (case.name, source.as_str(), false, case.stderr.as_bytes())).collect();
    crate::native_references::check_literal_cases_with_warnings("parallel/pa31/attribute-contracts-native", &cases)
}
