//! Independently qualified source nominal formation uses retained ordinary debts.
pub(super) const CASES: &[(&str, &str, bool)] = &[
    ("nested-freeze", "formation/nested-freeze.rs", true),
    ("reference-freeze", "formation/reference-freeze.rs", true),
    ("paid-copy-nominal", "formation/paid-copy-nominal.rs", true),
    (
        "unpaid-copy-nominal",
        "formation/unpaid-copy-nominal.rs",
        false,
    ),
    (
        "unpaid-reference-nominal",
        "formation/unpaid-reference-nominal.rs",
        false,
    ),
    ("nested-meta-sized", "formation/nested-meta-sized.rs", true),
    (
        "reference-pointee-sized",
        "formation/reference-pointee-sized.rs",
        true,
    ),
    (
        "nested-ordinary-destruct",
        "formation/nested-ordinary-destruct.rs",
        true,
    ),
    (
        "generic-paid-formation",
        "formation/generic-paid-formation.rs",
        true,
    ),
    (
        "repeated-distinct-formation",
        "formation/repeated-distinct-formation.rs",
        true,
    ),
    (
        "pointer-override-non-freeze-argument",
        "formation/pointer-override-non-freeze-argument.rs",
        true,
    ),
    (
        "callable-local-nominal",
        "formation/callable-local-nominal.rs",
        true,
    ),
    (
        "callable-local-unpaid-nominal",
        "formation/callable-local-unpaid-nominal.rs",
        false,
    ),
];
#[test]
fn source_nominal_formation_preserves_nested_reference_and_opposing_bound_proofs()
-> super::TestResult {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        let source = super::fixture(source)?;
        let result = super::lower(&source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
