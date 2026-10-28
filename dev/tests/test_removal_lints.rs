//! Test removal uses the enclosing lint level before removed owner attributes.
use rustgm::expand::config::Config;
use std::path::Path;
type TestResult = Result<(), Box<dyn std::error::Error>>;
struct Case {
    path: &'static str,
    warnings: usize,
    error: Option<&'static str>,
    cfg_test: bool,
}
const CASES: &[Case] = &[
    Case {
        path: "pa31/tests/spec/test-removal-lints/crate-allow.rs",
        warnings: 0,
        error: None,
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/crate-warn.rs",
        warnings: 1,
        error: None,
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/crate-deny.rs",
        warnings: 0,
        error: Some("duplicated test attribute"),
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/crate-forbid.rs",
        warnings: 0,
        error: Some("duplicated test attribute"),
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/module-allow.rs",
        warnings: 0,
        error: None,
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/owner-allow.rs",
        warnings: 1,
        error: None,
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/module-deny.rs",
        warnings: 0,
        error: Some("duplicated test attribute"),
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/owner-deny.rs",
        warnings: 1,
        error: None,
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/forbid-inner-allow.rs",
        warnings: 0,
        error: Some("duplicated test attribute"),
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/deny-inner-allow.rs",
        warnings: 0,
        error: Some("duplicated test attribute"),
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/triple-default.rs",
        warnings: 1,
        error: None,
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/duplicate-argument-after-first.rs",
        warnings: 1,
        error: None,
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/duplicate-argument-first.rs",
        warnings: 1,
        error: Some("test attribute requires an outer word attribute"),
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/alias-twice.rs",
        warnings: 0,
        error: None,
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/alias-then-test.rs",
        warnings: 1,
        error: None,
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/test-then-alias.rs",
        warnings: 0,
        error: None,
        cfg_test: false,
    },
    Case {
        path: "pa31/tests/spec/test-removal-lints/cfg-test-no-harness.rs",
        warnings: 1,
        error: None,
        cfg_test: true,
    },
];
#[test]
fn removed_test_owners_inherit_policy_before_their_own_lint_attributes() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repository root")?;
    for case in CASES {
        let source = std::fs::read_to_string(root.join(case.path))?;
        let mut config = Config::default();
        if case.cfg_test {
            config.add("test").map_err(|error| error.message)?;
        }
        let result = crate::attribute_contracts::analyze_with_config(&source, &config);
        if let Some(expected) = case.error {
            let error = result
                .err()
                .ok_or_else(|| format!("{} accepted", case.path))?;
            assert!(
                error.to_string().contains(expected),
                "{}: {error}",
                case.path
            );
            assert_eq!(error.warnings().len(), case.warnings, "{}", case.path);
            assert!(
                error
                    .warnings()
                    .iter()
                    .all(|warning| warning.code.name() == "duplicate_macro_attributes"),
                "{}",
                case.path
            );
            continue;
        }
        let (report, facts) = result.map_err(|error| format!("{}: {error}", case.path))?;
        assert!(report.invocations.is_empty(), "{}", case.path);
        let warnings: Vec<_> = facts.warnings().collect();
        assert_eq!(warnings.len(), case.warnings, "{}", case.path);
        assert!(
            warnings
                .iter()
                .all(|warning| warning.code.name() == "duplicate_macro_attributes"),
            "{}",
            case.path
        );
    }
    Ok(())
}

#[test]
fn removed_owner_completion_keeps_direct_and_parent_expectations_in_their_actual_scope()
-> TestResult {
    use std::fmt::Write;
    for (name, codes) in [
        ("removed-direct-before", &[][..]),
        ("removed-direct-after", &[][..]),
        ("removed-direct-empty", &[][..]),
        (
            "removed-parent-crate",
            &["unfulfilled_lint_expectations"][..],
        ),
        (
            "removed-parent-module",
            &["unfulfilled_lint_expectations"][..],
        ),
        ("removed-parent-duplicate", &[][..]),
        (
            "removed-owner-duplicate",
            &["duplicate_macro_attributes"][..],
        ),
        (
            "removed-sibling-unfulfilled",
            &["unfulfilled_lint_expectations"][..],
        ),
        ("removed-sibling-fulfilled", &[][..]),
        (
            "removed-parent-deprecated",
            &["unfulfilled_lint_expectations"][..],
        ),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa40/tests/spec/lint-expectations/{name}.rs"));
        let source = std::fs::read_to_string(path)?;
        let (report, facts) = crate::attribute_contracts::analyze(&source)?;
        assert!(
            !report.resolution.definitions.iter().any(|definition| report
                .package
                .names
                .get(definition.name)
                == "gone"),
            "{name}: invented removed declaration"
        );
        let mut expected = String::new();
        for code in codes {
            let (offset, message) = if *code == "duplicate_macro_attributes" {
                let first = source.find("#[test]").ok_or("missing first test")?;
                (
                    first
                        + 7
                        + source[first + 7..]
                            .find("#[test]")
                            .ok_or("missing duplicate test")?,
                    "duplicated test attribute",
                )
            } else {
                (
                    source
                        .find("#![expect")
                        .or_else(|| source.find("#[expect"))
                        .ok_or("missing expected source owner")?,
                    "this lint expectation is unfulfilled",
                )
            };
            writeln!(expected, "warning[{code}]: file 0: {offset}: {message}")?;
        }
        let mut actual = String::new();
        for warning in facts.warnings() {
            writeln!(
                actual,
                "warning[{}]: {}",
                warning.code.name(),
                warning.diagnostic
            )?;
        }
        assert_eq!(actual, expected, "{name}: exact code/count/location/order");
        let header = rustgm::sema::declarations::analyze(&report)?;
        assert_eq!(
            header.body_work.cfg_functions.get(),
            0,
            "{name}: header request built a body"
        );
        assert!(
            header
                .warnings()
                .all(|warning| warning.code.name() == "duplicate_macro_attributes"),
            "{name}: prematurely finalized an untouched parent/body expectation"
        );
    }
    Ok(())
}

#[test]
fn malformed_first_test_transfers_the_warning_through_the_expansion_driver_boundary() -> TestResult
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repository root")?;
    let fixture = root.join("pa31/tests/spec/test-removal-lints/duplicate-argument-first.rs");
    let source = std::fs::read_to_string(&fixture)?;
    let first = source
        .find("#[test(unread)]")
        .ok_or("actual malformed attribute")?;
    let duplicate = source.find("#[test]").ok_or("actual duplicate attribute")?;
    let expected = format!(
        "warning[duplicate_macro_attributes]: file 0: {duplicate}: duplicated test attribute\nERROR: file 0: {first}: test attribute requires an outer word attribute\n"
    );
    for mode in [
        "--emit-expanded-crate",
        "--emit-types",
        "--emit-semantics",
        "--emit-lowir",
        "--emit-constants",
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg(mode)
            .arg(&fixture)
            .output()?;
        assert_eq!(output.status.code(), Some(1), "{mode}: {:?}", output.stderr);
        assert!(output.stdout.is_empty(), "{mode}: {:?}", output.stdout);
        assert_eq!(output.stderr, expected.as_bytes(), "{mode}");
    }
    Ok(())
}
