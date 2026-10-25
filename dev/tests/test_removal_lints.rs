//! Test removal uses the enclosing lint level before removed owner attributes.
use rustgm::expand::config::Config;
use std::path::Path;
type TestResult = Result<(), Box<dyn std::error::Error>>;
struct Case { path: &'static str, warnings: usize, error: Option<&'static str>, cfg_test: bool }
const CASES: &[Case] = &[
    Case { path: "pa31/tests/spec/test-removal-lints/crate-allow.rs", warnings: 0, error: None, cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/crate-warn.rs", warnings: 1, error: None, cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/crate-deny.rs", warnings: 0, error: Some("duplicated test attribute"), cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/crate-forbid.rs", warnings: 0, error: Some("duplicated test attribute"), cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/module-allow.rs", warnings: 0, error: None, cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/owner-allow.rs", warnings: 1, error: None, cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/module-deny.rs", warnings: 0, error: Some("duplicated test attribute"), cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/owner-deny.rs", warnings: 1, error: None, cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/forbid-inner-allow.rs", warnings: 0, error: Some("duplicated test attribute"), cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/deny-inner-allow.rs", warnings: 0, error: Some("duplicated test attribute"), cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/triple-default.rs", warnings: 1, error: None, cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/duplicate-argument-after-first.rs", warnings: 1, error: None, cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/duplicate-argument-first.rs", warnings: 1, error: Some("test attribute requires an outer word attribute"), cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/alias-twice.rs", warnings: 0, error: None, cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/alias-then-test.rs", warnings: 1, error: None, cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/test-then-alias.rs", warnings: 0, error: None, cfg_test: false },
    Case { path: "pa31/tests/spec/test-removal-lints/cfg-test-no-harness.rs", warnings: 1, error: None, cfg_test: true },
 ];
#[test]
fn removed_test_owners_inherit_policy_before_their_own_lint_attributes() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().ok_or("missing repository root")?;
    for case in CASES {
        let source = std::fs::read_to_string(root.join(case.path))?;
        let mut config = Config::default();
        if case.cfg_test { config.add("test")?; }
        let result = crate::attribute_contracts::analyze_with_config(&source, &config);
        if let Some(expected) = case.error {
            let error = result.err().ok_or_else(|| format!("{} accepted", case.path))?;
            assert!(error.to_string().contains(expected), "{}: {error}", case.path);
            continue;
        }
        let (report, facts) = result.map_err(|error| format!("{}: {error}", case.path))?;
        assert!(report.invocations.is_empty(), "{}", case.path);
        let warnings: Vec<_> = facts.warnings().collect();
        assert_eq!(warnings.len(), case.warnings, "{}", case.path);
        assert!(warnings.iter().all(|warning| warning.code.name() == "duplicate_macro_attributes"), "{}", case.path);
    }
    Ok(())
}
