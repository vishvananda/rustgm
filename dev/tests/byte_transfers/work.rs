//! Repeated small writes and reads pay root conversion only once.
use super::{rustc, source};
use std::fmt::Write;
fn input(width: usize, repeats: usize, noise: usize, union: bool, moving: bool) -> String {
    let mut text = String::new();
    for n in 0..noise {
        let _ = write!(text, "type Noise{n}=[u8;{}];", n + 1);
    }
    if union {
        let _ = write!(
            text,
            "#[repr(align({width}))]union Target{{bits:u8,value:bool}}"
        );
    }
    text.push_str("union Source{bits:u8,value:bool}const X:u8={let source=Source{bits:2};");
    if union {
        text.push_str("let mut target=Target{bits:0};");
    } else {
        let _ = write!(text, "let mut target=[false;{width}];");
    }
    for _ in 0..repeats {
        if moving {
            if union {
                text.push_str("unsafe{core::ptr::copy(&raw const source.value,&raw mut target.value,1);}let _=unsafe{target.bits};target.value=true;");
            } else {
                let _ = write!(
                    text,
                    "unsafe{{core::ptr::copy(&raw const source.value,&raw mut target[{}],1);}}target[{}]=true;let _=target[0];",
                    width - 1,
                    width - 1
                );
            }
        } else if union {
            text.push_str("target.bits=2;let _=unsafe{target.bits};target.value=true;");
        } else {
            let _ = write!(
                text,
                "target[{}]=false;target[{}]=true;let _=target[0];",
                width - 1,
                width - 1
            );
        }
    }
    if union {
        text.push_str("unsafe{target.bits}");
    } else {
        text.push_str("1u8");
    }
    text.push_str("};fn main(){let _=X;}");
    text
}
#[test]
fn byte_storage_work_is_additive_in_width_and_repeated_access()
-> Result<(), Box<dyn std::error::Error>> {
    for union in [false, true] {
        for moving in [false, true] {
            let mut repeat_deltas = None;
            for width in [64, 512, 4096] {
                let mut baseline = Vec::new();
                for noise in [0, 64] {
                    let mut rows = Vec::new();
                    for repeats in [8, 32, 128] {
                        let text = input(width, repeats, noise, union, moving);
                        rustc::qualify(
                            &format!(
                                "parallel/pa28/byte-transport/work-{union}-{moving}-{width}-{repeats}-{noise}"
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
                        let _ = rustgm::lower::instances::build_prepared(
                            &report, &mut facts, entry, prepared,
                        )?;
                        let row = (facts.constant_work.steps, facts.constant_work.peak_slots);
                        println!(
                            "union={union} moving={moving} width={width} repeats={repeats} noise={noise} steps={} slots={}",
                            row.0, row.1
                        );
                        rows.push(row);
                    }
                    if noise == 0 {
                        baseline = rows;
                    } else {
                        assert_eq!(baseline, rows);
                    }
                }
                let deltas = (baseline[1].0 - baseline[0].0, baseline[2].0 - baseline[1].0);
                if union && !moving {
                    assert_eq!(deltas.1, deltas.0 * 4);
                } else if let Some(expected) = repeat_deltas {
                    assert_eq!(deltas, expected);
                } else {
                    repeat_deltas = Some(deltas);
                }
            }
        }
    }
    Ok(())
}

fn sparse_elements(width: usize, count: usize, mode: u8) -> String {
    let fields = "(),".repeat(width);
    let start = if mode == 0 {
        String::new()
    } else {
        format!("unsafe{{core::ptr::copy(&raw const bits.value,&raw mut target[0].{width},1);}}")
    };
    let transfer = if mode == 2 {
        "let [ref mut rest @ ..]=target;*rest=source;".to_owned()
    } else {
        format!(
            "unsafe{{core::ptr::copy_nonoverlapping(&raw const source[0],&raw mut target[0],{count});}}"
        )
    };
    format!(
        "union U{{bits:u8,value:bool}}const X:bool={{let bits=U{{bits:2}};let source=[({fields}true);{count}];let mut target=[({fields}false);{count}];{start}{transfer}target[{}].{width}}};fn main(){{let _=X;}}",
        count - 1
    )
}
#[test]
fn byte_repeat_work_adds_type_width_and_byte_extent() -> Result<(), Box<dyn std::error::Error>> {
    for mode in 0..3 {
        let mut deltas = None;
        for width in [4, 32, 128] {
            let mut rows = Vec::new();
            for count in [8, 32, 128] {
                let text = sparse_elements(width, count, mode);
                rustc::qualify(
                    &format!("parallel/pa28/byte-transport/sparse-{mode}-{width}-{count}"),
                    &text,
                    true,
                )?;
                let mut report = source::resolve(&text, true)?;
                let rustgm::pipeline::Analysis {
                    mut facts,
                    prepared,
                } = rustgm::pipeline::analyze_source(&report)?;
                let entry = rustgm::sema::entry::select(&mut report, &facts)?;
                let _ =
                    rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
                let row = (facts.constant_work.steps, facts.constant_work.peak_slots);
                println!(
                    "sparse mode={mode} width={width} count={count} steps={} slots={}",
                    row.0, row.1
                );
                rows.push(row);
            }
            let current = (rows[1].0 - rows[0].0, rows[2].0 - rows[1].0);
            if let Some(previous) = deltas {
                assert_eq!(current, previous);
            } else {
                deltas = Some(current);
            }
        }
    }
    Ok(())
}
