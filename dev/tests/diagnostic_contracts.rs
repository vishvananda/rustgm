//! Declaration diagnostics consume real failed trait goals and persistent warning views.
use std::{fmt::Write as _, path::Path};
type TestResult = Result<(), Box<dyn std::error::Error>>;
struct Case {
    path: &'static str,
    errors: &'static [&'static str],
    stderr: &'static str,
}
const CASES: &[Case] = &[
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-internal-unused.rs",
        errors: &[],
        stderr: "",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-internal-selected.rs",
        errors: &["NEEDS u32", "label: LABEL", "note: NOTE"],
        stderr: "",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-internal-unselected.rs",
        errors: &[],
        stderr: "",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-stable-selected.rs",
        errors: &["STABLE u32"],
        stderr: "",
    },
    Case {
        path: "pa10/tests/spec/trait-diagnostics/diagnostic-internal-wrong-owner.rs",
        errors: &[],
        stderr: "warning[unused_attributes]: file 0: 81: rustc_on_unimplemented requires a trait owner\n",
    },
    Case {
        path: "pa10/tests/spec/trait-diagnostics/diagnostic-stable-wrong-owner.rs",
        errors: &[],
        stderr: "warning[misplaced_diagnostic_attributes]: file 0: 56: diagnostic::on_unimplemented requires a trait owner\n",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-no-gate.rs",
        errors: &["rustc_on_unimplemented requires rustc_attrs"],
        stderr: "",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-empty.rs",
        errors: &[],
        stderr: "warning[malformed_diagnostic_attributes]: file 0: 81: malformed on_unimplemented attribute\n",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-bare.rs",
        errors: &[],
        stderr: "warning[malformed_diagnostic_attributes]: file 0: 81: malformed on_unimplemented attribute\n",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-unknown-option.rs",
        errors: &[],
        stderr: "warning[malformed_diagnostic_attributes]: file 0: 81: malformed on_unimplemented attribute\n",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-nonstring.rs",
        errors: &[],
        stderr: "warning[malformed_diagnostic_attributes]: file 0: 81: malformed on_unimplemented attribute\n",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-unknown-format.rs",
        errors: &[],
        stderr: "warning[malformed_diagnostic_format_literals]: file 0: 81: on_unimplemented template names an unknown trait parameter\n",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-duplicate.rs",
        errors: &["multiple rustc_on_unimplemented contracts are not supported"],
        stderr: "",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-duplicate-option.rs",
        errors: &[],
        stderr: "warning[malformed_diagnostic_attributes]: file 0: 81: duplicate on_unimplemented option\n",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-condition-selected.rs",
        errors: &["NEEDS u32", "note: EMPTY"],
        stderr: "",
    },
    Case {
        path: "pa19/tests/spec/trait-diagnostics/diagnostic-condition-not-selected.rs",
        errors: &["NEEDS u32"],
        stderr: "",
    },
];
#[test]
fn actual_trait_contracts_select_messages_and_keep_default_warnings() -> TestResult {
    for case in CASES {
        let source = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .ok_or("missing repository root")?
                .join(case.path),
        )?;
        let result = crate::attribute_contracts::analyze(&source);
        if !case.errors.is_empty() {
            let error = result
                .err()
                .ok_or_else(|| format!("{} accepted", case.path))?
                .to_string();
            for fragment in case.errors {
                assert!(error.contains(fragment), "{}: {error}", case.path);
            }
            if case.path.ends_with("condition-not-selected.rs") {
                assert!(!error.contains("note: EMPTY"), "{error}");
            }
            continue;
        }
        let (_, facts) = result.map_err(|error| format!("{}: {error}", case.path))?;
        for _ in 0..2 {
            let mut text = String::new();
            for warning in facts.warnings() {
                writeln!(
                    text,
                    "warning[{}]: {}",
                    warning.code.name(),
                    warning.diagnostic
                )?;
            }
            assert_eq!(text, case.stderr, "{}", case.path);
        }
    }
    Ok(())
}
#[test]
fn mixed_duplicate_internal_attribute_error_keeps_its_warning_contract() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repository root")?;
    let source = std::fs::read_to_string(
        root.join("pa19/tests/spec/trait-diagnostics/diagnostic-duplicate.rs"),
    )?;
    let error = crate::attribute_contracts::analyze(&source)
        .err()
        .ok_or("duplicate internal attribute accepted")?;
    assert!(
        error
            .to_string()
            .contains("multiple rustc_on_unimplemented contracts are not supported"),
        "{error}"
    );
    // Warning transport is typed; ordinary Error Display remains the original failure.
    assert_eq!(
        error
            .warnings()
            .iter()
            .map(|warning| warning.code.name())
            .collect::<Vec<_>>(),
        ["malformed_diagnostic_attributes"]
    );
    assert!(
        error.warnings()[0]
            .diagnostic
            .message
            .contains("duplicate on_unimplemented option")
    );
    Ok(())
}

#[test]
fn mixed_attribute_error_renders_warnings_once_at_each_driver_boundary() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repository root")?;
    let fixture = root.join("pa19/tests/spec/trait-diagnostics/diagnostic-duplicate.rs");
    let source = std::fs::read_to_string(&fixture)?;
    let name = "#[rustc_on_unimplemented";
    let first = source
        .find(name)
        .ok_or("missing first diagnostic attribute")?;
    let second = first
        + 1
        + source[first + 1..]
            .find(name)
            .ok_or("missing duplicate diagnostic attribute")?;
    let expected = format!(
        "warning[malformed_diagnostic_attributes]: file 0: {second}: duplicate on_unimplemented option\nERROR: file 0: {second}: multiple rustc_on_unimplemented contracts are not supported\n"
    );
    for mode in [
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
