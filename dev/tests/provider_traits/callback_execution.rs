//! Callback execution retains its defining and reached `FnOnce` obligations.
use super::{TestResult, lower};
pub(super) fn fixture(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-const-callable")
            .join(format!("{name}.rs")),
    )?)
}
#[test]
fn source_const_function_item_invocation_matches_original_and_opposite_contracts() -> TestResult {
    let mut mismatches = Vec::new();
    for (name, accepted) in [
        ("original-native29", true),
        ("runtime-nonconst29", true),
        ("ordinary-pointer29", true),
        ("runtime-const-helper-pointer29", true),
        ("ordinary-function-root7", true),
        ("const-helper-pointer-reject", false),
        ("original-nonconst-reject", false),
        ("unused-ordinary-bound-reject", false),
        ("conditional-target-required", false),
        ("work/repeated-8", true),
        ("work/repeated-32", true),
        ("work/repeated-128", true),
    ] {
        let result = lower(&fixture(name)?);
        println!(
            "{name}: {} {:?}",
            result.is_ok(),
            result.as_ref().err().map(ToString::to_string)
        );
        if result.is_ok() != accepted {
            mismatches.push(name);
        }
        if name == "conditional-target-required" {
            let error = result
                .err()
                .ok_or("conditional item debt silently accepted")?
                .to_string();
            assert!(
                error.contains("scoped continuation") || error.contains("conditional Invoke"),
                "{error}"
            );
        }
    }
    assert!(
        mismatches.is_empty(),
        "const invocation outcomes: {mismatches:?}"
    );
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn original_const_invocation_and_runtime_opposites_have_native_twenty_nine() -> TestResult {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/const-callable-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (name, expected) in [
        ("original-native29", 29),
        ("runtime-nonconst29", 29),
        ("ordinary-pointer29", 29),
        ("runtime-const-helper-pointer29", 29),
        ("work/repeated-8", 29),
        ("work/repeated-32", 29),
        ("work/repeated-128", 29),
        ("ordinary-function-root7", 7),
    ] {
        super::native::execute(&folder, &name.replace('/', "-"), &fixture(name)?, expected)?;
    }
    Ok(())
}
