//! Actual source callback selection preserves independent runtime and ownership contracts.
use crate::provider_traits::{TestResult, lower};
use std::path::Path;

const RUNTIME: &[(&str, &str)] = &[
    ("item-empty", "7\n"),
    ("pointer-two-targets", "9 13\n"),
    ("heterogeneous-tuple", "37\n"),
    ("generic-transfer", "37\n"),
    ("unused-reference-binder", "7\n"),
    ("reference-result", "37 1\n"),
    ("operand-effects-once", "7 1234\n"),
    ("unselected-const-loop", "7\n"),
    ("noncopy-aggregate", "37\n"),
    ("mapped-runtime-and-unused-const", "7\n"),
    ("ordinary-static-target", "7\n"),
    ("owned-drop-once", "43 1 43 1\n"),
    ("scale-mapped-8", "7 8\n"),
    ("scale-mapped-32", "7 32\n"),
    ("scale-mapped-128", "7 128\n"),
    ("scale-pointer-128", "9 128 13 128\n"),
];
fn source(name: &str, extension: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../pa31/tests/spec/const-eval-select-runtime/{name}.{extension}"
        )),
    )?)
}

#[test]
fn source_runtime_selector_uses_evaluated_runtime_callback_and_owned_tuple() -> TestResult {
    let mut failures = Vec::new();
    for &(name, _) in RUNTIME {
        match lower(&source(name, "rs")?) {
            Ok(_) => println!("{name}: checked lowering accepted"),
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn source_selector_keeps_callable_const_tuple_and_reference_obligations() -> TestResult {
    for (name, diagnostic) in [
        (
            "nonconst-first",
            "trait obligation requires an unambiguous matching assumption or implementation",
        ),
        (
            "first-function-pointer",
            "trait obligation requires an unambiguous matching assumption or implementation",
        ),
        (
            "non-tuple-arg",
            "associated type requires an unambiguous validated implementation",
        ),
        (
            "wrong-runtime-arity",
            "associated type requires an unambiguous validated implementation",
        ),
        ("wrong-runtime-result", "incompatible types"),
        (
            "reference-escape",
            "required lifetime does not outlive its target",
        ),
    ] {
        let error = lower(&source(name, "rs")?)
            .err()
            .ok_or_else(|| format!("{name}: invalid source was accepted"))?;
        assert!(error.to_string().contains(diagnostic), "{name}: {error}");
        println!("{name}: {error}");
    }
    Ok(())
}

#[test]
fn source_selector_ctfe_remains_explicit_and_malformed_header_diagnoses_safely() -> TestResult {
    // Rust accepts this CTFE program. Its callback remains required work, and
    // this temporary denial checks that no body-bearing placeholder can run.
    let error = lower(&source("ctfe-placeholder-must-not-run", "rs")?)
        .err()
        .ok_or("source selector placeholder executed in CTFE")?
        .to_string();
    assert!(
        error.contains("source const_eval_select constant callback execution is not implemented"),
        "{error}"
    );
    // Pinned rustc ICEs for this malformed declaration; this is a compiler
    // robustness opponent, without claiming it as an invalid Rust contract.
    assert!(lower(&source("malformed-unused-signature", "rs")?).is_err());
    Ok(())
}

#[test]
fn original_staged_provider_retains_its_explicit_unsupported_profile() -> TestResult {
    // Pinned rustc accepts and executes this unchanged full-staged provider.
    // Its API/const-stability profile remains required separately from the
    // independently qualified semantic provider used by the runtime cases.
    let error = lower(&source("literal-attributed-declaration", "rs")?)
        .err()
        .ok_or("full-staged provider support changed; review its retained boundary")?;
    let message = error.to_string();
    assert!(
        message.ends_with(
            "const, default-body and macro library stability contracts are not implemented"
        ),
        "{message}"
    );
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn source_runtime_selector_native_results_and_single_evaluation_match_rust() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/const-select-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for &(name, expected) in RUNTIME {
        let checked = || -> TestResult {
            let program = lower(&source(name, "rs")?).map_err(|e| format!("{name}: {e}"))?;
            crate::ffi_program::check(
                root,
                &folder,
                name,
                &program,
                &source(name, "c")?,
                expected,
            )?;
            assert!(std::fs::read(folder.join(format!("{name}-execute.stderr")))?.is_empty());
            Ok(())
        };
        if let Err(error) = checked() {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
