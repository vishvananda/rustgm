//! Fill work adds selected extent, one root conversion and retained path depth.
use crate::{qualify as rustc, resolve as source};
use std::fmt::Write;
fn input(
    width: usize,
    count: usize,
    depth: usize,
    repeats: usize,
    noise: usize,
    fill: bool,
) -> String {
    let mut text = String::new();
    for n in 0..noise {
        let _ = write!(text, "type Noise{n}=[u8;{}];", n + 1);
    }
    let value = format!("({}{}u8)", "(),".repeat(width), if fill { 0 } else { 7 });
    let mut root = format!("[{value};{count}]");
    for _ in 0..depth {
        root = format!("({root},)");
    }
    let path = format!("root{}", ".0".repeat(depth));
    let operation = if fill {
        format!("unsafe{{core::ptr::write_bytes(&raw mut {path}[0],7,{count});}}")
    } else {
        String::new()
    };
    let body = format!(
        "let mut root={root};{}{path}[{}].{width}",
        operation.repeat(repeats),
        count - 1
    );
    format!(
        "#![allow(dead_code,unused_mut)]{text}const X:u8={{{body}}};fn operate()->u8{{{body}}}fn main(){{let _=X;let _=operate();}}"
    )
}
fn measured(
    label: &str,
    text: &str,
    fill: bool,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    rustc::qualify(&format!("parallel/pa28/byte-fill/work-{label}"), text, true)?;
    let mut report = source::resolve(text, true)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let output = rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    assert_eq!(
        output
            .symbols
            .iter()
            .filter(|s| s.name.contains("rg.intrinsic.write_bytes.t"))
            .count(),
        usize::from(fill)
    );
    let row = (facts.constant_work.steps, facts.constant_work.peak_slots);
    println!("fill-work {label} steps={} slots={}", row.0, row.1);
    Ok(row)
}
#[test]
fn byte_fill_work_adds_type_width_path_depth_and_byte_extent()
-> Result<(), Box<dyn std::error::Error>> {
    for fill in [false, true] {
        let mut count_deltas = None;
        for width in [4, 32, 128] {
            for depth in [0, 8] {
                let mut rows = Vec::new();
                for count in [8, 32, 128] {
                    let label = format!("extent-{fill}-{width}-{count}-{depth}");
                    rows.push(measured(
                        &label,
                        &input(width, count, depth, 1, 0, fill),
                        fill,
                    )?);
                }
                let delta = (rows[1].0 - rows[0].0, rows[2].0 - rows[1].0);
                if let Some(expected) = count_deltas {
                    assert_eq!(delta, expected);
                } else {
                    count_deltas = Some(delta);
                }
            }
        }
        let ordinary = measured(
            &format!("noise-{fill}-0"),
            &input(32, 32, 8, 1, 0, fill),
            fill,
        )?;
        let noise = measured(
            &format!("noise-{fill}-64"),
            &input(32, 32, 8, 1, 64, fill),
            fill,
        )?;
        assert_eq!(ordinary, noise);
    }
    Ok(())
}
#[test]
fn byte_fill_repeated_spans_do_not_reencode_whole_roots() -> Result<(), Box<dyn std::error::Error>>
{
    let mut deltas = None;
    for count in [64, 512, 4096] {
        let mut rows = Vec::new();
        for repeats in [8, 32, 128] {
            // The zero-sized tuple prefix keeps the element one byte wide.
            let text = input(1, count, 0, repeats, 0, true).replace(
                &format!("write_bytes(&raw mut root[0],7,{count})"),
                "write_bytes(&raw mut root[0],7,1)",
            );
            rows.push(measured(&format!("repeat-{count}-{repeats}"), &text, true)?);
        }
        // Conversion touches the whole root once; subsequent one-byte fills
        // must not reencode it or scan its untouched cells.
        let delta = (rows[1].0 - rows[0].0, rows[2].0 - rows[1].0);
        assert_eq!(delta.1, delta.0 * 4);
        if let Some(previous) = deltas {
            assert_eq!(delta, previous);
        }
        deltas = Some(delta);
    }
    Ok(())
}
