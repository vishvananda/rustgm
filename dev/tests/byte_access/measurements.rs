//! Source work separates one backing conversion from repeated selected-byte demands.
use crate::{qualify as rustc, resolve as source};
use std::fmt::Write;
fn input(width: usize, depth: usize, count: usize, noise: usize) -> String {
    let mut text = String::from("#![allow(dead_code,unused_mut,unused_assignments)]");
    for n in 0..noise {
        let _ = write!(text, "type Noise{n}=[u8;{}];", n + 1);
    }
    let mut root = format!("[7u32;{width}]");
    for _ in 0..depth {
        root = format!("({root},)");
    }
    let path = format!("root{}", ".0".repeat(depth));
    let body = format!(
        "let mut root={root};let p=(&raw mut {path}) as *mut [u8;{}];let mut result=0u8;{}result",
        width * 4,
        "unsafe{(*p)[0]=9;result=(*p)[0];}".repeat(count)
    );
    format!(
        "{text}const X:u8={{{body}}};fn operate()->u8{{{body}}}fn main(){{let _=X;let _=operate();}}"
    )
}
fn measured(label: &str, text: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    rustc::qualify(
        &format!("parallel/pa27/byte-access-work-{label}"),
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
    println!("byte-source {label} steps={} slots={}", row.0, row.1);
    Ok(row)
}
#[test]
fn repeated_byte_access_adds_work_after_one_width_and_depth_preparation()
-> Result<(), Box<dyn std::error::Error>> {
    for depth in [0, 8] {
        let mut previous = None;
        for width in [8, 32, 128] {
            let mut rows = Vec::new();
            for count in [8, 32, 128] {
                let row = measured(
                    &format!("{width}-{depth}-{count}"),
                    &input(width, depth, count, 0),
                )?;
                assert!(row.0 <= 32768 + width * 128 + count * (depth + 1) * 2048);
                assert!(row.1 <= 8192 + width * 16 + count * 256 + depth * 128);
                rows.push(row);
            }
            let delta = (rows[1].0 - rows[0].0, rows[2].0 - rows[1].0);
            assert_eq!(delta.1, delta.0 * 4);
            if let Some(prior) = previous {
                assert_eq!(delta, prior);
            } else {
                previous = Some(delta);
            }
        }
    }
    let base = measured("noise-0", &input(32, 8, 32, 0))?;
    let noisy = measured("noise-64", &input(32, 8, 32, 64))?;
    assert_eq!(base, noisy);
    Ok(())
}
