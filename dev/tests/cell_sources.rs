//! Genuine `UnsafeCell` storage policy keeps source trait and ownership proofs.
use crate::provider_traits::{TestResult, lower, report};
use rustgm::{
    sema,
    support::bindings::LangItem,
    types::{ItemId, aggregate},
};
use std::path::{Path, PathBuf};

fn root() -> TestResult<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?
        .to_owned())
}
fn fixture(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(root()?.join(format!(
        "pa31/tests/spec/unsafe-cell-source/{name}"
    )))?)
}

#[test]
fn actual_cell_outcomes_preserve_source_traits_and_ordinary_borrows() -> TestResult {
    let mut count = 0;
    let mut failures = Vec::new();
    for line in fixture("cases.tsv")?.lines() {
        let (name, status) = line.split_once('\t').ok_or("two case columns")?;
        let accepted = match status {
            "0" => true,
            "1" => false,
            _ => return Err("invalid semantic prediction".into()),
        };
        let outcome = lower(&fixture(name)?);
        if accepted != outcome.is_ok() {
            failures.push(format!("{name}: {:?}", outcome.err()));
        } else if let Err(error) = outcome {
            let message = error.to_string();
            assert!(
                !message.contains("semantic contract is not implemented"),
                "{name}: {message}"
            );
        }
        count += 1;
    }
    assert_eq!(count, 15);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn cell_policy_uses_the_actual_nominal_and_preserves_field_destruction() -> TestResult {
    let source = report(&fixture("nominal.rs")?)?;
    let facts = sema::declarations::analyze(&source)?;
    let actual = source
        .resolution
        .source_bindings
        .as_deref()
        .ok_or("source roles")?
        .language(LangItem::UnsafeCell)
        .ok_or("actual UnsafeCell declaration")?;
    let item = ItemId(actual.definition.0);
    assert_eq!(
        facts.aggregates.shared_mutation(item),
        Some(aggregate::SharedMutation::Interior)
    );
    assert_eq!(
        facts.aggregates.destruction(item),
        Some(aggregate::FieldDestruction::Fields)
    );
    let shape = facts.aggregates.get(item).ok_or("actual fields")?;
    assert_eq!(shape.kind, aggregate::Kind::Struct);
    assert_eq!(facts.aggregates.variants(shape).len(), 1);
    assert_eq!(size_of::<rustgm::types::Kind>(), 24);
    assert_eq!(size_of::<rustgm::body::ir::Body>(), 456);
    let ordinary = report(&fixture("same-name.rs")?)?;
    let ordinary = sema::declarations::analyze(&ordinary)?;
    assert!(!ordinary.aggregates.has_shared_mutation());
    Ok(())
}

#[test]
fn shared_cell_globals_are_writable_and_plain_globals_remain_readonly() -> TestResult {
    let program = lower(&fixture("runtime.rs")?)?;
    assert!(
        program
            .globals
            .iter()
            .any(|global| global.span.bytes == 4 && !global.readonly)
    );
    assert!(
        program
            .globals
            .iter()
            .any(|global| global.span.bytes == 4 && global.readonly)
    );
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn actual_cell_native_preserves_aliasing_niches_and_ordinary_cleanup() -> TestResult {
    let root = root()?;
    let folder = root
        .join("obj/parallel/unsafe-cell-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (name, observer, expected) in [
        ("runtime", "runtime-observer.c", "23 23\n"),
        ("layout", "layout-observer.c", "8 16 8\n"),
        ("field-drop", "drop-observer.c", "drop=31\n"),
        ("nullable-shapes", "nullable-observer.c", "23 0 23 0 16\n"),
    ] {
        crate::ffi_program::check(
            &root,
            &folder,
            name,
            &lower(&fixture(&format!("{name}.rs"))?)?,
            &fixture(observer)?,
            expected,
        )?;
    }
    Ok(())
}
