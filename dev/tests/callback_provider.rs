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
    ("item-reuse", "7 2\n"),
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
fn source_selector_ctfe_ignores_placeholder_and_malformed_header_diagnoses_safely() -> TestResult {
    // Retain the original independent CTFE-positive source, including its
    // looping body-bearing selector. The reached intrinsic must enter F.
    lower(&source("ctfe-placeholder-must-not-run", "rs")?)?;
    // Pinned rustc ICEs for this malformed declaration; this is a compiler
    // robustness opponent, without claiming it as an invalid Rust contract.
    assert!(lower(&source("malformed-unused-signature", "rs")?).is_err());
    Ok(())
}

#[test]
fn original_staged_provider_uses_its_actual_declared_trait_policies() -> TestResult {
    // Pinned rustc accepts and executes this unchanged full-staged provider.
    // Completed policies are consumed as actual source facts; the historical
    // unsupported Tuple-header boundary has been implemented.
    lower(&source("literal-attributed-declaration", "rs")?)?;
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

fn callback_result_program(
    path: &str,
    identity: &str,
) -> TestResult<rustgm::lowir::program::Program> {
    let text = crate::resolve::fixture(&format!("pa31/tests/spec/{path}.rs"))?;
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = crate::resolve::source_provider_config(&text, identity, &config)?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    callback_export(&report, analysis)
}

fn callback_export(
    report: &rustgm::resolve::expansion::Report,
    rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    }: rustgm::pipeline::Analysis,
) -> TestResult<rustgm::lowir::program::Program> {
    let exports = facts
        .functions
        .iter()
        .filter(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "code"
        })
        .map(|f| f.definition)
        .collect::<Vec<_>>();
    assert_eq!(exports.len(), 1);
    Ok(rustgm::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &exports,
        },
        prepared,
    )?)
}

// The ordinary companion remains required in PA30's manifest. Its preexisting
// bootstrap conformance failure does not qualify a rejection or an artifact.
#[test]
fn source_projected_callback_results_prepare_actual_exported_calls() -> TestResult {
    callback_result_program(
        "callable-result-heads/native-order",
        "projected_callback_execution",
    )
    .map(|_| ())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn source_projected_callback_results_execute_once() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = root.join("obj/parallel/array-chunks-next/native");
    std::fs::create_dir_all(&out)?;
    crate::ffi_program::check(
        &root,
        &out,
        "pa31",
        &callback_result_program(
            "callable-result-heads/native-order",
            "projected_callback_execution",
        )?,
        &crate::resolve::fixture("pa30/tests/spec/callable-result-heads/observer.c")?,
        "callable-result 23 12\n",
    )
}

#[test]
fn source_wrapped_callback_results_prepare_actual_exported_calls() -> TestResult {
    callback_result_program(
        "by-ref-fold/native-native-order",
        "wrapped_callback_execution",
    )
    .map(|_| ())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn source_wrapped_callback_results_execute_once() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = root.join("obj/parallel/by-ref-fold/native");
    std::fs::create_dir_all(&out)?;
    crate::ffi_program::check(
        &root,
        &out,
        "pa31",
        &callback_result_program(
            "by-ref-fold/native-native-order",
            "wrapped_callback_execution",
        )?,
        &crate::resolve::fixture("pa30/tests/spec/by-ref-fold/observer.c")?,
        "by-ref-fold 23 1234\n",
    )
}

#[test]
fn source_try_captures_prepare_actual_protocol_and_owned_environments() -> TestResult {
    callback_result_program(
        "try-captures/native-captured-protocol",
        "try_capture_execution",
    )
    .map(|_| ())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn source_try_captures_execute_actual_value_output_and_residual_calls() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = root.join("obj/parallel/filter-try-capture/native");
    std::fs::create_dir_all(&out)?;
    crate::ffi_program::check(
        &root,
        &out,
        "pa31",
        &callback_result_program(
            "try-captures/native-captured-protocol",
            "try_capture_execution",
        )?,
        &crate::resolve::fixture("pa30/tests/spec/try-captures/observer.c")?,
        "try-capture 7 43 23 123\n",
    )
}

fn match_capture_program(owner: &str) -> TestResult<rustgm::lowir::program::Program> {
    if owner == "pa31" {
        return callback_result_program("match-captures/native-order", "match_capture_execution");
    }
    let text = crate::resolve::fixture("pa24/tests/spec/match-captures/native-order.rs")?;
    let report = crate::resolve::resolve(&text, true)?;
    let facts = rustgm::sema::declarations::analyze(&report)?;
    let analysis = rustgm::pipeline::analyze(&report, facts)?;
    callback_export(&report, analysis)
}

#[test]
fn computed_match_captures_prepare_actual_both_variant_calls() -> TestResult {
    for owner in ["pa24", "pa31"] {
        match_capture_program(owner)?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn computed_match_captures_execute_callbacks_once_in_branch_order() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = root.join("obj/parallel/filter-map-captures/native");
    std::fs::create_dir_all(&out)?;
    for owner in ["pa24", "pa31"] {
        crate::ffi_program::check(
            &root,
            &out,
            owner,
            &match_capture_program(owner)?,
            &crate::resolve::fixture(&format!("{owner}/tests/spec/match-captures/observer.c"))?,
            "match-capture 7 9 12314\n",
        )?;
    }
    Ok(())
}
