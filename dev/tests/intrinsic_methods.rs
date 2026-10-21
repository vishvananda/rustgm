//! Canonical atomics retain independent source lint, constant and runtime contracts.
#[path = "harness/native_references.rs"]
mod native;
#[path = "support/semantic.rs"]
mod semantic;
#[path = "helpers/source_cases.rs"]
mod source;
const CASES: &[(&str, bool)] = &[
    ("load-store", true),
    ("variable-order", true),
    ("ufcs-order", true),
    ("reified-order", true),
    ("const-alias-order", true),
    ("block-order", true),
    ("direct-release-load", false),
    ("direct-acqrel-load", false),
    ("direct-acquire-store", false),
    ("direct-acqrel-store", false),
    ("grouped-invalid-order", false),
    ("imported-invalid-order", false),
    ("allow-invalid-order", true),
    ("warn-invalid-order", true),
    ("forbid-weaken", false),
    ("closure-inherits", false),
    ("nested-allow", true),
    ("known-only-match", false),
    ("wildcard-match", true),
    ("known-only-let", false),
    ("nested-known-match", false),
    ("nested-wildcard-match", true),
    ("unknown-not-unreachable", true),
    ("wildcard-makes-unreachable", false),
    ("packed-receiver", false),
    ("packed-raw", true),
    ("const-load", false),
    ("const-store-skipped", false),
    ("const-reified", false),
    ("ordering-copy", true),
    ("ordering-constant-tags", true),
    ("wrong-order-type", false),
    ("wrong-store-type", false),
    ("shadowed-method", true),
];
#[test]
fn ordering_and_atomic_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa28/tests/spec/atomic-methods/{name}.rs"));
        let text = std::fs::read_to_string(path)?;
        source::check("parallel/pa28/atomic-methods", &[(name, accepted, &text)])?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn atomic_methods_execute_with_runtime_order_validation() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, panic) in &[
        ("ordering-tags", false),
        ("legal-orders", false),
        ("variable-orders", false),
        ("reified-methods", false),
        ("nested-shared-static", false),
        ("local-roundtrip", false),
        ("dynamic-release-load", true),
        ("dynamic-acqrel-load", true),
        ("dynamic-acquire-store", true),
        ("dynamic-acqrel-store", true),
        ("ufcs-invalid", true),
        ("reified-invalid", true),
        ("const-alias-invalid", true),
        ("allowed-invalid", true),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa28/tests/spec/atomic-method-native/{name}.rs"));
        let text = std::fs::read_to_string(path)?;
        native::check_cases(
            "parallel/pa28/atomic-method-native",
            &[(name, &text, panic)],
        )?;
    }
    Ok(())
}

#[test]
fn rejected_operations_keep_their_specific_semantic_reason()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, reason) in [
        (
            "direct-release-load",
            "invalid atomic ordering for this operation",
        ),
        (
            "direct-acquire-store",
            "invalid atomic ordering for this operation",
        ),
        ("known-only-match", "non-exhaustive match"),
        ("packed-receiver", "reference to an unaligned packed field"),
        (
            "const-load",
            "runtime-only intrinsic is not allowed during constant evaluation",
        ),
        (
            "const-store-skipped",
            "runtime-only intrinsic is not allowed during constant evaluation",
        ),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa28/tests/spec/atomic-methods/{name}.rs"));
        let text = std::fs::read_to_string(path)?;
        let Err(error) = lowered(&format!("{text}fn main(){{}}")) else {
            return Err(format!("{name}: invalid program accepted").into());
        };
        assert!(error.contains(reason), "{name}: {error}");
    }
    Ok(())
}

fn lowered(text: &str) -> Result<rustgm::lowir::program::Program, String> {
    let (mut report, mut facts) = semantic::analyze(text)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry).map_err(|e| e.to_string())
}

#[test]
fn canonical_methods_and_runtime_shims_have_bounded_growth()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{lowir::program::Operation, support::intrinsics::Intrinsic};
    use std::fmt::Write;
    for count in [8, 32, 128] {
        let mut text = String::from(
            "use core::sync::atomic::{AtomicU32,Ordering};static A:AtomicU32=AtomicU32::new(0);",
        );
        for index in 0..count {
            write!(
                text,
                "fn f{index}(){{A.store({index},Ordering::Relaxed);let _=A.load(Ordering::Acquire);}}"
            )?;
        }
        text.push_str("fn main(){");
        for index in 0..count {
            write!(text, "f{index}();")?;
        }
        text.push('}');
        source::qualify(
            &format!("parallel/pa28/atomic-method-scaling/{count}"),
            &text,
            true,
        )?;
        let (mut report, mut facts) = semantic::analyze(&text)?;
        assert_eq!(facts.intrinsics.iter().count(), 6);
        for kind in [Intrinsic::AtomicLoad, Intrinsic::AtomicStore] {
            let method = facts.intrinsics.get(kind).ok_or("missing atomic method")?;
            assert!(method.parameter.is_none());
            assert!(method.owner.is_some());
            assert!(facts.associated.trait_owner(method.definition).is_none());
            assert!(facts.associated.schema(method.definition).is_some());
            let template = facts
                .lifetimes
                .callables
                .get(rustgm::types::ItemId(method.definition.0))
                .ok_or("missing completed callable")?;
            assert_eq!(template.parameters, 0);
        }
        assert!(facts.body_work.steps < 1_024 * (count + 1));
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        let program = rustgm::lower::instances::build(&report, &mut facts, entry)?;
        let mut loads = 0;
        let mut stores = 0;
        for instruction in program
            .symbols
            .iter()
            .filter_map(|symbol| symbol.body.as_ref())
            .flat_map(|body| &body.instructions)
        {
            match instruction.operation {
                Operation::AtomicLoad { .. } => loads += 1,
                Operation::AtomicStore { .. } => stores += 1,
                _ => {}
            }
        }
        assert_eq!(
            (loads, stores),
            (3, 3),
            "{count}: runtime ordering branches must share two shims"
        );
    }
    Ok(())
}

#[test]
fn macro_assertion_remains_an_explicit_separate_boundary() -> Result<(), Box<dyn std::error::Error>>
{
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa28/tests/spec/atomic-methods/ordering-tags.rs"),
    )?;
    source::qualify("parallel/pa28/atomic-macro-boundary", &text, true)?;
    let Err(error) = lowered(&format!("{text}fn main(){{}}")) else {
        return Err("assertion macro boundary unexpectedly accepted".into());
    };
    assert!(
        error.contains("unresolved macro is not in textual scope or path scope"),
        "{error}"
    );
    Ok(())
}
#[test]
fn unreachable_arm_policy_uses_its_owning_source_lanes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, path) in &[
        (
            "unreachable-default",
            true,
            "pa13/tests/spec/pattern-policies/unreachable-default.rs",
        ),
        (
            "unreachable-deny",
            false,
            "pa13/tests/spec/pattern-policies/unreachable-deny.rs",
        ),
        (
            "unreachable-inner-allow",
            true,
            "pa13/tests/spec/pattern-policies/unreachable-inner-allow.rs",
        ),
        (
            "unreachable-inner-warn",
            true,
            "pa13/tests/spec/pattern-policies/unreachable-inner-warn.rs",
        ),
        (
            "unreachable-forbid-weaken",
            false,
            "pa13/tests/spec/pattern-policies/unreachable-forbid-weaken.rs",
        ),
        (
            "unreachable-closure",
            false,
            "pa24/tests/spec/pattern-policies/unreachable-closure.rs",
        ),
        (
            "unreachable-nested-item",
            false,
            "pa13/tests/spec/pattern-policies/unreachable-nested-item.rs",
        ),
    ] {
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join(path),
        )?;
        source::check(
            "parallel/pa28/shared-pattern-policy",
            &[(name, accepted, &text)],
        )?;
    }
    Ok(())
}
