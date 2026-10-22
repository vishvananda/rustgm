//! Canonical method publication and constant arithmetic scale with actual calls.
use crate::{qualify as rustc, resolve as source};
use rustgm::support::intrinsics::Intrinsic;
use std::fmt::Write;
fn input(width: usize, depth: usize, count: usize, noise: usize) -> String {
    let mut text = String::from("#![allow(dead_code)]");
    for n in 0..noise {
        let _ = write!(text, "type Noise{n}=[u8;{}];", n + 1);
    }
    let mut value = format!("[7u8;{width}]");
    for _ in 0..depth {
        value = format!("({value},)");
    }
    let path = format!("root{}", ".0".repeat(depth));
    let operation = "unsafe{result=*p.add(1);}".repeat(count);
    let _ = write!(
        text,
        "const X:u8={{let root={value};let p=(&raw const {path}) as *const u8;let mut result=0u8;{operation}result}};fn main(){{let _=X;}}"
    );
    text
}
fn measured(label: &str, text: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    rustc::qualify(
        &format!("parallel/pa28/pointer-add-work-{label}"),
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
    assert_eq!(
        facts
            .intrinsics
            .iter()
            .filter(|d| d.kind.is_pointer_offset())
            .count(),
        2
    );
    let constant = facts
        .intrinsics
        .get(Intrinsic::ConstPointerAdd)
        .ok_or("const identity")?;
    let mutable = facts
        .intrinsics
        .get(Intrinsic::MutPointerAdd)
        .ok_or("mutable identity")?;
    assert_ne!(constant.owner, mutable.owner);
    assert!(facts.definitions[constant.definition.0 as usize].is_some());
    assert!(facts.definitions[mutable.definition.0 as usize].is_none());
    let row = (facts.constant_work.steps, facts.constant_work.peak_slots);
    println!("pointer-add {label} steps={} slots={}", row.0, row.1);
    Ok(row)
}
#[test]
fn repeated_offsets_reuse_canonical_method_and_closed_layout_work()
-> Result<(), Box<dyn std::error::Error>> {
    for depth in [0, 8] {
        let mut prior = None;
        for width in [8, 32, 128] {
            let mut rows = Vec::new();
            for count in [8, 32, 128] {
                let row = measured(
                    &format!("{width}-{depth}-{count}"),
                    &input(width, depth, count, 0),
                )?;
                assert!(row.0 <= 16384 + count * (depth + 1) * 1024);
                assert!(row.1 <= 4096 + count * 64 + depth * 64);
                rows.push(row);
            }
            let delta = (rows[1].0 - rows[0].0, rows[2].0 - rows[1].0);
            assert_eq!(delta.1, delta.0 * 4);
            if let Some(previous) = prior {
                assert_eq!(delta, previous);
            } else {
                prior = Some(delta);
            }
        }
    }
    assert_eq!(
        measured("noise-0", &input(32, 8, 32, 0))?,
        measured("noise-64", &input(32, 8, 32, 64))?
    );
    Ok(())
}
