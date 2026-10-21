//! Exact shared raw reborrows retain provenance, alignment and immutable authority.
#[path = "harness/checked_sources.rs"]
mod checked;
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "helpers/resolve.rs"]
mod source;
use std::path::PathBuf;
const CASES: &[(&str, i32, i32, i32, &str)] = &[
    ("local-scalar", 0, 0, 0, ""),
    ("product", 0, 0, 0, ""),
    ("array-field", 0, 0, 0, ""),
    ("static-reference", 0, 0, 0, ""),
    ("static-field", 0, 0, 0, ""),
    ("const-function", 0, 0, 0, ""),
    ("shared-mut-type", 0, 0, 0, ""),
    ("writable-alias", 0, 0, 0, ""),
    (
        "shared-derived-write",
        1,
        1,
        1,
        "constant raw pointer write is derived from a shared reference",
    ),
    ("packed-whole", 0, 0, 0, ""),
    (
        "packed-field-unaligned",
        1,
        1,
        1,
        "constant raw shared reference is unaligned",
    ),
    ("packed-field-aligned", 0, 0, 0, ""),
    (
        "packed-source-reference",
        1,
        1,
        1,
        "constant raw shared reference is unaligned",
    ),
    ("symbolic-zst", 0, 0, 0, ""),
    (
        "numeric-zst",
        0,
        1,
        1,
        "constant pointer has no allocation provenance",
    ),
    (
        "null",
        1,
        1,
        1,
        "constant pointer has no allocation provenance",
    ),
    (
        "expired",
        1,
        1,
        1,
        "constant pointer refers to expired storage",
    ),
    (
        "escape",
        1,
        1,
        1,
        "constant pointer refers to expired storage",
    ),
    (
        "uninitialized",
        1,
        1,
        1,
        "borrowed place may be uninitialized or moved",
    ),
    ("mutable-reference", 0, 0, 0, ""),
    (
        "union-field",
        0,
        1,
        1,
        "references into constant union images are not implemented",
    ),
    (
        "reinterpret",
        0,
        1,
        1,
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    (
        "mutable-static-reference",
        0,
        1,
        1,
        "constant raw shared references to mutable or external static storage are not implemented",
    ),
    (
        "foreign-static-reference",
        0,
        1,
        1,
        "constant raw shared references to mutable or external static storage are not implemented",
    ),
    (
        "packed-source-aligned",
        1,
        0,
        1,
        "reference to an unaligned packed field",
    ),
    (
        "cyclic-static",
        0,
        1,
        1,
        "constant raw shared references to cyclic static storage are not implemented",
    ),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn shared_reconstruction_matches_declared_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let evidence = root.join(format!(
        "obj/parallel/pa27/shared-reference-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, types, frontend, reason) in CASES {
        let input = root.join(format!("pa27/tests/spec/raw-shared-reference/{name}.rs"));
        if types == frontend {
            checked::check(&input, &evidence, name, (rust, frontend, reason))?;
        } else {
            checked::check_modes(
                &input,
                &evidence,
                name,
                (rust, [types, frontend, frontend], reason),
            )?;
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn shared_reference_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "local-products",
        "alias-rights",
        "packed-zst",
        "static-values",
        "static-reference-values",
    ] {
        let source = std::fs::read_to_string(root()?.join(format!(
            "pa27/tests/spec/raw-shared-reference-native/{name}.rs"
        )))?;
        native::check_cases(
            "parallel/pa27/shared-reference-native",
            &[(name, &source, false)],
        )?;
    }
    Ok(())
}
#[test]
fn repeated_shared_references_have_bounded_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut ordinary = std::collections::HashMap::new();
    let mut overhead = None;
    for raw in [false, true] {
        let mut previous = None;
        for count in [8, 32, 128] {
            let reference = if raw { "unsafe{&*p}" } else { "&x" };
            let pointer = if raw { "let p=&raw const x;" } else { "" };
            let text = format!(
                "const fn f(n:u32)->u32{{let x=1u32;{pointer}let mut total=0u32;let mut i=0u32;while i<n{{let r={reference};total+=*r;i+=1;}}total}}const X:u32=f({count});fn main(){{let _=X;}}"
            );
            rustc::qualify(
                &format!("parallel/pa27/shared-reference-scaling-{raw}-{count}"),
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
            if let Some(old) = previous {
                assert!(work.steps < old * 5);
            }
            assert!(work.peak_slots <= 2 * count + 64);
            if raw {
                let baseline = ordinary
                    .get(&count)
                    .copied()
                    .ok_or("ordinary observation")?;
                let extra = work
                    .peak_slots
                    .checked_sub(baseline)
                    .ok_or("raw slot baseline")?;
                if let Some(old) = overhead {
                    assert_eq!(extra, old);
                }
                overhead = Some(extra);
            } else {
                ordinary.insert(count, work.peak_slots);
            }
            previous = Some(work.steps);
        }
    }
    Ok(())
}
