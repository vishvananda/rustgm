//! Original Rust 2021/2024 inputs have independently retained source outcomes.
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn check(cases: &[(&str, bool)]) -> TestResult {
    let mut failures = Vec::new();
    for &(name, accepted) in cases {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/primitive-inherent")
            .join(format!("{name}.rs"));
        let source = std::fs::read_to_string(path)?;
        let result = crate::test_source::resolve(&source, true).and_then(|report| {
            crate::pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn primitive_owner_permission_uses_actual_attributes_and_nominal_ownership() -> TestResult {
    check(&[
        ("ordinary-primitive-rejected", false),
        ("misleading-core-name", false),
        ("request-alone-insufficient", false),
        ("flag-without-request", false),
        ("empty-primitive-with-request", true),
        ("empty-primitive-without-request", false),
        ("flag-does-not-authorize-generic-self", false),
        ("flag-does-not-authorize-foreign-nominal", false),
        ("flag-does-not-authorize-orphan", false),
        ("member-alternative-missing-one", false),
        ("member-alternative-wrong-owner", false),
        ("member-alternative-const-owner", false),
        ("core-flag-wrong-owner", false),
        ("core-flag-duplicate", false),
        ("core-flag-list", false),
        ("core-flag-value", false),
        ("core-flag-unsafe", false),
        ("core-flag-disabled-cfg", false),
        ("disabled-primitive-owner", true),
        ("raw-mutability-exclusion", false),
    ])
}

#[test]
fn structural_members_use_full_self_and_real_receiver_adjustments() -> TestResult {
    check(&[
        ("actual-core-flag", true),
        ("ordinary-local-nominal", true),
        ("member-alternative", true),
        ("core-flag-selected-cfg", true),
        ("primitive-alias", true),
        ("different-scalars-same-name", true),
        ("different-scalar-missing-member", false),
        ("primitive-associated-function", true),
        ("primitive-associated-constant", true),
        ("str-receiver", true),
        ("slice-receiver", true),
        ("array-generic-receiver", true),
        ("array-const-exclusion", false),
        ("array-disjoint-same-name", true),
        ("raw-pointer-receiver", true),
        ("reference-receiver", true),
        ("mutable-reference-receiver", true),
        ("unit-receiver", true),
        ("tuple-receiver", true),
        ("function-pointer-receiver", false),
        ("never-associated-function", true),
        ("generic-primitive-bound-success", true),
        ("generic-primitive-bound-failure", false),
    ])
}

#[test]
fn structural_duplicates_remain_demanded_without_weakening_nominal_coherence() -> TestResult {
    check(&[
        ("duplicate-primitive-members", true),
        ("duplicate-across-used", false),
        ("duplicate-within-impl-unused", true),
        ("duplicate-constants-unused", true),
        ("duplicate-constants-used", false),
        ("generic-overlap-unused", true),
        ("generic-overlap-used", false),
        ("nominal-overlap-still-rejected", false),
        ("fn-pointer-declaration-only", true),
        ("fn-pointer-qualified-call", false),
        ("fn-pointer-alias-qualified-call", false),
        ("fn-pointer-trait-method-control", true),
        ("fn-pointer-member-alternative", false),
    ])
}
