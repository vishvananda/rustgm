//! Constant selection enters actual F; ordinary execution retains actual G.
use crate::provider_traits::{TestResult, lower};
use std::path::Path;
const POSITIVE: &[(&str, &str)] = &[
    ("owned-tuple", "37 9\n"),
    ("mapped-through-return", "7 1\n"),
    ("operand-order", "7 3 1 3\n"),
    ("repeated-empty", "7 1\n"),
    ("unreached-const-loop", "7\n"),
    ("unselected-runtime-loop", "7\n"),
    ("null-raw-read-runtime-opposite", "9\n"),
    ("same-item-instances", "7 1\n"),
    ("conditional-owner-two-selectors", "7 1\n"),
];
fn fixture(name: &str, extension: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../pa31/tests/spec/const-eval-select-ctfe/{name}.{extension}"
        )),
    )?)
}
#[test]
fn source_ctfe_selector_uses_owned_tuple_and_actual_callback_without_placeholder() -> TestResult {
    for &(name, _) in POSITIVE {
        lower(&fixture(name, "rs")?).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn source_ctfe_selector_keeps_reached_fault_and_argument_ownership() -> TestResult {
    for (name, diagnostic) in [
        (
            "selected-null-raw-read-fault",
            "constant pointer has no allocation provenance",
        ),
        (
            "owned-argument-reuse-reject",
            "local may be uninitialized at this read",
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
fn source_ctfe_selector_retains_valid_binder_and_conditional_payment_frontiers() -> TestResult {
    let error = lower(&fixture("reference-binder", "rs")?)
        .err()
        .ok_or("callback binder support changed; review its actual opened signature")?
        .to_string();
    assert!(
        error.contains("source const_eval_select callback binders are not implemented"),
        "{error}"
    );
    let error = lower(&fixture("mapped-without-static-proof", "rs")?)
        .err()
        .ok_or("unproved callback lifetime acquired a static execution identity")?
        .to_string();
    assert!(
        error.contains(
            "source const_eval_select lifetime-bearing callback entry is not implemented"
        ),
        "{error}"
    );
    for name in ["conditional-empty", "conditional-nonempty"] {
        let error = lower(&fixture(name, "rs")?)
            .err()
            .ok_or_else(|| {
                format!(
                    "{name}: conditional callable authority changed; review its reached payment"
                )
            })?
            .to_string();
        assert!(
            error.contains(
                "source const callable conditional obligations require a scoped continuation"
            ),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn source_ctfe_selector_native_and_runtime_opposites_match_declared_answers() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/ctfe-select-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for &(name, expected) in POSITIVE {
        let program = lower(&fixture(name, "rs")?).map_err(|e| format!("{name}: {e}"))?;
        crate::ffi_program::check(
            root,
            &folder,
            name,
            &program,
            &fixture(name, "c")?,
            expected,
        )?;
    }
    let original = root.join("pa31/tests/spec/const-eval-select-runtime");
    let source = std::fs::read_to_string(original.join("ctfe-placeholder-must-not-run.rs"))?;
    let client = std::fs::read_to_string(original.join("item-empty.c"))?;
    crate::ffi_program::check(
        root,
        &folder,
        "original-empty",
        &lower(&source)?,
        &client,
        "7\n",
    )?;
    Ok(())
}
