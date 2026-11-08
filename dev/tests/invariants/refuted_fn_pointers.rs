//! `T: FnPtr` holds only for a function pointer, decided by the outer Self type
//! as in rustc 1.98. Stable coherence uses that for a knowable Self type; a
//! reference or fundamental nominal over an unknown type stays possible. With
//! `with_negative_coherence`, as in core, rustc's builtin `impl !FnPtr for T`
//! refutes every rigid type that is not a function pointer, so core's reference
//! impls are disjoint from its `FnPtr` blankets. A placeholder refutes nothing.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/fn-pointer-coherence";
const OVERLAP: &str = "overlapping trait implementation headers require coherence proof";
const UNORDERED: &str = "overlapping implementation headers have no specialization ordering";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects (E0119).
const CASES: [(&str, Option<&str>); 14] = [
    ("core-references", None),
    ("reference", None),
    ("fundamental", None),
    ("trait-object", None),
    ("parameter", Some(OVERLAP)),
    ("function-pointer", Some(OVERLAP)),
    ("strict-tuple", None),
    ("stable-core-references", Some(UNORDERED)),
    ("stable-reference", Some(OVERLAP)),
    ("stable-fundamental", Some(OVERLAP)),
    ("stable-tuple", None),
    ("stable-nominal", None),
    ("stable-raw-pointer", None),
    ("stable-array", None),
];

#[test]
fn only_function_pointers_implement_fn_ptr_and_negative_coherence_refutes_rigid_types() -> TestResult
{
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "fn_pointer_coherence")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
