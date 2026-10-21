//! Repeated operations share full callable Copy capabilities without sharing permissions.
use super::work::{rustc, source};
use std::fmt::Write;
fn input(width: usize, repeats: usize, unrelated: usize) -> String {
    let mut noise = String::new();
    for n in 0..unrelated {
        let _ = write!(noise, "type Noise{n}=[u8;{}];", n + 1);
    }
    let mut fields32 = String::new();
    let mut fields64 = String::new();
    for n in 0..width {
        let _ = write!(fields32, "{n}u32,");
        let _ = write!(fields64, "{n}u64,");
    }
    let mut operations = String::new();
    for _ in 0..repeats {
        operations.push_str("unsafe{core::ptr::write(p,core::ptr::read(p));core::ptr::write(q,core::ptr::read(q));}");
    }
    let body = format!(
        "let mut a=({fields32});let mut b=({fields64});let p=&raw mut a;let q=&raw mut b;{operations}(a.0,b.0)"
    );
    format!(
        "{noise}const X:(u32,u64)={{{body}}};fn operate()->(u32,u64){{{body}}}fn main(){{let _=X;let _=operate();}}"
    )
}
#[test]
fn repeated_memory_calls_reuse_capabilities_across_width_and_unrelated_types()
-> Result<(), Box<dyn std::error::Error>> {
    for width in [2, 16, 64] {
        let mut expected = None;
        for unrelated in [0, 64] {
            let mut previous = None;
            for repeats in [1, 8, 32] {
                let text = input(width, repeats, unrelated);
                rustc::qualify(
                    &format!("parallel/pa28/pointer-capabilities/{width}-{repeats}-{unrelated}"),
                    &text,
                    true,
                )?;
                let mut report = source::resolve(&text, true)?;
                let rustgm::pipeline::Analysis {
                    mut facts,
                    prepared,
                } = rustgm::pipeline::analyze_source(&report)?;
                let entry = rustgm::sema::entry::select(&mut report, &facts)?;
                let output =
                    rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
                let work = &facts.constant_work;
                println!(
                    "width={width} repeats={repeats} unrelated={unrelated} copy={} total={} slots={}",
                    work.copy_steps, work.steps, work.peak_slots
                );
                assert_eq!(work.copy_steps, width * 8 + 20);
                // Two typed aggregates and four source operations per repetition.
                assert!(work.peak_slots <= width * 4 + repeats * 18 + 32);
                if let Some(old) = expected {
                    assert_eq!(work.copy_steps, old);
                } else {
                    expected = Some(work.copy_steps);
                }
                if let Some(old) = previous {
                    assert!(work.steps < old * 9);
                }
                previous = Some(work.steps);
                let shims = output
                    .symbols
                    .iter()
                    .filter(|s| {
                        s.name.contains("rg.intrinsic.read.t")
                            || s.name.contains("rg.intrinsic.write.t")
                    })
                    .count();
                assert_eq!(shims, 4);
            }
        }
    }
    Ok(())
}
