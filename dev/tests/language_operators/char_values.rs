//! Char MIN/MAX retain ordinary associated lookup, CTFE and four-byte scalar storage.
use super::{mutation::frontend, native};
use std::path::Path;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let tail = if name == "char-required-original" {
        "primitive-constants/char-required".to_owned()
    } else {
        format!("char-constants/{name}")
    };
    Ok(std::fs::read_to_string(
        root.join(format!("pa26/tests/spec/{tail}.rs")),
    )?)
}
const ACCEPTED: &[&str] = &[
    "char-required-original",
    "min",
    "max",
    "alias",
    "qualified",
    "lexical-shadow",
    "alias-shadow",
    "core-shadow",
    "ctfe",
    "patterns",
    "borrow-temporary",
    "generic-char-value",
    "generic-integer-cast",
    "self-priority",
    "integer-control",
    "widths",
];
#[test]
fn char_values_preserve_their_real_source_lookup_and_scalar_contracts() -> TestResult {
    for &name in ACCEPTED {
        frontend::checked(&source(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in [
        "wrong-result",
        "member-arguments",
        "primitive-arguments",
        "not-callable",
        "assign-constant",
        "generic-wrong-value",
        "unbounded-generic",
        "invalid-char-value",
    ] {
        let error = frontend::checked(&source(name)?)
            .err()
            .ok_or_else(|| format!("{name}: invalid char input accepted"))?;
        println!("char-negative={name} diagnostic={error}");
        assert!(!error.contains("not implemented"), "{name}: {error}");
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned Rust and the qualified CPPGM backend"]
fn char_values_execute_literal_sources_through_existing_widths() -> TestResult {
    for &name in ACCEPTED {
        let text = source(name)?;
        native::check_literal_cases_capped(
            "parallel/pa26/char-constant-values-v1/native",
            &[(name, &text, false)],
        )?;
    }
    Ok(())
}
