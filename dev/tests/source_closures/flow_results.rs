//! Callable result constraints precede structural consumers, preserving proof.
use crate::{native_references as native, resolve, semantic};
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa24/tests/spec/callable-results";
fn cases() -> Result<Vec<(String, String, bool)>> {
    let mut cases = Vec::new();
    for line in resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (name, status) = line.split_once('\t').ok_or("callable result columns")?;
        let accepted = match status {
            "0" => true,
            "1" => false,
            _ => return Err("callable result outcome".into()),
        };
        cases.push((
            name.to_owned(),
            resolve::fixture(&format!("{PACKET}/{name}"))?,
            accepted,
        ));
    }
    assert_eq!(cases.len(), 52);
    Ok(cases)
}
#[test]
fn callable_outputs_reach_consumers_with_all_call_and_borrow_proofs() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join("obj/parallel/callable-results/public");
    std::fs::create_dir_all(&out)?;
    let mut failures = Vec::new();
    for (name, source, accepted) in cases()? {
        let path = out.join(&name);
        std::fs::write(&path, source)?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&path)
            .output()?;
        let error = String::from_utf8_lossy(&output.stderr);
        if output.status.code() != Some(i32::from(!accepted))
            || output.stderr.is_empty() != accepted
            || output.stdout.is_empty() == accepted
            || [
                "not implemented",
                "budget",
                "exhausted",
                "invalid inference",
                "requires normalization",
            ]
            .iter()
            .any(|part| error.contains(part))
        {
            failures.push(format!("{name}: {:?}: {error}", output.status));
        }
        std::fs::write(path.with_extension("stdout"), output.stdout)?;
        std::fs::write(path.with_extension("stderr"), output.stderr)?;
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn callable_result_work_tracks_sites_without_unrelated_type_scans() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join("obj/parallel/callable-results/work");
    std::fs::create_dir_all(&out)?;
    let mut previous = None;
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut ordinary = None;
        for noise in [0, 128] {
            let source = resolve::fixture(&format!("{PACKET}/width-{width}-noise-{noise}.rs"))?;
            let (_, facts) = semantic::analyze(&source)?;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
                facts.body_work.projection_steps,
                facts.body_work.projection_selections,
                facts.body_work.unsizing_inputs,
                facts.methods.selection_steps,
                facts.methods.selection_probes,
            ];
            observations.push(format!("width={width} noise={noise} work={work:?}"));
            std::fs::write(out.join("observations.txt"), observations.join("\n"))?;
            if noise == 0 {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(actual <= 4 * old + 1024, "{before:?}->{work:?}");
                    }
                }
                previous = Some(work);
                ordinary = Some(work);
            } else {
                assert_eq!(ordinary, Some(work));
            }
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn inferred_callable_results_execute_values_captures_moves_and_borrows() -> Result {
    let inputs: Vec<_> = cases()?.into_iter().filter(|row| row.2).collect();
    assert_eq!(inputs.len(), 39);
    let cases: Vec<_> = inputs
        .iter()
        .map(|(name, source, _)| {
            (
                name.strip_suffix(".rs").unwrap_or(name),
                source.as_str(),
                false,
            )
        })
        .collect();
    native::check_literal_cases("parallel/callable-results/native", &cases)
}
