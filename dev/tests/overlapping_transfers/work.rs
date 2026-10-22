//! Repeated moves share their exact shims; storage work does not rediscover type ancestry.
use super::{rustc, source};
use std::fmt::Write;
fn input(width: usize, repeats: usize, unrelated: usize, moving: bool) -> String {
    let mut text = String::new();
    for n in 0..unrelated {
        let _ = write!(text, "type Noise{n}=[u8;{}];", n + 1);
    }
    let mut values = Vec::new();
    for suffix in ["u32", "u64"] {
        let mut array = String::new();
        for n in [37, 41, 43] {
            let row = format!("{n}{suffix},").repeat(width);
            let _ = write!(array, "({row}),");
        }
        values.push(array);
    }
    let operation = if moving {
        "unsafe{core::ptr::copy(&raw const a[0],&raw mut a[1],2);core::ptr::copy(&raw const b[1],&raw mut b[0],2);}"
    } else {
        "a[1]=a[0];a[2]=a[0];b[0]=b[1];b[1]=b[2];"
    };
    let body = format!(
        "let mut a=[{}];let mut b=[{}];{}(a[2].0,b[0].0)",
        values[0],
        values[1],
        operation.repeat(repeats)
    );
    format!(
        "{text}const X:(u32,u64)={{{body}}};fn operate()->(u32,u64){{{body}}}fn main(){{let _=X;let _=operate();}}"
    )
}
#[test]
fn overlapping_source_work_shares_two_instantiated_shims() -> Result<(), Box<dyn std::error::Error>>
{
    for moving in [false, true] {
        for width in [2, 16] {
            let mut baseline = Vec::new();
            for noise in [0, 64] {
                let mut rows = Vec::new();
                let mut previous = None;
                for repeats in [8, 32, 128] {
                    let text = input(width, repeats, noise, moving);
                    rustc::qualify(
                        &format!(
                            "parallel/pa28/overlapping-memory/work-{moving}-{width}-{repeats}-{noise}"
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
                        "moving={moving} width={width} repeats={repeats} noise={noise} steps={} slots={}",
                        measured.0, measured.1
                    );
                    if let Some((steps, slots)) = previous {
                        assert!(measured.0 < steps * 5);
                        assert!(measured.1 < slots * 5);
                    }
                    previous = Some(measured);
                    rows.push(measured);
                    assert_eq!(
                        output
                            .symbols
                            .iter()
                            .filter(|s| s.name.contains("rg.intrinsic.copy.t"))
                            .count(),
                        if moving { 2 } else { 0 }
                    );
                }
                if noise == 0 {
                    baseline = rows;
                } else {
                    assert_eq!(rows, baseline);
                }
            }
        }
    }
    Ok(())
}
