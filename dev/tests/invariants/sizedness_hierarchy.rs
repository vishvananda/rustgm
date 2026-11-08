//! Trait and nominal arguments follow rustc 1.98's sizedness hierarchy: a rigid
//! projection is sized from its associated type's own item bounds, including the
//! implicit `MetaSized` of a `?Sized` result, even when the environment holds
//! sizing assumptions over other projections, and `PointeeSized` holds for
//! every type.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const SIZEDNESS: &str = "pa31/tests/spec/projection-sizedness";
const NORMALIZED: &str = "pa31/tests/spec/normalized-projection-sizing";
const UNSATISFIED: &str = "trait argument does not satisfy its declaration requirements";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects (E0277).
const CASES: [(&str, &str, Option<&str>); 13] = [
    (SIZEDNESS, "pointee-rhs", None),
    (SIZEDNESS, "meta-rhs", None),
    (SIZEDNESS, "pin-partial-eq", None),
    (SIZEDNESS, "struct-where", None),
    (SIZEDNESS, "pointee-item", None),
    (SIZEDNESS, "sized-item-meta", None),
    (SIZEDNESS, "sized-rhs", Some(UNSATISFIED)),
    (SIZEDNESS, "pointee-item-meta", Some(UNSATISFIED)),
    (NORMALIZED, "flatten-next", None),
    (NORMALIZED, "inherent-item", None),
    (NORMALIZED, "free-item", None),
    (NORMALIZED, "relaxed-meta", None),
    (
        NORMALIZED,
        "relaxed-option",
        Some("nominal type argument must be sized"),
    ),
];

#[test]
fn rigid_projections_prove_sizing_arguments_from_their_item_bounds() -> TestResult {
    for (controls, name, expected) in CASES {
        let path = format!("{controls}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "projection_sizing")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
