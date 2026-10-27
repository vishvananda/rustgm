//! Selected shared RHS arithmetic preserves ordinary lookup, loans and execution order.
use super::{mutation::frontend, native};
use std::path::Path;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let path = match name {
        "add-original" => "primitive-add/reference-boundary".to_owned(),
        "sub-original" => "primitive-binary/reference-boundary".to_owned(),
        _ => format!("shared-value-operands/{name}"),
    };
    Ok(std::fs::read_to_string(
        root.join(format!("pa26/tests/spec/{path}.rs")),
    )?)
}
const POSITIVE: &[&str] = &[
    "add-original",
    "sub-original",
    "qualified",
    "generic",
    "dot-typed",
    "dot-inferred",
    "alias",
    "reified-static",
    "coerced-mut-borrow",
    "float",
    "wide",
    "order",
    "local-owner",
    "wrapper-reified",
    "overflow-add",
    "overflow-sub",
];
#[test]
fn shared_rhs_sources_keep_actual_type_mutability_loan_and_owner_controls() -> TestResult {
    for &name in POSITIVE {
        frontend::checked(&source(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in [
        "implicit-dot",
        "lexical-ambiguity",
        "mixed-width",
        "mutable-rhs",
        "deep-reference",
        "wrong-result",
        "bool",
        "char",
        "live-loan",
        "const-call",
        "reified",
    ] {
        let error = frontend::checked(&source(name)?)
            .err()
            .ok_or_else(|| format!("{name}: invalid source accepted"))?;
        println!("shared-rhs-negative={name} diagnostic={error}");
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned Rust and the qualified CPPGM backend"]
fn shared_rhs_literal_sources_execute_with_reference_and_overflow_contracts() -> TestResult {
    for &name in POSITIVE {
        let text = source(name)?;
        native::check_literal_cases(
            "parallel/pa26/shared-primitive-operands-v1/native",
            &[(name, &text, name.starts_with("overflow-"))],
        )?;
    }
    Ok(())
}
