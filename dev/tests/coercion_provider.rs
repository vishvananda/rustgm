//! Actual source unsizing roles retain independent goal and native contracts.
use crate::provider_traits::{TestResult, lower};
use std::path::Path;

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
