//! Tail metadata selection touches the retained path, not unrelated fields or types.
use crate::{qualify as rustc, resolve as source};
use std::fmt::Write;
fn input(width: usize, depth: usize, count: usize, noise: usize, dst: bool) -> String {
    let mut text = String::from("#![allow(dead_code,unused_assignments)]");
    for n in 0..noise {
        let _ = write!(text, "type Noise{n}=[u8;{}];", n + 1);
    }
    text.push_str("struct Tail<T:?Sized>{");
    let mut fields = String::new();
    for n in 0..width {
        let _ = write!(text, "p{n}:u8,");
        let _ = write!(fields, "p{n}:0u8,");
    }
    text.push_str("tail:T}");
    let mut owner = "Tail".to_owned();
    let mut value = format!("Tail{{{fields}tail:[1u8,2]}}");
    for n in 0..depth {
        let _ = write!(text, "struct W{n}<T:?Sized>{{inner:{owner}<T>}}");
        owner = format!("W{n}");
        value = format!("{owner}{{inner:{value}}}");
    }
    let element = if dst { "[u8]" } else { "[u8;2]" };
    let path = format!("r{}.tail", ".inner".repeat(depth));
    let _ = write!(
        text,
        "const fn take(r:&{owner}<{element}>)->u8{{match &{path}{{[a,b]=>*a+*b,_=>0}}}}"
    );
    let body = format!(
        "let root={value};let mut value=0u8;{}value",
        "value=take(&root);".repeat(count)
    );
    let _ = write!(
        text,
        "const X:u8={{{body}}};fn operate()->u8{{{body}}}fn main(){{let _=X;let _=operate();}}"
    );
    text
}
fn measured(label: &str, text: &str) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
    rustc::qualify(
        &format!("parallel/pa25/dst-tail-metadata/work-{label}"),
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
    let row = (
        facts.constant_work.steps,
        facts.constant_work.frames_started,
        facts.constant_work.peak_slots,
    );
    println!(
        "dst-tail-work {label} steps={} frames={} slots={}",
        row.0, row.1, row.2
    );
    Ok(row)
}
#[test]
fn repeated_tail_selection_does_not_rescan_unrelated_fields_or_store_types()
-> Result<(), Box<dyn std::error::Error>> {
    for dst in [false, true] {
        for depth in [0, 8] {
            let mut growth = None;
            for width in [4, 32, 128] {
                let mut rows = Vec::new();
                for count in [8, 32, 128] {
                    let label = format!("{dst}-{width}-{depth}-{count}");
                    let row = measured(&label, &input(width, depth, count, 0, dst))?;
                    assert!((count..=count + 1).contains(&row.1), "{label}: {row:?}");
                    assert!(
                        row.0 <= 4096 + width * 128 + depth * 512 + count * (depth + 1) * 512,
                        "{label}: {row:?}"
                    );
                    assert!(
                        row.2 <= 4096 + width * 128 + count * 128 + depth * 128,
                        "{label}: {row:?}"
                    );
                    rows.push(row);
                }
                let delta = (rows[1].0 - rows[0].0, rows[2].0 - rows[1].0);
                assert_eq!(delta.1, delta.0 * 4);
                if let Some(previous) = growth {
                    assert_eq!(previous, delta);
                } else {
                    growth = Some(delta);
                }
            }
        }
        let plain = measured(&format!("noise-{dst}-0"), &input(32, 8, 32, 0, dst))?;
        let noise = measured(&format!("noise-{dst}-64"), &input(32, 8, 32, 64, dst))?;
        assert_eq!(plain, noise);
    }
    Ok(())
}
