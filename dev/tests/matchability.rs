//! Real source equality and structural markers retain value-specific pattern proof.
use crate::provider_traits::lower;
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/structural-constant-patterns";

#[test]
fn source_constant_patterns_require_actual_equality_and_value_specific_markers() -> Result {
    let mut count = 0;
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let columns: Vec<_> = line.split('\t').collect();
        let [path, outcome, reason] = columns.as_slice() else {
            return Err("invalid structural pattern case".into());
        };
        let source = crate::resolve::fixture(path)?;
        let result = lower(&source);
        if *outcome == "0" {
            result.map_err(|error| format!("{path}: {error}"))?;
        } else {
            let error = result
                .err()
                .ok_or_else(|| format!("{path}: structural opponent unexpectedly accepted"))?;
            assert!(error.to_string().contains(reason), "{path}: {error}");
        }
        count += 1;
    }
    assert_eq!(count, 19);
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn actual_structural_patterns_execute_without_calling_handwritten_equality() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root
        .join("obj/parallel/structural-constant-patterns/native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let source = crate::resolve::fixture(&format!("{PACKET}/native.rs"))?;
    let observer = crate::resolve::fixture(&format!("{PACKET}/native-observer.c"))?;
    crate::ffi_program::check(
        root,
        &folder,
        "structural",
        &lower(&source)?,
        &observer,
        "1 0 1 0 1 0 0\n",
    )
}

#[test]
fn source_pattern_proof_work_tracks_distinct_constants() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/generic-structural.rs"))?;
    let (prefix, _) = source
        .rsplit_once("#[derive(PartialEq)]struct Token<T>")
        .ok_or("pattern prefix")?;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut text = prefix.to_owned();
        text.push_str("#[derive(PartialEq)]struct Token<T>(T);\n");
        for index in 0..width {
            writeln!(
                text,
                "const C{index}:Token<u8>=Token({index});fn probe{index}(x:Token<u8>)->bool{{match x{{C{index}=>true,_=>false}}}}"
            )?;
        }
        let report = crate::resolve::source_provider(&text, "structural_patterns")?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        let work = [
            analysis.facts.body_work.inference_steps,
            analysis.facts.body_work.canonical_steps,
            analysis.facts.dispatch_work().selections,
        ];
        println!("structural patterns width={width} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(
                    actual <= 5 * old + 512,
                    "width={width}: {before:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
        lower(&text)?;
    }
    Ok(())
}
