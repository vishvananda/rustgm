//! Source capture facts are separate from the later executable and borrow gates.
use super::{rustc, semantic};
use rustgm::{sema::closures::captures::Mode, types::closure::CallKind};

const CASES: &[(&str, &str, Mode, CallKind)] = &[
    ("shared", "shared-capture", Mode::Shared, CallKind::Shared),
    (
        "mutable",
        "mutable-capture",
        Mode::Mutable,
        CallKind::Mutable,
    ),
    (
        "move-shared",
        "move-still-fn",
        Mode::Value,
        CallKind::Shared,
    ),
    ("once", "move-once", Mode::Value, CallKind::Once),
    (
        "precise-borrow",
        "precise-field-borrow",
        Mode::Shared,
        CallKind::Shared,
    ),
    (
        "precise-move",
        "precise-field-move",
        Mode::Value,
        CallKind::Once,
    ),
    (
        "unique",
        "unique-immutable",
        Mode::UniqueImmutable,
        CallKind::Mutable,
    ),
];

fn fixture(stem: &str) -> Result<String, String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/captures/{stem}.rs"));
    std::fs::read_to_string(path).map_err(|error| error.to_string())
}

#[test]
fn source_capture_modes_retain_their_separate_call_permissions() -> Result<(), String> {
    for &(name, stem, mode, kind) in CASES {
        let source = fixture(stem)?;
        let (_, facts) = semantic::analyze(&source).map_err(|error| format!("{name}: {error}"))?;
        assert_eq!(facts.closures.all().len(), 1, "{name}");
        let body = &facts.closures.all()[0];
        let environment = facts
            .closures
            .environment(body.identity)
            .ok_or("missing captured environment")?;
        assert_eq!(environment.plan.captures().len(), 1, "{name}");
        assert_eq!(environment.plan.captures()[0].mode, mode, "{name}");
        assert_eq!(
            facts.types.closure_kind(body.identity),
            Some(kind),
            "{name}"
        );
        assert_eq!(environment.path_types.len(), environment.plan.path_count());
        assert_eq!(
            environment.formation_types.len(),
            environment.path_types.len()
        );
        assert_eq!(
            body.execution.ok_or("missing capture execution")?.receiver,
            kind
        );
    }
    Ok(())
}

#[test]
fn capture_plan_inputs_have_independent_source_acceptance() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, stem, ..) in CASES {
        rustc::qualify(&format!("capture-plan-{name}"), &fixture(stem)?, true)?;
    }
    Ok(())
}

const MENTION: &str = "fn main(){let x=7u8;let f=||{let _=x;};f();}";
#[test]
fn wildcard_mentions_require_no_environment_storage() -> Result<(), Box<dyn std::error::Error>> {
    rustc::qualify("capture-plan-wildcard", MENTION, true)?;
    let (_, facts) = semantic::analyze(MENTION)?;
    let body = &facts.closures.all()[0];
    let environment = facts
        .closures
        .environment(body.identity)
        .ok_or("missing wildcard facts")?;
    assert!(environment.plan.captures().is_empty());
    assert!(environment.field_types.is_empty());
    assert_eq!(environment.mentions.len(), 1);
    assert!(body.execution.is_none());
    Ok(())
}
