//! Scalar union addresses alias the current Image while copied values retain snapshots.
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
    ("read-wide", 0, 0, ""),
    ("read-small", 0, 0, ""),
    ("cross-field-small-write", 0, 0, ""),
    ("cross-field-wide-write", 0, 0, ""),
    ("write-initializes-wide", 0, 0, ""),
    (
        "wide-read-uninitialized",
        1,
        1,
        "constant union scalar read uses uninitialized bytes",
    ),
    (
        "small-write-still-uninitialized",
        1,
        1,
        "constant union scalar read uses uninitialized bytes",
    ),
    ("bool-valid", 0, 0, ""),
    (
        "bool-invalid",
        1,
        1,
        "constant union scalar read has invalid bool representation",
    ),
    ("bool-address-no-read", 0, 0, ""),
    ("char-valid", 0, 0, ""),
    (
        "char-invalid",
        1,
        1,
        "constant union scalar read has invalid char representation",
    ),
    ("static-projected", 0, 0, ""),
    (
        "mutable-static",
        1,
        1,
        "mutable static access requires an unsafe context",
    ),
    ("packed-base-store", 0, 0, ""),
    (
        "packed-field-store",
        1,
        1,
        "constant raw pointer write is unaligned",
    ),
    (
        "packed-field-read",
        1,
        1,
        "constant raw pointer read is unaligned",
    ),
    (
        "shared-derived",
        1,
        1,
        "constant raw pointer write is derived from a shared reference",
    ),
    ("raw-const-local-cast", 0, 0, ""),
    (
        "expired",
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
    (
        "cast-reinterpret-boundary",
        0,
        1,
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    (
        "reference-reconstruction-boundary",
        0,
        1,
        "references into constant union images are not implemented",
    ),
    (
        "pointer-family-boundary",
        0,
        1,
        "constant union images require an all-fields primitive scalar family; pointer and aggregate provenance storage is not implemented",
    ),
    (
        "mutable-static-unsafe",
        1,
        1,
        "constant execution cannot read mutable static storage",
    ),
    ("nested-product", 0, 0, ""),
    ("copy-snapshot", 0, 0, ""),
    (
        "cross-field-bool-invalid",
        1,
        1,
        "constant union scalar read has invalid bool representation",
    ),
    ("cross-field-bool-valid", 0, 0, ""),
    ("generic-address", 0, 0, ""),
    (
        "generic-shared-write",
        1,
        1,
        "derived from a shared reference",
    ),
    ("mutable-static-address-only", 0, 0, ""),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn scalar_union_addresses_match_independent_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let evidence = root.join(format!(
        "obj/parallel/pa27/union-address-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, frontend, reason) in CASES {
        let source = root.join(format!("pa27/tests/spec/union-addresses/{name}.rs"));
        checked::check(&source, &evidence, name, (rust, frontend, reason))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn aliased_union_images_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "cross-fields",
        "initialized-scalars",
        "static-views",
        "snapshots-nested",
        "packed",
        "generic",
    ] {
        let source = std::fs::read_to_string(
            root()?.join(format!("pa27/tests/spec/union-address-native/{name}.rs")),
        )?;
        native::check_cases(
            "parallel/pa27/union-address-native",
            &[(name, &source, false)],
        )?;
    }
    Ok(())
}
#[test]
fn repeated_union_address_updates_have_linear_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut previous = None;
    let mut peak = None;
    for count in [8, 32, 128] {
        let text = format!(
            "union U{{a:u32,b:u8}}const fn f(n:u32)->u32{{let mut u=U{{a:0}};let a=&raw mut u.a;let b=&raw const u.b;let mut i=0;while i<n{{unsafe{{*a=*a+1;let _=*b;}}i+=1;}}unsafe{{u.a}}}}const X:u32=f({count});fn main(){{let _=X;}}"
        );
        rustc::qualify(
            &format!("parallel/pa27/union-address-scaling-{count}"),
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
            "n={count} steps={} slots={} expressions={}",
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
    Ok(())
}
