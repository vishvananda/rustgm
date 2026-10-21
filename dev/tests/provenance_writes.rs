//! Exact-type writes preserve per-pointer authority and physical storage checks.
#[path = "harness/checked_sources.rs"]
mod checked;
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "helpers/resolve.rs"]
mod source;
use std::path::PathBuf;
const CASES: &[(&str, i32, i32, &str)] = &[
    ("local-raw-mut", 0, 0, ""),
    ("mutable-reference", 0, 0, ""),
    ("mut-to-const-to-mut", 0, 0, ""),
    ("raw-const-local-cast", 0, 0, ""),
    (
        "shared-reference",
        1,
        1,
        "constant raw pointer write is derived from a shared reference",
    ),
    (
        "shared-reference-reborrow",
        1,
        1,
        "constant raw pointer write is derived from a shared reference",
    ),
    (
        "shared-pointer-copy",
        1,
        1,
        "constant raw pointer write is derived from a shared reference",
    ),
    (
        "expired-local",
        1,
        1,
        "constant pointer refers to expired storage",
    ),
    (
        "null",
        1,
        1,
        "constant pointer has no allocation provenance",
    ),
    ("packed-aligned", 0, 0, ""),
    (
        "packed-unaligned",
        1,
        1,
        "constant raw pointer write is unaligned",
    ),
    ("packed-raw-base-field", 0, 0, ""),
    (
        "reinterpret-bool-invalid",
        1,
        1,
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    (
        "reinterpret-bool-valid",
        0,
        1,
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    (
        "reinterpret-width",
        0,
        1,
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    (
        "static-immutable",
        1,
        1,
        "constant execution cannot write static storage",
    ),
    (
        "const-pointer-write",
        1,
        1,
        "cannot assign through a shared reference",
    ),
    ("shared-runtime-default-lint", 1, 0, ""),
    ("shared-runtime-allowed-lint", 0, 0, ""),
    ("writable-before-shared", 0, 0, ""),
    (
        "shared-before-writable",
        1,
        1,
        "access conflicts with a live reference loan",
    ),
    (
        "shared-aggregate",
        1,
        1,
        "constant raw pointer write is derived from a shared reference",
    ),
    (
        "shared-call",
        1,
        1,
        "constant raw pointer write is derived from a shared reference",
    ),
    (
        "shared-field-address",
        1,
        1,
        "constant raw pointer write is derived from a shared reference",
    ),
    ("mutable-field-address", 0, 0, ""),
    ("writable-call", 0, 0, ""),
    ("writable-aggregate", 0, 0, ""),
    (
        "shared-interior",
        0,
        1,
        "constant shared-derived interior-mutable raw writes are not implemented",
    ),
    ("promoted-shared", 1, 1, "derived from a shared reference"),
    (
        "mutable-to-shared-coercion",
        1,
        1,
        "derived from a shared reference",
    ),
    ("array-bounds", 1, 1, "out of bounds"),
    ("static-mutable", 1, 1, "cannot write static storage"),
    ("union-base-boundary", 0, 0, ""),
    ("typed-scalars", 0, 0, ""),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn exact_raw_stores_match_declared_objects_and_boundaries() -> Result<(), Box<dyn std::error::Error>>
{
    let root = root()?;
    let evidence = root.join(format!(
        "obj/parallel/pa27/raw-store-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, frontend, reason) in CASES {
        let source = if name == "shared-interior" {
            root.join("pa28/tests/spec/atomic-storage/shared-derived-raw-store.rs")
        } else {
            root.join(format!("pa27/tests/spec/raw-store-validity/{name}.rs"))
        };
        checked::check(&source, &evidence, name, (rust, frontend, reason))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn exact_raw_store_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in ["values", "packed", "products"] {
        let source = std::fs::read_to_string(
            root()?.join(format!("pa27/tests/spec/raw-store-native/{name}.rs")),
        )?;
        native::check_cases("parallel/pa27/raw-store-native", &[(name, &source, false)])?;
    }
    Ok(())
}

#[test]
fn repeated_raw_stores_have_linear_work_and_fixed_retained_storage()
-> Result<(), Box<dyn std::error::Error>> {
    for raw in [false, true] {
        let mut previous = None;
        let mut peak = None;
        for count in [8, 32, 128] {
            let update = if raw { "unsafe{*p=*p+1;}" } else { "x=x+1;" };
            let pointer = if raw { "let p=&raw mut x;" } else { "" };
            let text = format!(
                "const fn f(n:u32)->u32{{let mut x=0u32;{pointer}let mut i=0;while i<n{{{update}i+=1;}}x}}const X:u32=f({count});fn main(){{let _=X;}}"
            );
            rustc::qualify(
                &format!("parallel/pa27/raw-store-scaling-{raw}-{count}"),
                &text,
                true,
            )?;
            let mut report = source::resolve(&text, true)?;
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = rustgm::pipeline::analyze_source(&report)?;
            let entry = rustgm::sema::entry::select(&mut report, &facts)?;
            rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
            let work = &facts.constant_work;
            println!(
                "raw={raw} n={count} steps={} slots={} expressions={}",
                work.steps, work.peak_slots, work.expressions
            );
            if let Some(previous) = previous {
                assert!(work.steps < previous * 5);
            }
            if let Some(peak) = peak {
                assert_eq!(work.peak_slots, peak);
            }
            assert!(work.steps >= count);
            previous = Some(work.steps);
            peak = Some(work.peak_slots);
        }
    }
    Ok(())
}
