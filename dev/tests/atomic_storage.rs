//! Canonical `AtomicU32` storage composes with ordinary ownership and constant validity.
use crate::native_references as native;
use crate::semantic;
use crate::source_cases as source;
const CASES: &[(&str, bool, &str)] = &[
    ("new", true, "spec/atomic-storage/new.rs"),
    ("alias-new", true, "spec/atomic-storage/alias-new.rs"),
    ("no-prelude", false, "spec/atomic-storage/no-prelude.rs"),
    (
        "no-type-parameter",
        false,
        "spec/atomic-storage/no-type-parameter.rs",
    ),
    ("wrong-input", false, "spec/atomic-storage/wrong-input.rs"),
    (
        "private-field",
        false,
        "spec/atomic-storage/private-field.rs",
    ),
    ("private-rest", true, "spec/atomic-storage/private-rest.rs"),
    (
        "zero-array-not-copy",
        false,
        "spec/atomic-storage/zero-array-not-copy.rs",
    ),
    (
        "atomic-markers",
        true,
        "spec/atomic-storage/atomic-markers.rs",
    ),
    (
        "atomic-not-copy",
        false,
        "spec/atomic-storage/atomic-not-copy.rs",
    ),
    (
        "atomic-not-clone",
        false,
        "spec/atomic-storage/atomic-not-clone.rs",
    ),
    ("const-new", true, "spec/atomic-storage/const-new.rs"),
    (
        "const-reference-static",
        true,
        "spec/atomic-storage/const-reference-static.rs",
    ),
    (
        "const-reference-const",
        false,
        "spec/atomic-storage/const-reference-const.rs",
    ),
    (
        "packed-atomic",
        true,
        "spec/atomic-storage/packed-atomic.rs",
    ),
    (
        "zero-array-return",
        true,
        "spec/atomic-storage/zero-array-return.rs",
    ),
    (
        "inactive-option-return",
        true,
        "spec/atomic-storage/inactive-option-return.rs",
    ),
    (
        "active-option-return",
        false,
        "spec/atomic-storage/active-option-return.rs",
    ),
    (
        "const-temporary-reference",
        false,
        "spec/atomic-storage/const-temporary-reference.rs",
    ),
    (
        "static-temporary-reference",
        false,
        "spec/atomic-storage/static-temporary-reference.rs",
    ),
    (
        "const-fn-local",
        true,
        "spec/atomic-storage/const-fn-local.rs",
    ),
    (
        "const-unit-local",
        true,
        "spec/atomic-storage/const-unit-local.rs",
    ),
    (
        "const-field-reference",
        true,
        "spec/atomic-storage/const-field-reference.rs",
    ),
    (
        "temporary-field-reference",
        true,
        "spec/atomic-storage/temporary-field-reference.rs",
    ),
    (
        "runtime-named-field-reference",
        false,
        "spec/atomic-storage/runtime-named-field-reference.rs",
    ),
    (
        "runtime-temporary-field-reference",
        false,
        "spec/atomic-storage/runtime-temporary-field-reference.rs",
    ),
];
#[test]
fn atomic_storage_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, path) in CASES {
        let text = fixture(path)?;
        source::check("parallel/pa28/atomic-storage", &[(name, accepted, &text)])?;
    }
    Ok(())
}
#[test]
fn inline_constant_repeat_preserves_empty_atomic_storage() -> Result<(), Box<dyn std::error::Error>>
{
    let text = fixture("spec/atomic-storage/zero-array-repeat-return.rs")?;
    source::qualify("parallel/pa28/atomic-storage-inline-const", &text, true)?;
    lowered(&format!("{text}fn main(){{let _=f();}}"))?;
    Ok(())
}
fn lowered(text: &str) -> Result<rustgm::lowir::program::Program, String> {
    let (mut report, mut facts) = semantic::analyze(text)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry).map_err(|e| e.to_string())
}

#[test]
fn writable_allocations_and_readonly_reference_slots_are_distinct()
-> Result<(), Box<dyn std::error::Error>> {
    let text = "use core::sync::atomic::AtomicU32 as Atomic;struct AtomicU32(u32);static OWNED:Atomic=Atomic::new(0);static SLOT:&Atomic=&OWNED;static ORDINARY:AtomicU32=AtomicU32(7);fn main(){let _=SLOT;let _=unsafe{core::ptr::read_volatile(&raw const ORDINARY.0)};}";
    let program = lowered(text)?;
    assert!(
        program
            .globals
            .iter()
            .any(|g| g.span.bytes == 4 && !g.readonly)
    );
    assert!(
        program
            .globals
            .iter()
            .any(|g| g.span.bytes == 8 && g.readonly)
    );
    assert!(
        program
            .globals
            .iter()
            .any(|g| g.span.bytes == 4 && g.readonly)
    );
    Ok(())
}

#[test]
fn canonical_factory_identity_and_body_work_remain_additive()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{body::ir::Rvalue, support::library_types::LibraryType};
    use std::{collections::HashSet, fmt::Write};
    for callers in [8, 32, 128] {
        let mut text = String::from("use core::sync::atomic::AtomicU32;");
        for caller in 0..callers {
            write!(
                text,
                "fn f{caller}()->AtomicU32{{AtomicU32::new({caller})}}"
            )?;
        }
        source::qualify(
            &format!("parallel/pa28/atomic-storage-scaling/{callers}"),
            &text,
            true,
        )?;
        let (report, facts) = semantic::analyze(&text)?;
        let nominal = report
            .resolution
            .library_types
            .get(LibraryType::AtomicU32)
            .ok_or("missing canonical atomic type")?;
        assert!(nominal.parameters.is_empty());
        assert_eq!(nominal.variants.len(), 1);
        let factory = nominal.variants[0].constructor;
        assert_eq!(
            facts.members.constructors.get(&factory).map(|c| c.kind),
            Some(rustgm::sema::members::ConstructorKind::ConstFunction)
        );
        assert_eq!(
            facts
                .aggregates
                .shared_mutation(rustgm::types::ItemId(nominal.definition.0)),
            Some(rustgm::types::aggregate::SharedMutation::Interior)
        );
        assert!(facts.body_work.steps < 256 * (callers + 1));
        let program = rustgm::body::build(&report, &facts)?;
        let mut instances = HashSet::new();
        let mut operations = 0;
        for body in &program.bodies {
            for value in &body.values {
                if let Rvalue::Call { callee, .. } = value.kind {
                    let ty = body.values[callee.0 as usize].ty;
                    assert!(
                        matches!(facts.types.kind(ty), Some(rustgm::types::Kind::FunctionItem { item, .. }) if item.0 == factory.0)
                    );
                    instances.insert(ty);
                    operations += 1;
                }
            }
        }
        assert_eq!(operations, callers);
        assert_eq!(instances.len(), 1);
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn canonical_atomic_storage_executes() -> Result<(), Box<dyn std::error::Error>> {
    let empty = fixture("spec/atomic-storage/zero-array-repeat-return.rs")?;
    native::check_cases(
        "parallel/pa28/atomic-storage-inline",
        &[(
            "zero-repeat",
            &format!("{empty}fn main(){{let _=f();}}"),
            false,
        )],
    )?;
    for &(name, path, panic) in &[
        (
            "shared-static",
            "spec/atomic-storage-native/shared-static.rs",
            false,
        ),
        (
            "nested-static",
            "spec/atomic-storage-native/nested-static.rs",
            false,
        ),
        (
            "readonly-slot",
            "spec/atomic-storage-native/readonly-slot.rs",
            false,
        ),
        (
            "projected-constant",
            "spec/atomic-storage-native/projected-constant.rs",
            false,
        ),
        (
            "transient-const",
            "spec/atomic-storage-native/transient-const.rs",
            false,
        ),
    ] {
        let text = fixture(path)?;
        native::check_cases(
            "parallel/pa28/atomic-storage-native",
            &[(name, &text, panic)],
        )?;
    }
    Ok(())
}

fn fixture(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa28/tests")
            .join(path),
    )
}
