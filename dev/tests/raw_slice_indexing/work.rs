//! Selected slice reads retain compressed roots; repeated accesses have bounded path work.
use crate::{qualify as rustc, resolve as source};
use std::fmt::Write;
fn input(width: usize, depth: usize, count: usize, noise: usize, raw: bool) -> String {
    let mut text = String::from("#![allow(dead_code,unused_mut,unused_assignments)]");
    for n in 0..noise {
        let _ = write!(text, "type Noise{n}=[u8;{}];", n + 1);
    }
    let mut root = format!("[7u32;{width}]");
    for _ in 0..depth {
        root = format!("({root},)");
    }
    let path = format!("root{}", ".0".repeat(depth));
    let pointer = if raw {
        format!("(&raw const {path}) as *const [u32]")
    } else {
        format!("&{path}")
    };
    let read = if raw {
        "result=unsafe{(*p)[1]};"
    } else {
        "result=p[1];"
    };
    let body = format!(
        "let root={root};let p={pointer};let mut result=0u32;{}result",
        read.repeat(count)
    );
    format!(
        "{text}const X:u32={{{body}}};fn operate()->u32{{{body}}}fn main(){{let _=X;let _=operate();}}"
    )
}
fn measured(label: &str, text: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    rustc::qualify(
        &format!("parallel/pa27/raw-slice-index/work-{label}"),
        text,
        true,
    )?;
    let mut report = source::resolve(text, true)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let _ = rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    let row = (facts.constant_work.steps, facts.constant_work.peak_slots);
    println!("raw-slice-work {label} steps={} slots={}", row.0, row.1);
    Ok(row)
}
#[test]
fn selected_reads_add_repetitions_without_materializing_compressed_array_width()
-> Result<(), Box<dyn std::error::Error>> {
    for raw in [false, true] {
        for depth in [0, 8] {
            let mut growth = None;
            for width in [8, 32, 128] {
                let mut rows = Vec::new();
                for count in [8, 32, 128] {
                    let label = format!("{raw}-{width}-{depth}-{count}");
                    let row = measured(&label, &input(width, depth, count, 0, raw))?;
                    assert!(
                        row.0 <= 4096 + count * (depth + 1) * 256,
                        "{label}: {row:?}"
                    );
                    assert!(
                        row.1 <= 4096 + count * 128 + depth * 128,
                        "{label}: {row:?}"
                    );
                    rows.push(row);
                }
                let delta = (rows[1].0 - rows[0].0, rows[2].0 - rows[1].0);
                assert_eq!(delta.1, delta.0 * 4);
                if let Some(previous) = growth {
                    assert_eq!(delta, previous);
                } else {
                    growth = Some(delta);
                }
            }
        }
        let plain = measured(&format!("noise-{raw}-0"), &input(32, 8, 32, 0, raw))?;
        let noise = measured(&format!("noise-{raw}-64"), &input(32, 8, 32, 64, raw))?;
        assert_eq!(plain, noise);
    }
    Ok(())
}
