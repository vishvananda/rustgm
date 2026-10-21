//! Physical reference-cell compatibility preserves semantic lifetimes and validity gates.
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
    ("expired-reference-cell", 0, 0, ""),
    ("live-reference-cell", 0, 0, ""),
    ("dangling-raw-field", 0, 0, ""),
    (
        "expired-reference-demanded",
        1,
        1,
        "constant pointer refers to expired storage",
    ),
    ("ordinary-reference-cell-read", 0, 0, ""),
    ("nested-shared-write", 0, 0, ""),
    ("original-mutable-after-sharing", 0, 0, ""),
    ("nested-shared-read", 0, 0, ""),
    (
        "different-inner-region",
        0,
        1,
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    (
        "different-referent",
        0,
        1,
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    (
        "different-mutability",
        0,
        1,
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    ("reference-cell-store", 0, 0, ""),
    (
        "dangling-raw-field-read",
        1,
        1,
        "constant pointer refers to expired storage",
    ),
    (
        "shared-reference-cell-store",
        1,
        1,
        "constant raw pointer write is derived from a shared reference",
    ),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn reference_cells_match_independent_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let evidence = root.join(format!(
        "obj/parallel/pa27/reference-cells-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, status, reason) in CASES {
        checked::check(
            &root.join(format!("pa27/tests/spec/raw-reference-cells/{name}.rs")),
            &evidence,
            name,
            (rust, status, reason),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn reference_cell_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "live-cells",
        "terminal-storage",
        "alias-rights",
        "static-cell",
    ] {
        let source = std::fs::read_to_string(
            root()?.join(format!("pa27/tests/spec/reference-cell-native/{name}.rs")),
        )?;
        native::check_cases(
            "parallel/pa27/reference-cells-native",
            &[(name, &source, false)],
        )?;
    }
    Ok(())
}
#[test]
fn reference_cell_compatibility_has_bounded_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut ordinary = std::collections::HashMap::new();
    let mut overhead = None;
    for raw in [false, true] {
        let mut previous = None;
        for count in [8, 32, 128] {
            let pointer = if raw {
                "let p=&raw const r;"
            } else {
                "let p=&r;"
            };
            let text = format!(
                "const fn f(n:u32)->u32{{let x=1u32;let r=&x;{pointer}let mut i=0u32;let mut total=0u32;while i<n{{total+=unsafe{{**p}};i+=1;}}total}}const X:u32=f({count});fn main(){{let _=X;}}"
            );
            rustc::qualify(
                &format!("parallel/pa27/reference-cells-scaling-{raw}-{count}"),
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
                "raw={raw} n={count} steps={} slots={}",
                work.steps, work.peak_slots
            );
            assert!(work.peak_slots <= count + 96);
            if let Some(old) = previous {
                assert!(work.steps < old * 5);
            }
            previous = Some(work.steps);
            if raw {
                let baseline = ordinary.get(&count).copied().ok_or("ordinary baseline")?;
                let delta = i64::try_from(work.peak_slots)? - baseline;
                if let Some(old) = overhead {
                    assert_eq!(delta, old);
                }
                overhead = Some(delta);
            } else {
                ordinary.insert(count, i64::try_from(work.peak_slots)?);
            }
        }
    }
    Ok(())
}
