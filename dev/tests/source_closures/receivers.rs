//! Typed receiver use determines capture strength independently of storage mode.
use super::semantic;
use rustgm::{sema::closures::captures::Mode, types::closure::CallKind};

const CASES: &[(&str, Mode, CallKind, usize)] = &[
    ("method-shared", Mode::Shared, CallKind::Shared, 1),
    ("method-mutable", Mode::Mutable, CallKind::Mutable, 1),
    ("method-once", Mode::Value, CallKind::Once, 1),
    ("method-move-shared", Mode::Value, CallKind::Shared, 1),
    ("method-precise-field", Mode::Shared, CallKind::Shared, 1),
    ("array-shared", Mode::Shared, CallKind::Shared, 1),
    ("array-mutable", Mode::Mutable, CallKind::Mutable, 1),
    ("array-index-captured", Mode::Shared, CallKind::Shared, 2),
    ("array-move-shared", Mode::Value, CallKind::Shared, 1),
    ("array-explicit-borrow", Mode::Shared, CallKind::Shared, 1),
    ("array-field-copy", Mode::Shared, CallKind::Shared, 1),
    ("array-field-mutable", Mode::Mutable, CallKind::Mutable, 1),
    ("array-nested-shared", Mode::Shared, CallKind::Shared, 1),
    ("array-nested-mutable", Mode::Mutable, CallKind::Mutable, 1),
    ("method-indexed-element", Mode::Shared, CallKind::Shared, 1),
    (
        "local-projection-unresolved",
        Mode::Shared,
        CallKind::Shared,
        1,
    ),
    ("direct-function", Mode::Shared, CallKind::Shared, 1),
    ("direct-constructor", Mode::Shared, CallKind::Shared, 1),
    ("direct-function-pointer", Mode::Shared, CallKind::Shared, 2),
];

#[test]
fn method_and_array_uses_retain_typed_capture_permissions() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, mode, kind, count) in CASES {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa24/tests/spec/typed-capture-uses/{name}.rs"));
        let source = std::fs::read_to_string(path)?;
        let (_, facts) = semantic::analyze(&source).map_err(|error| format!("{name}: {error}"))?;
        assert_eq!(facts.closures.all().len(), 1, "{name}");
        let body = &facts.closures.all()[0];
        let environment = facts
            .closures
            .environment(body.identity)
            .ok_or("environment")?;
        assert_eq!(environment.plan.captures().len(), count, "{name}");
        assert_eq!(
            facts.types.closure_kind(body.identity),
            Some(kind),
            "{name}"
        );
        for capture in environment.plan.captures() {
            assert_eq!(capture.mode, mode, "{name}");
            if name.starts_with("array-") {
                assert!(environment.plan.parent(capture.path).is_none(), "{name}");
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn method_and_index_captures_execute_their_retained_permissions()
-> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "methods",
        "indices",
        "noncopy-field",
        "moving",
        "leaf-local",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa24/tests/spec/typed-capture-native/{name}.rs"));
        let source = std::fs::read_to_string(path)?;
        super::native::check_cases("parallel/capture-uses/native", &[(name, &source, false)])?;
    }
    Ok(())
}
