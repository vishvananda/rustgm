//! Nested query namespaces preserve source bounds before anonymous specialization.
use super::native;
use std::path::{Path, PathBuf};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ACCEPTED: &[&str] = &[
    "scalar",
    "scalar-nested",
    "scalar-closure",
    "required",
    "nested",
    "siblings",
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/inline-query-context")
        .join(format!("{name}.rs"))
}
#[test]
fn nested_query_callers_preserve_their_actual_parent_premises() -> TestResult {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-inline-queries/source")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for name in ACCEPTED {
        super::checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    for (name, reason) in [
        ("foreign-parent", "trait obligation"),
        ("unused-runtime", "non-const"),
    ] {
        super::checked_sources::check(&fixture(name), &evidence, name, (1, 1, reason))?;
    }
    super::checked_sources::check_modes(
        &fixture("siblings-drop"),
        &evidence,
        "siblings-drop",
        (1, [0, 1, 1], "cannot run destructors"),
    )?;
    Ok(())
}
#[test]
fn projected_inline_results_retain_their_required_execution_frontiers() -> TestResult {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-inline-queries/remaining")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for (name, rust) in [
        ("closure", 0),
        ("paid-lifetime", 0),
        ("foreign-lifetime", 1),
    ] {
        super::checked_sources::check(
            &fixture(name),
            &evidence,
            name,
            (
                rust,
                1,
                "nested output requires its checked parent contract",
            ),
        )?;
    }
    super::checked_sources::check_modes(
        &fixture("unpaid-lifetime"),
        &evidence,
        "unpaid-lifetime",
        (
            1,
            [0, 1, 1],
            "source computation retains a non-lifetime parameter",
        ),
    )?;
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn selected_inline_query_values_execute() -> TestResult {
    let sources = ACCEPTED
        .iter()
        .map(|name| Ok((*name, std::fs::read_to_string(fixture(name))?)))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let cases = sources
        .iter()
        .map(|(name, text)| {
            Ok((
                *name,
                text.strip_prefix(native::CHECK).ok_or("check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa25-inline-queries/native", &cases)
}
