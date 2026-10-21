//! Typed receiver use determines capture strength independently of storage mode.
use super::semantic;
use rustgm::{
    sema::closures::{
        AccessKind,
        captures::{Mode, Step},
    },
    types::closure::CallKind,
};

const ADJUSTED_CASES: &[(&str, AccessKind, u32, Mode, CallKind)] = &[
    (
        "method-shared",
        AccessKind::Method,
        1,
        Mode::Shared,
        CallKind::Shared,
    ),
    (
        "method-deep",
        AccessKind::Method,
        2,
        Mode::Shared,
        CallKind::Shared,
    ),
    (
        "method-mutable",
        AccessKind::Method,
        1,
        Mode::Mutable,
        CallKind::Mutable,
    ),
    (
        "method-moving",
        AccessKind::Method,
        1,
        Mode::Value,
        CallKind::Shared,
    ),
    (
        "method-field",
        AccessKind::Method,
        1,
        Mode::Shared,
        CallKind::Shared,
    ),
    (
        "call-function",
        AccessKind::Invocation,
        1,
        Mode::Shared,
        CallKind::Shared,
    ),
    (
        "call-closure",
        AccessKind::Invocation,
        1,
        Mode::Shared,
        CallKind::Shared,
    ),
    (
        "call-mutable",
        AccessKind::Invocation,
        1,
        Mode::Mutable,
        CallKind::Mutable,
    ),
];

#[test]
fn adjusted_receiver_paths_preserve_the_original_source_place()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, kind, depth, mode, call) in ADJUSTED_CASES {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa24/tests/spec/adjusted-places/{name}.rs"));
        let source = std::fs::read_to_string(path)?;
        let (report, facts) =
            semantic::analyze(&source).map_err(|error| format!("{name}: {error}"))?;
        let (body, environment) = facts
            .closures
            .all()
            .iter()
            .find_map(|body| {
                facts
                    .closures
                    .environment(body.identity)
                    .filter(|environment| !environment.adjusted_accesses.is_empty())
                    .map(|environment| (body, environment))
            })
            .ok_or("missing adjusted capture")?;
        assert_eq!(environment.adjusted_accesses.len(), 1, "{name}");
        let (&(site, actual_kind), access) = environment
            .adjusted_accesses
            .iter()
            .next()
            .ok_or("missing receiver access")?;
        assert_eq!((actual_kind, access.dereferences), (kind, depth), "{name}");
        assert_eq!(
            facts.types.closure_kind(body.identity),
            Some(call),
            "{name}"
        );
        let capture = environment
            .plan
            .covering(access.path)
            .ok_or("uncaptured receiver")?;
        assert_eq!(capture.mode, mode, "{name}");
        let operand = match kind {
            AccessKind::Method => {
                report
                    .resolution
                    .method_calls
                    .get(&site)
                    .ok_or("missing method")?
                    .receiver
            }
            AccessKind::Invocation => {
                let rustgm::syntax::ast::Kind::Call { callee, .. } = report
                    .package
                    .unit(site.file)
                    .configured
                    .ast
                    .node(site.node)
                    .kind
                else {
                    return Err("missing invocation".into());
                };
                callee
            }
        };
        let key = rustgm::resolve::unit::NodeKey {
            node: operand,
            ..site
        };
        let original = environment
            .accesses
            .get(&key)
            .ok_or("missing source place")?;
        let mut cursor = access.path;
        for _ in 0..depth {
            let (parent, step) = environment
                .plan
                .parent(cursor)
                .ok_or("missing adjusted prefix")?;
            assert!(matches!(step, Step::Dereference { .. }), "{name}");
            cursor = parent;
        }
        assert_eq!(cursor, *original, "{name}");
    }
    Ok(())
}

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
fn captured_receiver_binding_mutability_is_separate_from_its_referent()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, mutable, accepted) in [
        ("named-borrow", false, true),
        ("elided-borrow", false, true),
        ("owned-mut", true, true),
        ("explicit-mut", true, true),
        ("named-rebind", false, false),
        ("ref-mut-referent", false, true),
        ("ref-mut-rebind", false, false),
        ("ref-mut-unused-rebind", false, false),
        ("local-mut-rebind", true, true),
        ("local-immutable-rebind", false, false),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa24/tests/spec/receiver-binding/{name}.rs"));
        let source = std::fs::read_to_string(&path)?;
        super::rustc::qualify(&format!("capture-receiver-{name}"), &source, accepted)?;
        let (_, facts) = semantic::analyze(&source).map_err(|error| format!("{name}: {error}"))?;
        assert_eq!(facts.closures.all().len(), 1, "{name}");
        let body = &facts.closures.all()[0];
        let environment = facts
            .closures
            .environment(body.identity)
            .ok_or("environment")?;
        assert_eq!(environment.binding_mutability.len(), 1, "{name}");
        assert_eq!(
            environment.binding_mutability.values().copied().next(),
            Some(mutable),
            "{name}"
        );
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&path)
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(i32::from(!accepted)),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if !accepted {
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("cannot assign through an immutable captured binding"),
                "{name}"
            );
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn captured_reference_bindings_preserve_referent_and_replacement_permissions()
-> Result<(), Box<dyn std::error::Error>> {
    for name in ["ref-mut-referent", "local-mut-rebind"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa24/tests/spec/receiver-binding/{name}.rs"));
        let source = std::fs::read_to_string(path)?;
        let body = source
            .strip_prefix(super::native::CHECK)
            .ok_or("missing check helper")?;
        super::native::check_cases("parallel/ref-binding/native", &[(name, body, false)])?;
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
