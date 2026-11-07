//! Rust 2015/2018 rules either follow the frozen pinned-rustc outcome or fail
//! with the typed "not implemented for edition" diagnostic, never a newer rule.
use std::{path::Path, process::Command};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const FENCE: &str = "is not implemented for edition";
const OUTCOMES: &str = "pa34/tests/legacy-edition-outcomes.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expected {
    /// The compiler implements the rule and accepts.
    Accepted,
    /// The compiler implements the rule and rejects without a fence.
    Rejected,
    /// The compiler fails with the typed edition fence.
    Fenced,
    /// The rule is shared before 2024: the Rust 2021 outcome is reproduced.
    Same,
}
use Expected::{Accepted, Fenced, Rejected, Same};

/// Declared before any candidate run; mirrors the frozen outcome file.
const CASES: &[(&str, &str, Expected)] = &[
    ("root-import", "2015", Accepted),
    ("root-import", "2018", Rejected),
    ("pattern-fragment", "2015", Accepted),
    ("pattern-fragment", "2018", Accepted),
    ("pattern-fragment", "2021", Rejected),
    ("prelude-convert", "2015", Rejected),
    ("prelude-convert", "2018", Rejected),
    ("prelude-convert-import", "2015", Same),
    ("prelude-convert-import", "2018", Same),
    ("prelude-from-iterator", "2015", Rejected),
    ("prelude-from-iterator", "2018", Rejected),
    ("prelude-future", "2015", Rejected),
    ("prelude-future", "2018", Rejected),
    ("keyword-async", "2015", Fenced),
    ("keyword-async", "2018", Rejected),
    ("keyword-dyn", "2015", Fenced),
    ("keyword-dyn", "2018", Rejected),
    ("dyn-type", "2015", Accepted),
    ("dyn-type", "2018", Accepted),
    ("async-fn", "2015", Fenced),
    ("async-fn", "2018", Same),
    ("reserved-prefix", "2015", Fenced),
    ("reserved-prefix", "2018", Fenced),
    ("guarded-string", "2015", Same),
    ("guarded-string", "2018", Same),
    ("expression-fragment", "2015", Same),
    ("expression-fragment", "2018", Same),
    ("panic-brace", "2015", Same),
    ("panic-brace", "2018", Same),
    ("panic-payload", "2015", Same),
    ("panic-payload", "2018", Same),
    ("unreachable-brace", "2015", Same),
    ("unreachable-brace", "2018", Same),
    ("assert-brace", "2015", Same),
    ("assert-brace", "2018", Same),
    ("bare-trait-object", "2015", Fenced),
    ("bare-trait-object", "2018", Fenced),
    ("dot-dot-dot-pattern", "2015", Fenced),
    ("dot-dot-dot-pattern", "2018", Fenced),
    ("anonymous-parameter", "2015", Fenced),
    ("anonymous-parameter", "2018", Rejected),
    ("absolute-root", "2015", Accepted),
    ("absolute-root", "2018", Rejected),
    ("nested-import", "2015", Rejected),
    ("nested-import", "2018", Accepted),
    ("injected-core", "2015", Accepted),
    ("injected-core", "2018", Rejected),
    ("use-core", "2015", Same),
    ("use-core", "2018", Same),
    ("closure-disjoint", "2015", Rejected),
    ("closure-disjoint", "2018", Rejected),
    ("closure-disjoint", "2021", Accepted),
    ("closure-wildcard", "2015", Rejected),
    ("closure-wildcard", "2018", Rejected),
    ("closure-wildcard", "2021", Accepted),
    ("closure-move-field", "2015", Rejected),
    ("closure-move-field", "2018", Rejected),
    ("closure-move-field", "2021", Accepted),
    ("closure-unique", "2015", Accepted),
    ("closure-unique", "2018", Accepted),
    ("closure-unique", "2021", Accepted),
    ("array-into-iter", "2015", Fenced),
    ("array-into-iter", "2018", Fenced),
    ("mixed-matches", "2015", Same),
    ("mixed-matches", "2018", Same),
    ("driver-edition/keyword", "2015", Accepted),
    ("driver-edition/keyword", "2018", Accepted),
    ("foreign-editions/plain-block", "2015", Same),
    ("foreign-editions/plain-block", "2018", Same),
    ("foreign-editions/bare-no-mangle", "2015", Same),
    ("foreign-editions/bare-no-mangle", "2018", Same),
    ("pattern-editions/mut-reset", "2015", Same),
    ("pattern-editions/mut-reset", "2018", Same),
    ("temporary-editions/tail-borrow", "2015", Same),
    ("temporary-editions/tail-borrow", "2018", Same),
    ("opaque-editions/implicit-hidden-reference", "2015", Same),
    ("opaque-editions/implicit-hidden-reference", "2018", Same),
    ("never-editions/unit-only-2021", "2018", Same),
    ("use-core-option", "2015", Accepted),
    ("use-core-option", "2018", Accepted),
    ("body-use-core", "2015", Accepted),
    ("body-use-core", "2018", Accepted),
];

struct Outcome {
    status: Option<i32>,
    first: String,
}

fn root() -> Result<&'static Path> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repository root")?)
}

fn fixture(name: &str) -> Result<std::path::PathBuf> {
    let relative = if name.contains('/') {
        format!("pa34/tests/spec/{name}.rs")
    } else {
        format!("pa34/tests/spec/legacy-stdlib-editions/{name}.rs")
    };
    Ok(root()?.join(relative))
}

fn run(path: &Path, edition: &str) -> Result<Outcome> {
    let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-loans")
        .arg(format!("--edition={edition}"))
        .arg(path)
        .output()?;
    let stderr = String::from_utf8(output.stderr)?;
    let first = stderr.lines().next().unwrap_or_default().to_owned();
    Ok(Outcome {
        status: output.status.code(),
        first,
    })
}

fn matches(name: &str, edition: &str, expected: Expected) -> Result<Option<String>> {
    let path = fixture(name)?;
    let actual = run(&path, edition)?;
    let fenced = actual.first.contains(&format!("{FENCE} {edition}"));
    let matched = match expected {
        Accepted => actual.status == Some(0),
        Rejected => actual.status == Some(1) && !actual.first.contains(FENCE),
        Fenced => actual.status == Some(1) && fenced,
        Same => {
            let newer = run(&path, "2021")?;
            actual.status == newer.status && actual.first == newer.first
        }
    };
    Ok((!matched).then(|| {
        format!(
            "{name}/{edition}: {:?} {} (expected {expected:?})",
            actual.status, actual.first
        )
    }))
}

#[test]
fn legacy_rules_follow_frozen_outcomes_or_fail_with_the_typed_fence() -> Result {
    let mut failures = Vec::new();
    for &(name, edition, expected) in CASES {
        failures.extend(matches(name, edition, expected)?);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// The table must be the frozen file's declared candidate column, in order.
#[test]
fn declared_table_mirrors_the_frozen_outcome_file() -> Result {
    let text = std::fs::read_to_string(root()?.join(OUTCOMES))?;
    let cases = text
        .split("\"mixed\": [")
        .next()
        .ok_or("missing outcome cases")?;
    let field = |line: &str, key: &str| {
        line.trim()
            .strip_prefix(&format!("\"{key}\": \""))
            .map(|rest| rest.trim_end_matches([',', '"']).to_owned())
    };
    let (mut name, mut edition, mut declared) = (String::new(), String::new(), Vec::new());
    for line in cases.lines() {
        if let Some(value) = field(line, "name") {
            name = value;
        } else if let Some(value) = field(line, "edition") {
            edition = value;
        } else if let Some(value) = field(line, "rustgm") {
            declared.push((name.clone(), edition.clone(), value));
        }
    }
    let table: Vec<_> = CASES
        .iter()
        .map(|&(name, edition, expected)| {
            let kind = match expected {
                Accepted => "accepted",
                Rejected => "rejected",
                Fenced => "fenced",
                Same => "same-as-2021",
            };
            (name.to_owned(), edition.to_owned(), kind.to_owned())
        })
        .collect();
    assert_eq!(declared, table);
    Ok(())
}
