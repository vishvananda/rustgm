//! Captured object conversions retain per-use proofs and select sealed native bodies.
#[path = "harness/checked_sources.rs"]
mod checked;
#[path = "harness/native_references.rs"]
mod native;
#[path = "captured_dispatch/quantifiers.rs"]
mod quantifiers;
use std::path::PathBuf;

const ACCEPTED: &[&str] = &[
    "shared",
    "mutable",
    "weaker",
    "borrowed",
    "generic",
    "generic-weaker",
    "upcast",
    "inherited",
    "aggregate",
    "once-view",
    "generic-consumer",
    "higher-ranked",
    "result-lifetime",
    "destruction",
    "captured-zst",
    "multiple-lifetimes",
    "quantified-capture",
];
const OPPOSING: &[(&str, i32, [i32; 3], &str)] = &[
    ("result", 1, [1, 1, 1], "cannot infer an output lifetime"),
    (
        "mutable-to-shared",
        1,
        [1, 1, 1],
        "trait obligation requires an unambiguous",
    ),
    (
        "owned-object",
        1,
        [1, 1, 1],
        "trait obligation requires an unambiguous",
    ),
    (
        "borrow-escape",
        1,
        [0, 1, 1],
        "access conflicts with a live reference loan",
    ),
    (
        "late-permission",
        1,
        [0, 1, 1],
        "required lifetime does not outlive its target",
    ),
    (
        "loan-conflict",
        1,
        [0, 1, 1],
        "access conflicts with a live reference loan",
    ),
];
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/captured-dispatch/{name}.rs"))
}
#[test]
fn captured_object_views_match_independent_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/captured-dispatch/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &name in ACCEPTED {
        checked::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    for &(name, rust, frontend, reason) in OPPOSING {
        checked::check_modes(&fixture(name), &evidence, name, (rust, frontend, reason))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM backend; run make test-maintainer-native"]
fn captured_objects_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &name in ACCEPTED {
        let source = std::fs::read_to_string(fixture(name))?;
        let body = source
            .strip_prefix(native::CHECK)
            .ok_or("missing check prefix")?;
        native::check_cases(
            "parallel/pa24/captured-dispatch/native",
            &[(name, body, false)],
        )?;
    }
    Ok(())
}
