//! Actual source unsizing roles retain independent goal and native contracts.
use crate::provider_traits::{TestResult, lower};
use std::path::Path;
#[path = "coercion_provider/discovery.rs"]
mod discovery;
#[path = "coercion_provider/envelopes.rs"]
mod envelopes;
#[path = "coercion_provider/erasure.rs"]
mod erasure;
#[path = "coercion_provider/struct_goals.rs"]
mod struct_goals;

const POSITIVE: &[&str] = &[
    "array-slice",
    "zero-array",
    "generic-length",
    "equal-lifetime",
    "object-goal",
    "object-scoped",
    "object-associated",
    "coerce-reference",
    "coerce-pointer",
    "coerce-mut-to-const",
    "array-element-inference",
    "callable-element-alpha",
    "dyn-width-8",
    "dyn-width-32",
    "dyn-width-128",
    "dyn-callers-8",
    "dyn-callers-32",
    "dyn-callers-128",
    "formation-declarations",
];
const NEGATIVE: &[&str] = &[
    "wrong-element",
    "identity-scalar",
    "identity-slice",
    "shortening-not-equality",
    "zero-still-equality",
    "object-unpaid",
    "object-escape",
    "object-associated-wrong",
    "coerce-shared-to-mut",
    "explicit-unsize-impl",
    "invalid-coerce-scalar",
    "invalid-coerce-element",
    "dyn-unsize-denied",
    "lookalike-no-builtin",
    "coerce-missing-outlives",
    "ordinary-no-roles",
    "other-owner-no-builtin",
    "callable-element-correlation",
    "formation-source-unpaid",
    "formation-target-unpaid",
    "formation-region-unpaid",
    "formation-projection-unpaid",
    "formation-valid-mismatch",
    "formation-deferred-other-invalid",
];
fn source(name: &str, extension: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../pa31/tests/spec/unsizing-roles/{name}.{extension}"
        )),
    )?)
}
#[test]
fn actual_unsizing_roles_retain_goal_equations_and_all_caller_debts() -> TestResult {
    let mut failures = Vec::new();
    for &name in POSITIVE {
        match lower(&source(name, "rs")?) {
            Ok(_) => println!("{name}: accepted"),
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn source_unsizing_never_substitutes_layout_name_or_one_way_lifetimes() -> TestResult {
    let mut failures = Vec::new();
    for &name in NEGATIVE {
        if lower(&source(name, "rs")?).is_ok() {
            failures.push(name);
        }
    }
    assert!(failures.is_empty(), "accepted: {failures:?}");
    Ok(())
}

#[test]
fn nominal_coercion_implementations_prove_one_actual_changed_field() -> TestResult {
    let folder = "pa31/tests/spec/nominal-coercions";
    let cases = crate::resolve::fixture(&format!("{folder}/cases.tsv"))?;
    let mut failures = Vec::new();
    let mut count = 0;
    for line in cases.lines() {
        let (name, expected) = line.split_once('\t').ok_or("nominal case columns")?;
        let text = crate::resolve::fixture(&format!("{folder}/{name}"))?;
        let result = lower(&text);
        match (expected, result) {
            ("0", Ok(_)) => {}
            ("1", Err(error))
                if ![
                    "not implemented",
                    "budget",
                    "exhausted",
                    "lacks its completed",
                    "undefined",
                    "syntax",
                    "feature request",
                ]
                .iter()
                .any(|fence| error.to_string().contains(fence)) =>
            {
                println!("{name}: {error}");
            }
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|error| error.to_string(), |_| "accepted".into())
            )),
        }
        count += 1;
    }
    assert_eq!(count, 16);
    // Previously qualified Rust-positive required fixture remains a real goal.
    lower(&source("nominal-coerce-required", "rs")?)?;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn nominal_value_typing_retains_actual_goals_and_full_source_opponents() -> TestResult {
    let folder = "pa31/tests/spec/nominal-coercions/values";
    let text = crate::resolve::fixture(&format!("{folder}/native.rs"))?;
    let report = crate::resolve::source_provider(&text, "nominal_values")?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    let nominal = analysis
        .facts
        .unsizing_adjustments
        .values()
        .filter(|conversion| {
            matches!(
                conversion,
                rustgm::types::unsizing::Conversion::Nominal { .. }
            )
        })
        .count();
    assert_eq!(nominal, 6);
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())?;
    for name in ["escape", "mutability", "element", "moved-source"] {
        let text = crate::resolve::fixture(&format!("{folder}/{name}.rs"))?;
        let report = crate::resolve::source_provider(&text, "nominal_opponent")?;
        let error = rustgm::pipeline::analyze_source(&report)
            .and_then(
                |rustgm::pipeline::Analysis {
                     mut facts,
                     prepared,
                 }| {
                    rustgm::lower::instances::inspect_prepared(
                        &report,
                        &mut facts,
                        prepared,
                        |_, _| (),
                    )
                },
            )
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(
            ![
                "not implemented",
                "budget",
                "exhausted",
                "undefined",
                "unsupported",
                "wrapper conversion plan",
                "aggregate conversion plan"
            ]
            .iter()
            .any(|fence| error.contains(fence)),
            "{name}: {error}"
        );
        println!("{name}: {error}");
        let cause = match name {
            "escape" => "required lifetime does not outlive its target",
            "moved-source" => "local may be uninitialized at this read",
            _ => "trait obligation requires an unambiguous matching assumption or implementation",
        };
        assert!(error.contains(cause), "{name}: {error}");
    }
    assert_eq!(
        size_of::<rustgm::types::unsizing::Conversion<rustgm::types::TypeId>>(),
        12
    );
    assert_eq!(size_of::<rustgm::body::ir::Body>(), 456);
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn nominal_wrapper_values_preserve_transfer_effects_and_real_cleanup() -> TestResult {
    let folder = "pa31/tests/spec/nominal-coercions/values";
    let text = crate::resolve::fixture(&format!("{folder}/native.rs"))?;
    let report = crate::resolve::source_provider(&text, "nominal_native")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let output = root.join("obj/parallel/nominal-coercions-v1/candidate-native");
    std::fs::create_dir_all(&output)?;
    crate::ffi_program::check(
        &root,
        &output,
        "wrappers",
        &program,
        &crate::resolve::fixture(&format!("{folder}/observer.c"))?,
        &crate::resolve::fixture(&format!("{folder}/native.expected"))?,
    )?;
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn source_unsizing_goal_native_answers() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/coercion-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for &name in POSITIVE {
        let text = source(name, "rs")?;
        let program = lower(&text)?;
        std::fs::write(folder.join(format!("{name}.rs")), text)?;
        crate::ffi_program::check(root, &folder, name, &program, &source(name, "c")?, "7\n")?;
        println!("{name}: native7");
    }
    Ok(())
}

#[path = "coercion_provider/upcast_goals.rs"]
mod upcast_goals;
