//! Rustc's winnowing drops a candidate that another applicable candidate
//! specializes, and `assoc_def` reads an omitted associated type from the
//! nearest defining ancestor. Declarations select through one completed pair
//! comparison before the graph closes; finality, always-applicability and
//! the feature gate still reject their opponents.
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/specialization-selection";

/// Independently checked rustc 1.98.0 outcomes: `None` accepts, otherwise the
/// first error this compiler reports for the control rustc also rejects.
/// `no-relation` is rejected by the selection that meets both impls first;
/// without that use, coherence reports the unordered pair itself.
const CASES: [(&str, Option<&str>); 9] = [
    ("step-by-inherited", None),
    ("fuse-marker", None),
    ("zip-chain", None),
    ("no-relation", Some("invalid canonical trait selection")),
    (
        "final-method-override",
        Some("specializing method overrides a final parent member"),
    ),
    (
        "not-always-applicable",
        Some("minimal specialization introduces a non-marker predicate"),
    ),
    (
        "type-override",
        Some("specializing implementation overrides a final parent associated type"),
    ),
    ("inherited-mismatch", Some("incompatible types")),
    (
        "default-type",
        Some("default declaration requires its specialization feature or exact token allowance"),
    ),
];

fn analyze(name: &str) -> TestResult<Result<(), String>> {
    let path = format!("{CONTROLS}/{name}.rs");
    let source = crate::test_source::fixture(&path)?;
    // The feature gate rejects `default type` while the crate resolves; its
    // located message keeps the gate's own text as its suffix.
    let report = match crate::test_source::source_provider(&source, "specialization_selection") {
        Ok(report) => report,
        Err(error) => return Ok(Err(error)),
    };
    Ok(crate::pipeline::analyze_source(&report)
        .map(|_| ())
        .map_err(|error| error.message.clone()))
}

#[test]
fn specializing_candidates_select_and_inherit_like_rustc() -> TestResult {
    for (name, expected) in CASES {
        match (expected, analyze(name)?) {
            (None, Ok(())) => {}
            (Some(message), Err(error)) => assert!(
                error == message || error.ends_with(&format!(": {message}")),
                "{name}: {error}"
            ),
            (None, Err(error)) => return Err(format!("{name}: rejected: {error}").into()),
            (Some(message), Ok(())) => {
                return Err(format!("{name}: accepted, not {message}").into());
            }
        }
    }
    Ok(())
}

/// The unordered pair without a selecting use reaches coherence, which names
/// the missing specialization ordering instead of a folded selection cause.
#[test]
fn an_unordered_overlap_without_a_use_is_a_coherence_error() -> TestResult {
    let path = format!("{CONTROLS}/no-relation.rs");
    let source = crate::test_source::fixture(&path)?;
    let unused = source
        .rsplit_once("pub fn both")
        .map(|(declarations, _)| declarations)
        .ok_or("no-relation lost its use")?;
    let report = crate::test_source::source_provider(unused, "specialization_unused")?;
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("an unordered overlap was accepted")?;
    assert_eq!(
        error.message,
        "overlapping implementation headers have no specialization ordering"
    );
    Ok(())
}
