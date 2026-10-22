//! Unaligned pointer transfers retain ownership and every non-alignment raw check.
use crate::checked_sources as checked;
use crate::native_references as native;
use crate::reconstruction_work as work;
use std::path::Path;
const CASES: &[(&str, i32, [i32; 3], &str)] = &[
    ("packed-read", 0, [0; 3], ""),
    (
        "fresh-raw-anchor",
        1,
        [1; 3],
        "constant raw pointer read is unaligned",
    ),
    ("packed-write", 0, [0; 3], ""),
    ("packed-nested", 0, [0; 3], ""),
    ("const-generic", 0, [0; 3], ""),
    ("packed-product", 0, [0; 3], ""),
    ("packed-array", 0, [0; 3], ""),
    ("packed-reference", 0, [0; 3], ""),
    ("packed-pointer", 0, [0; 3], ""),
    ("symbolic-zst", 0, [0; 3], ""),
    ("union-alias", 0, [0; 3], ""),
    ("union-repair", 0, [0; 3], ""),
    ("write-authority", 0, [0; 3], ""),
    (
        "shared-derived",
        1,
        [1; 3],
        "constant raw pointer write is derived from a shared reference",
    ),
    (
        "expired-read",
        1,
        [1; 3],
        "constant pointer refers to expired storage",
    ),
    (
        "expired-write",
        1,
        [1; 3],
        "constant pointer refers to expired storage",
    ),
    (
        "uninitialized-source",
        1,
        [1; 3],
        "borrowed place may be uninitialized or moved",
    ),
    (
        "invalid-bool",
        1,
        [1; 3],
        "constant union scalar read has invalid bool representation",
    ),
    (
        "uninitialized-bytes",
        1,
        [1; 3],
        "constant union scalar read uses uninitialized bytes",
    ),
    (
        "ordinary-aligned-read",
        1,
        [1; 3],
        "constant raw pointer read is unaligned",
    ),
    (
        "ordinary-aligned-write",
        1,
        [1; 3],
        "constant raw pointer write is unaligned",
    ),
    (
        "packed-reference-forbidden",
        1,
        [0, 1, 1],
        "reference to an unaligned packed field",
    ),
    (
        "read-unsafe",
        1,
        [1; 3],
        "call to unsafe function requires an unsafe context",
    ),
    (
        "write-unsafe",
        1,
        [1; 3],
        "call to unsafe function requires an unsafe context",
    ),
    ("wrong-type", 1, [1; 3], "incompatible types"),
    ("wrong-arity", 1, [1; 3], "call argument count mismatch"),
    ("reified", 0, [0; 3], ""),
    (
        "const-reified",
        1,
        [1; 3],
        "non-const calls are not allowed in constant initializers",
    ),
    ("identity", 0, [0; 3], ""),
    ("shadow", 0, [0; 3], ""),
    ("ordinary-control", 0, [0; 3], ""),
    (
        "explicit-align-packed-invalid",
        1,
        [1; 3],
        "packed aggregate transitively contains an explicitly aligned type",
    ),
];
#[test]
fn unaligned_memory_matches_independent_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa28/unaligned/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for &(name, rust, statuses, reason) in CASES {
        let path = root.join(format!("pa28/tests/spec/unaligned-memory/{name}.rs"));
        if statuses == [statuses[0]; 3] {
            checked::check(&path, &out, name, (rust, statuses[0], reason))?;
        } else {
            checked::check_modes(&path, &out, name, (rust, statuses, reason))?;
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn unaligned_memory_executes_owned_snapshots() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let cases = [
        "scalar-widths",
        "owned-products",
        "reference-values",
        "overwrite-no-drop",
        "read-forget",
        "reified-generic",
        "once-operands",
        "const-results",
        "self-snapshot",
        "wide-reference",
    ]
    .into_iter()
    .map(|name| {
        std::fs::read_to_string(root.join(format!("pa28/tests/unaligned-memory-native/{name}.rs")))
            .map(|body| (name, body))
    })
    .collect::<Result<Vec<_>, _>>()?;
    let borrowed = cases
        .iter()
        .map(|(name, body)| (*name, body.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("parallel/pa28/unaligned/native", &borrowed)
}
#[test]
fn unaligned_memory_work_has_bounded_overhead() -> Result<(), Box<dyn std::error::Error>> {
    work::compare("parallel/pa28/unaligned/loop", |memory, count| {
        let operation = if memory {
            "unsafe{core::ptr::write_unaligned(&raw mut p.b,core::ptr::read_unaligned(&raw const p.b)+1);}"
        } else {
            "p.b=p.b+1;"
        };
        format!(
            "#[repr(C,packed)]struct P{{a:u8,b:u32}}const fn f(n:u32)->u32{{let mut p=P{{a:3,b:0}};let mut i=0u32;while i<n{{{operation}i=i+1;}}p.b}}const X:u32=f({count});fn main(){{let _=X;}}"
        )
    })
}

#[test]
fn unaligned_transfer_layouts_and_shims_scale_with_touched_values()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    let mut all_widths = Vec::new();
    for width in [8, 32, 128] {
        let mut records = Vec::new();
        for unrelated in [0, 64] {
            for repeats in [1, 8, 32] {
                let mut noise = String::new();
                for n in 0..unrelated {
                    write!(noise, "type Noise{n}=[u8;{}];", n + 1)?;
                }
                let text = format!(
                    "{noise}struct V{{v:({})}}#[repr(C,packed)]struct P{{a:u8,v:V}}const X:u32={{let mut p=P{{a:3,v:V{{v:({})}}}};{}p.v.v.0}};fn operate(){{let mut p=P{{a:3,v:V{{v:({})}}}};{}}}fn main(){{let _=X;operate();}}",
                    "u32,".repeat(width), "37u32,".repeat(width),
                    "unsafe{core::ptr::write_unaligned(&raw mut p.v,core::ptr::read_unaligned(&raw const p.v));}".repeat(repeats),
                    "37u32,".repeat(width),
                    "unsafe{core::ptr::write_unaligned(&raw mut p.v,core::ptr::read_unaligned(&raw const p.v));}".repeat(repeats),
                );
                work::rustc::qualify(
                    &format!("parallel/pa28/unaligned/layout-{width}-{repeats}-{unrelated}"),
                    &text,
                    true,
                )?;
                let mut report = work::source::resolve(&text, true)?;
                let rustgm::pipeline::Analysis {
                    mut facts,
                    prepared,
                } = rustgm::pipeline::analyze_source(&report)?;
                let entry = rustgm::sema::entry::select(&mut report, &facts)?;
                let output =
                    rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
                let shims = output
                    .symbols
                    .iter()
                    .filter(|s| {
                        s.name.contains("rg.intrinsic.read_unaligned.t")
                            || s.name.contains("rg.intrinsic.write_unaligned.t")
                    })
                    .count();
                assert_eq!(shims, 2);
                let measured = (facts.constant_work.steps, facts.constant_work.peak_slots);
                println!(
                    "width={width} repeats={repeats} unrelated={unrelated} steps={} slots={}",
                    measured.0, measured.1
                );
                assert!(measured.0 < 256 * (width + repeats + 1));
                assert!(measured.1 < 32 * (width + repeats + 1));
                if unrelated == 0 {
                    records.push(measured);
                } else {
                    let index = [1, 8, 32]
                        .iter()
                        .position(|&n| n == repeats)
                        .ok_or("repeat")?;
                    assert_eq!(measured, records[index]);
                }
            }
        }
        assert_eq!(
            7 * (records[2].0 - records[1].0),
            24 * (records[1].0 - records[0].0)
        );
        assert_eq!(
            7 * (records[2].1 - records[1].1),
            24 * (records[1].1 - records[0].1)
        );
        all_widths.push(records);
    }
    for ((small, medium), large) in all_widths[0].iter().zip(&all_widths[1]).zip(&all_widths[2]) {
        assert_eq!(large.0 - medium.0, 4 * (medium.0 - small.0));
        assert_eq!(large.1 - medium.1, 4 * (medium.1 - small.1));
    }
    for records in &all_widths[1..] {
        assert_eq!(
            records[2].0 - records[0].0,
            all_widths[0][2].0 - all_widths[0][0].0
        );
        assert_eq!(
            records[2].1 - records[0].1,
            all_widths[0][2].1 - all_widths[0][0].1
        );
    }
    Ok(())
}
