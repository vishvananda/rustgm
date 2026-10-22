//! Repeated instances retain one shim and source work grows with statements.
use super::{rustc, source};
use std::fmt::Write;
fn input(width: usize, repeats: usize, unrelated: usize, counted: bool) -> String {
    let mut text = String::new();
    for n in 0..unrelated {
        let _ = write!(text, "type Noise{n}=[u8;{}];", n + 1);
    }
    let mut fields32 = String::new();
    let mut fields64 = String::new();
    for n in 0..width {
        let _ = write!(fields32, "{}u32,", n + 37);
        let _ = write!(fields64, "{}u64,", n + 41);
    }
    let operation = if counted {
        "unsafe{core::ptr::copy_nonoverlapping(p,r,1);core::ptr::copy_nonoverlapping(q,s,1);}"
    } else {
        "b=a;d=c;"
    };
    let body = format!(
        "let a=({fields32});let c=({fields64});let mut b=({});let mut d=({});let p=&raw const a;let q=&raw const c;let r=&raw mut b;let s=&raw mut d;{}(b.0,d.0)",
        "0u32,".repeat(width),
        "0u64,".repeat(width),
        operation.repeat(repeats)
    );
    format!(
        "{text}const X:(u32,u64)={{{body}}};fn operate()->(u32,u64){{{body}}}fn main(){{let _=X;let _=operate();}}"
    )
}
#[test]
fn counted_source_work_and_shims_reuse_full_instantiated_types()
-> Result<(), Box<dyn std::error::Error>> {
    for counted in [false, true] {
        for width in [2, 16] {
            let mut baseline = Vec::new();
            for unrelated in [0, 64] {
                let mut rows = Vec::new();
                let mut previous = None;
                for repeats in [8, 32, 128] {
                    let text = input(width, repeats, unrelated, counted);
                    rustc::qualify(
                        &format!(
                            "parallel/pa28/counted-memory/source-work-{counted}-{width}-{repeats}-{unrelated}"
                        ),
                        &text,
                        true,
                    )?;
                    let mut report = source::resolve(&text, true)?;
                    let rustgm::pipeline::Analysis {
                        mut facts,
                        prepared,
                    } = rustgm::pipeline::analyze_source(&report)?;
                    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
                    let output = rustgm::lower::instances::build_prepared(
                        &report, &mut facts, entry, prepared,
                    )?;
                    let measured = (facts.constant_work.steps, facts.constant_work.peak_slots);
                    println!(
                        "counted={counted} width={width} repeats={repeats} unrelated={unrelated} steps={} slots={}",
                        measured.0, measured.1
                    );
                    if let Some((steps, slots)) = previous {
                        assert!(measured.0 < steps * 5);
                        assert!(measured.1 < slots * 5);
                    }
                    previous = Some(measured);
                    rows.push(measured);
                    let shims = output
                        .symbols
                        .iter()
                        .filter(|s| s.name.contains("rg.intrinsic.copy_nonoverlapping.t"))
                        .count();
                    assert_eq!(shims, if counted { 2 } else { 0 });
                }
                if unrelated == 0 {
                    baseline = rows;
                } else {
                    assert_eq!(rows, baseline);
                }
            }
        }
    }
    Ok(())
}
