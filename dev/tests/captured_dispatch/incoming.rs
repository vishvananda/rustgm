//! Input-only conditions keep explicit results checked without weakening actual callers.
use super::{checked, native};
use std::path::PathBuf;
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/capture-inputs/{name}.rs"))
}
#[test]
fn capture_input_conditions_match_independent_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/reference-result/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for name in ["static-input", "nested-reference", "opaque-call"] {
        checked::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    for name in [
        "nonstatic-input",
        "owned-local",
        "public-input",
        "opaque-escape",
        "nested-binder",
    ] {
        checked::check_modes(
            &fixture(name),
            &evidence,
            name,
            (
                1,
                [0, 1, 1],
                "required lifetime does not outlive its target",
            ),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM backend; run make test-maintainer-native"]
fn capture_input_conditions_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in ["static-input", "nested-reference"] {
        let source = std::fs::read_to_string(fixture(name))?;
        let body = source
            .strip_prefix(native::CHECK)
            .ok_or("missing check prefix")?;
        native::check_cases(
            "parallel/pa24/reference-result/native",
            &[(name, body, false)],
        )?;
    }
    Ok(())
}
