//! Lexicographic calls retain element evidence, actual lengths and independent receiver borrows.
use super::{mutation::frontend, native, rustc};
const CASES: &[(&str, bool)] = &[
    ("array-ranks", true),
    ("slice-length", true),
    ("float-unordered", true),
    ("public-override", true),
    ("ord-versus-partial", true),
    ("generic-arrays", true),
    ("qualified-array", true),
    ("reified-array", true),
    ("reified-slice", true),
    ("nested-arrays-original", true),
    ("mutable-slices", false),
    ("empty-arrays", true),
    ("independent-element-lifetimes", true),
    ("lookalike", true),
    ("noncopy-borrow", true),
    ("different-length", false),
    ("array-slice", false),
    ("heterogeneous-array", false),
    ("heterogeneous-slice", false),
    ("float-ord", false),
    ("missing-prerequisite", false),
    ("missing-element-bound", false),
    ("borrow-escape", false),
    ("const-denied", false),
    ("mutable-slices-borrowed-rhs", true),
    ("bare-order", true),
    ("later-unrelated-query", true),
    ("module-origin", true),
    ("macro-origin", true),
    ("array-helpers", true),
    ("zero-sized-elements", true),
    ("short-circuit", true),
    ("empty-unpaid", false),
    ("distinct-symbolic-lengths", false),
    ("uninitialized", false),
    ("live-mutable-loan", false),
    ("invariant-inner-unpaid", true),
    ("invariant-inner-paid", true),
    ("conditional-lifetime-unpaid", false),
    ("conditional-lifetime-paid", true),
    ("comparison-does-not-extend-source", false),
    ("reborrow-storage-duration", true),
    ("invariant-parameters-unpaid", false),
    ("invariant-parameters-paid", true),
];
fn source(name: &str) -> Result<String, std::io::Error> {
    let path = if name == "nested-arrays-original" {
        "sequence-equality/nested-arrays.rs".to_string()
    } else {
        format!("sequence-ordering/{name}.rs")
    };
    std::fs::read_to_string(format!(
        "{}/../pa26/tests/spec/{path}",
        env!("CARGO_MANIFEST_DIR")
    ))
}
#[test]
fn sequence_ordering_matches_declared_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let source = source(name)?;
        rustc::qualify(&format!("sequence-ordering-{name}"), &source, accepted)?;
        let result = frontend::checked(&source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn sequence_ordering_preserves_partial_and_total_dispatch() -> Result<(), Box<dyn std::error::Error>>
{
    let sources = CASES
        .iter()
        .filter(|case| case.1 && case.0 != "invariant-inner-unpaid")
        .map(|&(name, _)| Ok((name, source(name)?)))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let cases = sources
        .iter()
        .map(|(name, source)| {
            Ok((
                *name,
                source.strip_prefix(native::CHECK).ok_or("check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26-sequence-ordering/native", &cases)
}
