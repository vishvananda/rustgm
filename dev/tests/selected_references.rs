//! A retained whole-call relation permits shared references through actual F.
use crate::provider_traits::{TestResult, lower};
use std::path::Path;

const POSITIVE: &[(&str, &str)] = &[
    ("reference-binder", "7 1\n"),
    ("reference-local-argument", "7 9 1\n"),
    ("reference-nested-caller", "7 9 1\n"),
    ("reference-two-arguments", "7 9 1\n"),
    ("reference-slice-complete-provider", "7 1\n"),
    ("reference-early-named", "7 1\n"),
    ("caller-scaling-8", "7 1\n"),
    ("caller-scaling-32", "7 1\n"),
    ("caller-scaling-128", "7 1\n"),
    ("reference-nested-named", "7 1\n"),
    ("reference-anonymous-runtime-opposite", "7 1\n"),
];
fn fixture(name: &str, extension: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../pa31/tests/spec/const-eval-select-ctfe/{name}.{extension}"
        )),
    )?)
}
#[test]
fn selected_reference_callbacks_preserve_caller_allocations_and_whole_signatures() -> TestResult {
    for &(name, _) in POSITIVE {
        lower(&fixture(name, "rs")?).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
#[test]
fn selected_reference_callbacks_keep_lifetime_and_const_eligibility_obligations() -> TestResult {
    for (name, diagnostic) in [
        ("reference-return-local-reject", "does not outlive"),
        ("reference-crossed-result-reject", "does not outlive"),
        ("reference-static-result-reject", "does not outlive"),
        (
            "reference-nonconst-first-reject",
            "trait obligation requires an unambiguous matching assumption or implementation",
        ),
        (
            "reference-function-pointer-first-reject",
            "trait obligation requires an unambiguous matching assumption or implementation",
        ),
    ] {
        let error = lower(&fixture(name, "rs")?)
            .err()
            .ok_or_else(|| format!("{name}: required rejection disappeared"))?
            .to_string();
        assert!(error.contains(diagnostic), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn anonymous_selector_expression_does_not_acquire_named_owner_permission() -> TestResult {
    // The separately declared scalar case isolates owner admission. The original
    // reference case retains an earlier Copy/read frontier and its own observer.
    let error = lower(&fixture("anonymous-empty-selector", "rs")?)
        .err()
        .ok_or("anonymous selector caller permission changed")?
        .to_string();
    assert!(
        error.contains(
            "source const_eval_select execution for this caller owner is not implemented"
        ),
        "{error}"
    );
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn selected_reference_native_and_runtime_opposites_match_declared_answers() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/reference-select-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for &(name, expected) in POSITIVE {
        let program = lower(&fixture(name, "rs")?).map_err(|error| format!("{name}: {error}"))?;
        crate::ffi_program::check(
            root,
            &folder,
            name,
            &program,
            &fixture(name, "c")?,
            expected,
        )?;
    }
    Ok(())
}
