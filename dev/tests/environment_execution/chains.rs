//! Every borrowed callable receiver level retains its physical pointer and permission.
use super::{checked_sources, native, semantic};
use std::{fmt::Write, path::PathBuf};
const CASES: &[(&str, bool)] = &[
    ("shared-shared-shared", true),
    ("shared-shared-mutable", true),
    ("shared-shared-once", true),
    ("mutable-shared-mutable", true),
    ("mutable-shared-once", true),
    ("mutable-mutable-mutable", true),
    ("mutable-mutable-once", true),
    ("three-shared", true),
    ("three-mixed", true),
    ("shared-mutable-shared-reject", false),
    ("shared-mutable-once-reject", false),
    ("mutable-shared-shared-reject", false),
    ("shared-chain-mutation-reject", false),
    ("once-leaf-reject", false),
    ("external-result", true),
    ("generic-owner", true),
    ("static-escape-reject", false),
    ("owned-result-escape-reject", false),
    ("borrow-conflict-reject", false),
    ("nested-binder", true),
    ("owned-drop", true),
];
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/nested-receivers/{name}.rs"))
}
#[test]
fn declared_nested_receiver_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/nested-receivers/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, accepted) in CASES {
        let status = i32::from(!accepted);
        checked_sources::check_views(
            &fixture(name),
            &evidence,
            name,
            status,
            &[("--emit-lowir", status)],
            "",
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn nested_receiver_dereferences_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        if accepted {
            let source = std::fs::read_to_string(fixture(name))?;
            native::check_cases(
                "parallel/pa24/nested-receivers/native",
                &[(
                    name,
                    source
                        .strip_prefix(native::CHECK)
                        .ok_or("missing check prefix")?,
                    false,
                )],
            )?;
        }
    }
    Ok(())
}
#[test]
fn nested_receiver_depth_keeps_one_exact_wrapper() -> Result<(), Box<dyn std::error::Error>> {
    let mut observations = Vec::new();
    for depth in [8, 16, 32] {
        let mut source = String::from(
            "fn once<F:FnOnce()->u8>(f:F)->u8{f()}fn main(){let n=7u8;let f=move||n;let r0=&f;",
        );
        for index in 1..depth {
            write!(source, "let r{index}=&r{};", index - 1)?;
        }
        write!(source, "let _=once(r{});}}", depth - 1)?;
        super::rustc::qualify(
            &format!("parallel/pa24/nested-receivers/scaling/{depth}"),
            &source,
            true,
        )?;
        let (mut report, mut facts) = semantic::analyze(&source)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        let output = rustgm::lower::instances::build(&report, &mut facts, entry)?;
        let count = output
            .symbols
            .iter()
            .filter(|symbol| {
                symbol.name.starts_with("rg.closure.adapter.") && symbol.body.is_some()
            })
            .count();
        assert_eq!(count, 1);
        let work = facts.body_work.canonical_steps;
        eprintln!(
            "nested receiver depth={depth}: canonical={work}, typed={}, selections={}",
            facts.body_work.steps, facts.body_work.trait_selections
        );
        observations.push(work);
    }
    assert!(
        observations[2] - observations[1] <= 3 * (observations[1] - observations[0]),
        "{observations:?}"
    );
    Ok(())
}
