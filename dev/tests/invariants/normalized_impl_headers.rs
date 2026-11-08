//! An impl header that names a projection of its own parameter,
//! `impl<R: Try> ControlFlow<R, R::Output>`, normalizes it under the impl's
//! bounds, as a function signature does under its own.
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/impl-header-projections";

/// Independently checked rustc 1.98.0 outcomes: `None` accepts, otherwise the
/// first error this compiler reports for the control rustc also rejects.
/// The `self-` controls name the impl's own `Self` in its bounds, so a header
/// projection stays as written until the impl's environment is complete.
const CASES: [(&str, Option<&str>); 14] = [
    ("control-flow", None),
    ("try-residual", None),
    ("pair", None),
    ("method-self", None),
    ("qualified", None),
    ("trait-impl", None),
    ("where-clause", None),
    (
        "unbounded",
        Some("associated type requires an unambiguous implementation"),
    ),
    (
        "unconstrained",
        Some("impl type and const parameters must constrain its target"),
    ),
    ("self-where", None),
    ("self-nested", None),
    ("self-argument", None),
    ("self-plain", None),
    (
        "self-unbounded",
        Some("associated type requires an unambiguous implementation"),
    ),
];

/// A declared control whose rustc outcome this analysis does not reach yet,
/// with the outcome it has instead. The assertion fails when it changes.
/// `self-equality`: rustc accepts `impl<R: Try<Output = u8>> W<R, R::Output>
/// where Self: Sized`. Its header must stay as written while `Self` names it,
/// and the completed environment normalizes it to `W<R, u8>`, which is
/// rejected by name instead of re-forming the header.
const GAPS: [(&str, Option<&str>); 1] = [(
    "self-equality",
    Some("impl header projection must stay rigid when the impl's bounds name `Self`"),
)];

fn analyze(name: &str) -> TestResult<Result<(), String>> {
    let path = format!("{CONTROLS}/{name}.rs");
    let source = crate::test_source::fixture(&path)?;
    let report = crate::test_source::source_provider(&source, "impl_header")?;
    Ok(crate::pipeline::analyze_source(&report)
        .map(|_| ())
        .map_err(|error| error.message.clone()))
}

#[test]
fn projected_impl_headers_normalize_under_the_impl_bounds_like_rustc() -> TestResult {
    for (name, expected) in CASES {
        match (expected, analyze(name)?) {
            (None, Ok(())) => {}
            (Some(message), Err(error)) => assert_eq!(error, message, "{name}"),
            (None, Err(error)) => return Err(format!("{name}: rejected: {error}").into()),
            (Some(message), Ok(())) => {
                return Err(format!("{name}: accepted, not {message}").into());
            }
        }
    }
    for (name, expected) in GAPS {
        let expected = expected.map_or(Ok(()), |message| Err(message.to_owned()));
        assert_eq!(analyze(name)?, expected, "{name}");
    }
    Ok(())
}
