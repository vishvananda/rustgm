//! Physical address replay is charged by actual count and depth, without a new cache.
use crate::test_source;
use std::{fmt::Write, path::Path};

fn source(depth: usize, repeats: usize, noise: usize) -> Result<String, String> {
    let mut text = String::from("#![recursion_limit=\"512\"]enum E{A(u32),Empty}");
    let mut constructor = String::from("E::A(37)");
    let mut parent = String::from("E");
    for n in 0..depth {
        write!(&mut text, "struct S{n}{{p:{parent}}}").map_err(|e| e.to_string())?;
        constructor = format!("S{n}{{p:{constructor}}}");
        parent = format!("S{n}");
    }
    for n in 0..noise {
        write!(&mut text, "struct Noise{n}{{a:[u8;{n}]}}").map_err(|e| e.to_string())?;
    }
    let field = format!("r{}", ".p".repeat(depth));
    write!(&mut text, "const N:u32=unsafe{{let mut r={constructor};let mut i=0;while i<{repeats}{{let p=match &mut {field}{{E::A(v)=>v as *mut u32,_=>loop{{}}}};core::ptr::write(p,i);i+=1;}}let v=match {field}{{E::A(v)=>v,_=>0}};1/(if v=={}{{1}}else{{0}})}};fn main(){{}}", repeats-1).map_err(|e| e.to_string())?;
    Ok(text)
}

#[test]
fn default_enum_address_replay_scales_with_count_times_depth_and_ignores_unrelated_types()
-> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let output = root.join(format!(
        "obj/parallel/pa27/default-tag-enum-bytes/scaling-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&output).map_err(|e| e.to_string())?;
    let mut slopes = Vec::new();
    for depth in [1usize, 8, 32] {
        let mut observations = Vec::new();
        for noise in [0, 1024] {
            let mut rows = Vec::new();
            for repeats in [8usize, 32, 128] {
                let source = source(depth, repeats, noise)?;
                std::fs::write(
                    output.join(format!("depth{depth}-repeat{repeats}-noise{noise}.rs")),
                    &source,
                )
                .map_err(|e| e.to_string())?;
                let report = test_source::resolve(&source, true)?;
                let facts = crate::pipeline::analyze_source(&report)
                    .map_err(|e| e.to_string())?
                    .facts;
                let steps = facts.constant_work.steps;
                println!(
                    "default-replay depth={depth} noise={noise} repeats={repeats} steps={steps} peak={}",
                    facts.constant_work.peak_slots
                );
                rows.push(steps);
            }
            assert_eq!(rows[2] - rows[1], 4 * (rows[1] - rows[0]));
            observations.push(rows);
        }
        assert_eq!(observations[0], observations[1]);
        slopes.push((observations[0][1] - observations[0][0]) / 24);
    }
    assert_eq!(7 * (slopes[2] - slopes[1]), 24 * (slopes[1] - slopes[0]));
    Ok(())
}
