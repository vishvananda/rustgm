//! rustc 1.98's `type_allowed_to_implement_copy` accepts scalars, `!`, raw
//! pointers, shared references and arrays as Copy targets without fields, and
//! does not check an array's element. Core's `impl Copy for ! {}` is therefore
//! accepted, as is an array Copy impl whose element is only Clone. A unique
//! reference, `str` or a slice is no target (E0206), and `!` still needs its
//! `Clone` supertrait.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/copy-impl-targets";
const UNSATISFIED: &str = "trait implementation does not satisfy the trait's requirements";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects. rustc
/// rejects `tuple-target` as a target (E0206); this compiler rejects it because
/// its `bool` element has no Copy impl here.
const CASES: [(&str, Option<&str>); 11] = [
    ("core-never", None),
    ("scalars", None),
    ("raw-pointers", None),
    ("core-array", None),
    ("array-clone-element", None),
    ("array-fixed-length", None),
    (
        "unique-reference",
        Some("unique references cannot implement source Copy"),
    ),
    ("str-target", Some(UNSATISFIED)),
    ("slice-target", Some(UNSATISFIED)),
    ("never-without-clone", Some(UNSATISFIED)),
    (
        "tuple-target",
        Some("source Copy implementation requires every field to implement actual Copy"),
    ),
];

/// Declared controls whose rustc outcome this analysis does not reach yet,
/// with the outcome it has instead. The assertion fails when it changes.
/// - `fn-pointer-target` and `tuple-copy-elements`: rustc gives fn pointers
///   and tuples builtin Copy and rejects a written Copy impl for them (E0206);
///   this compiler accepts the written impls.
const GAPS: [(&str, Option<&str>); 2] =
    [("fn-pointer-target", None), ("tuple-copy-elements", None)];

#[test]
fn copy_impls_accept_exactly_rustc_targets_without_fields() -> TestResult {
    for (name, expected) in CASES.into_iter().chain(GAPS) {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "copy_impl_targets")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
