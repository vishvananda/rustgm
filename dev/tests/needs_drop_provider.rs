//! Actual source `needs_drop` is a checked bool kernel, never a placeholder API.
use crate::provider_traits::{TestResult, lower, report};
use rustgm::{sema, types::Kind};
#[path = "needs_drop_provider/linked.rs"]
mod linked;
const POSITIVE: &[(&str, &str, u8)] = &[
    ("primitive", "minimal-primitive", 0),
    ("drop", "minimal-drop", 1),
    ("recursive", "minimal-recursive-drop", 1),
    ("zero", "minimal-zero-drop-array", 0),
    ("reference", "minimal-reference", 0),
    ("generic", "minimal-generic-drop", 1),
    ("copy", "minimal-copy-bound", 0),
    ("ordinary-drop", "ordinary-drop-trait", 0),
    ("ordinary-function", "ordinary-needs-drop-function", 1),
    ("fallback-true", "intrinsic-fallback-true", 1),
    ("fallback-false", "intrinsic-fallback-false", 1),
];
const SCALING: &[(&str, &str, u8)] = &[
    ("width8-calls8", "work/width8-calls8", 1),
    ("width8-calls32", "work/width8-calls32", 1),
    ("width8-calls128", "work/width8-calls128", 1),
    ("width32-calls8", "work/width32-calls8", 1),
    ("width32-calls32", "work/width32-calls32", 1),
    ("width32-calls128", "work/width32-calls128", 1),
    ("width128-calls8", "work/width128-calls8", 1),
    ("width128-calls32", "work/width128-calls32", 1),
    ("width128-calls128", "work/width128-calls128", 1),
    ("width8-calls8-noise64", "work/width8-calls8-noise64", 1),
    ("width8-calls8-noise1024", "work/width8-calls8-noise1024", 1),
    (
        "ordinary-unused-intrinsic",
        "work/ordinary-unused-intrinsic",
        0,
    ),
    ("primitive-calls128", "work/primitive-calls128", 0),
];
const NEGATIVE: &[(&str, &str)] = &[
    ("unpaid-copy", "minimal-unpaid-copy"),
    ("missing-drop", "minimal-missing-drop-method"),
    ("wrong-receiver", "minimal-wrong-drop-receiver"),
    ("copy-conflict", "drop-copy-conflict"),
];
fn read_source(name: &str) -> TestResult<String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa31/tests/spec/needs-drop/{name}.rs"));
    Ok(std::fs::read_to_string(path)?)
}
#[test]
fn actual_needs_drop_selects_concrete_and_wrapper_instances() -> TestResult {
    let mut failures = Vec::new();
    for &(name, path, _) in POSITIVE {
        let source = read_source(path)?;
        match lower(&source) {
            Ok(ir) => {
                let mut output = Vec::new();
                rustgm::lowir::writer::write(&ir, &mut output)?;
                assert!(!output.is_empty(), "{name}");
                println!("{name}: accepted");
            }
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn actual_needs_drop_keeps_caller_and_destructor_contracts() -> TestResult {
    for &(name, path) in NEGATIVE {
        let source = read_source(path)?;
        assert!(lower(&source).is_err(), "{name}");
    }
    Ok(())
}
#[test]
fn actual_drop_registration_is_an_ordinary_trait_implementation() -> TestResult {
    let source = read_source(POSITIVE[1].1)?;
    let report = report(&source)?;
    let facts = sema::declarations::analyze(&report)?;
    assert_eq!(facts.destructors.implementations().len(), 1);
    let implementation = &facts.destructors.implementations()[0];
    assert!(matches!(
        facts.types.kind(implementation.target),
        Some(Kind::Adt(..))
    ));
    let role = report
        .resolution
        .source_bindings
        .as_deref()
        .ok_or("roles")?
        .language(rustgm::support::bindings::LangItem::Drop)
        .ok_or("Drop")?;
    assert_ne!(Some(role.definition), report.resolution.drop_trait);
    Ok(())
}
