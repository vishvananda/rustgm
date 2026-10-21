//! Owned helpers keep selected comparisons, owned cleanup and checked runtime failure.
use super::{mutation::frontend, native, rustc};
use std::path::{Path, PathBuf};
const CASES: &[(&str, bool, bool)] = &[
    ("const-not-eligible", false, false),
    ("tie-min", true, false),
    ("tie-max", true, false),
    ("clamp-middle", true, false),
    ("clamp-low", true, false),
    ("clamp-high", true, false),
    ("clamp-equal", true, false),
    ("clamp-inverted", true, true),
    ("primitive-helpers", true, false),
    ("generic-selected", true, false),
    ("qualified-reified", true, false),
    ("borrowed-contained", true, false),
    ("override-helpers", true, false),
    ("relation-override-min-max", true, false),
    ("relation-override-clamp", true, false),
    ("atomic-traits", false, false),
    ("float-ord-nan", false, false),
    ("missing-ord", false, false),
    ("wrong-helper-result", false, false),
    ("moved-owned", false, false),
    ("lookalike", true, false),
    ("drop-min", true, false),
    ("drop-max", true, false),
    ("drop-clamp", true, false),
    ("escaping-owned", false, false),
    ("clamp-override-panic", true, true),
    ("argument-order", true, false),
    ("unordered-clamp", true, true),
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../pa26/tests/spec/ord-helpers/{name}.rs"))
}
#[test]
fn owned_helper_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted, _) in CASES {
        let source = std::fs::read_to_string(fixture(name))?;
        rustc::qualify(
            &format!("parallel/pa26/ord-helpers/independent/{name}"),
            &source,
            accepted,
        )?;
        let checked = frontend::checked(&source);
        if checked.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", checked.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn owned_helpers_execute() -> Result<(), Box<dyn std::error::Error>> {
    let names = CASES.iter().filter(|case| case.1).collect::<Vec<_>>();
    let sources = names
        .iter()
        .map(|case| std::fs::read_to_string(fixture(case.0)))
        .collect::<Result<Vec<_>, _>>()?;
    let cases = names
        .iter()
        .zip(&sources)
        .map(|(case, source)| {
            Ok((
                case.0,
                source.strip_prefix(native::CHECK).ok_or("check prefix")?,
                case.2,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26/ord-helpers/native", &cases)
}
