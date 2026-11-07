//! A const or static item's own annotation types its constants in the item's
//! lexical module even though the item has no generic binder.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/static-constant-modules";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the rejected control.
const CASES: [(&str, Option<&str>); 3] = [
    ("restricted-const-length", None),
    ("restricted-macro-length", None),
    (
        "private-initializer-through-alias",
        Some("semantic type or associated item is private"),
    ),
];

#[test]
fn binderless_item_constants_see_their_own_module() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "constant_module")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
