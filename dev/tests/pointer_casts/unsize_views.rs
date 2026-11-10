//! Nominal cast operands retain selected tail metadata and ordinary borrow proofs.
use super::native;
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn cases() -> Result<Vec<(String, String, bool)>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let mut cases = Vec::new();
    for (stage, folder, count) in [
        ("pa22", "tests/spec/casts/nominal-reference", 36),
        ("pa23", "tests/spec/nominal-reference-casts", 5),
    ] {
        let folder = root.join(stage).join(folder);
        let rows = std::fs::read_to_string(folder.join("cases.tsv"))?;
        assert_eq!(rows.lines().count(), count);
        for row in rows.lines() {
            let (path, status) = row.split_once('\t').ok_or("cast columns")?;
            let accepted = match status {
                "0" => true,
                "1" => false,
                _ => return Err("cast status".into()),
            };
            let name = format!("{stage}-{}", path.trim_end_matches(".rs"));
            cases.push((name, std::fs::read_to_string(folder.join(path))?, accepted));
        }
    }
    Ok(cases)
}

#[test]
fn nominal_reference_casts_keep_elements_markers_lifetimes_and_loans() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join("obj/parallel/dst-casts/public");
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for (name, source, accepted) in cases()? {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, source)?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&path)
            .output()?;
        let error = String::from_utf8_lossy(&output.stderr);
        let expected = i32::from(!accepted);
        if output.status.code() != Some(expected)
            || output.stderr.is_empty() != accepted
            || (!accepted && !output.stdout.is_empty())
            || [
                "not implemented",
                "budget exceeded",
                "projection requires normalization",
            ]
            .iter()
            .any(|fence| error.contains(fence))
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
fn nominal_cast_work_follows_sites_without_scanning_unrelated_types() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join("pa22/tests/spec/casts/nominal-reference");
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut ordinary = None;
        for noise in [0, 128] {
            let source =
                std::fs::read_to_string(folder.join(format!("width-{width}-noise-{noise}.rs")))?;
            let report = crate::semantic::resolve(&source)?;
            let facts = rustgm::pipeline::analyze_source(&report)?.facts;
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
            println!("nominal casts width={width} noise={noise} work={work:?}");
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
fn nominal_reference_casts_execute_real_fields_metadata_and_effects() -> Result {
    let inputs: Vec<_> = cases()?.into_iter().filter(|case| case.2).collect();
    assert_eq!(inputs.len(), 27);
    let borrowed: Vec<_> = inputs
        .iter()
        .map(|(name, source, _)| (name.as_str(), source.as_str(), false))
        .collect();
    native::check_literal_cases("parallel/dst-casts/native", &borrowed)
}
