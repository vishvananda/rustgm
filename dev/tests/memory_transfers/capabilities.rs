//! Repeated transfers retain typed layout and shim identity across unrelated types.
use super::work::{rustc, source};
use std::fmt::Write;
fn input(width: usize, repeats: usize, unrelated: usize, owned: bool) -> String {
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
    let (declarations, a, b, result) = if owned {
        (
            format!(
                "struct P{{v:({})}}struct Q{{v:({})}}",
                "u32,".repeat(width),
                "u64,".repeat(width)
            ),
            format!("P{{v:({fields32})}}"),
            format!("Q{{v:({fields64})}}"),
            "(a.v.0,b.v.0)",
        )
    } else {
        (
            String::new(),
            format!("({fields32})"),
            format!("({fields64})"),
            "(a.0,b.0)",
        )
    };
    let body = format!(
        "let mut a={a};let mut b={b};let p=&raw mut a;let q=&raw mut b;{operations}{result}"
    );
    format!(
        "{noise}{declarations}const X:(u32,u64)={{{body}}};fn operate()->(u32,u64){{{body}}}fn main(){{let _=X;let _=operate();}}"
    )
}
#[test]
fn repeated_memory_calls_reuse_layouts_and_shims_across_width_and_unrelated_types()
-> Result<(), Box<dyn std::error::Error>> {
    for owned in [false, true] {
        for width in [2, 16, 64] {
            for unrelated in [0, 64] {
                let mut previous = None;
                for repeats in [1, 8, 32] {
                    let text = input(width, repeats, unrelated, owned);
                    rustc::qualify(
                        &format!(
                            "parallel/pa28/pointer-layouts/{owned}-{width}-{repeats}-{unrelated}"
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
                    let work = &facts.constant_work;
                    println!(
                        "owned={owned} width={width} repeats={repeats} unrelated={unrelated} total={} slots={}",
                        work.steps, work.peak_slots
                    );
                    // Four slots per field and eighteen per statement group;
                    // nominal wrapper roots retain four additional fixed slots.
                    let fixed = if owned { 23 } else { 19 };
                    assert_eq!(work.peak_slots, width * 4 + repeats * 18 + fixed);
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
    }
    Ok(())
}
