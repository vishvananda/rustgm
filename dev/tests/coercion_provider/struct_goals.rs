//! Source Unsize follows completed storage tails and retains all caller proofs.
use crate::provider_traits::{TestResult, lower};
const PACKET: &str = "pa31/tests/spec/nominal-unsize";

fn cases() -> TestResult<Vec<(String, String, bool, bool)>> {
    let mut rows = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let parts: Vec<_> = line.split('\t').collect();
        let [name, status, native] = parts.as_slice() else {
            return Err("nominal Unsize case columns".into());
        };
        let accepted = match *status {
            "0" => true,
            "1" => false,
            _ => return Err("nominal Unsize outcome".into()),
        };
        rows.push((
            name.trim_end_matches(".rs").to_owned(),
            crate::resolve::fixture(&format!("{PACKET}/{name}"))?,
            accepted,
            *native == "execute",
        ));
    }
    assert_eq!(rows.len(), 44);
    Ok(rows)
}

#[test]
fn nominal_unsize_goals_prove_real_tail_arguments_and_input_debts() -> TestResult {
    let mut failures = Vec::new();
    for (name, source, accepted, _) in cases()? {
        match (accepted, lower(&source)) {
            (true, Ok(_)) => {}
            (false, Err(error))
                if ![
                    "not implemented",
                    "budget",
                    "exhausted",
                    "invalid",
                    "normalization",
                ]
                .iter()
                .any(|fence| error.to_string().contains(fence)) => {}
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|error| error.to_string(), |_| "accepted".into())
            )),
        }
    }
    lower(&crate::resolve::fixture(
        "pa31/tests/spec/unsizing-roles/nominal-unsize-required.rs",
    )?)?;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn nominal_unsize_work_follows_calls_and_ignores_unrelated_types() -> TestResult {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut ordinary = None;
        for noise in [0, 128] {
            let source =
                crate::resolve::fixture(&format!("{PACKET}/width-{width}-noise-{noise}.rs"))?;
            let report = crate::resolve::source_provider(&source, "nominal_unsize_work")?;
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
            println!("nominal Unsize width={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(actual <= 4 * old + 1024, "{before:?}->{work:?}");
                    }
                }
                previous = Some(work);
                ordinary = Some(work);
            } else {
                assert_eq!(ordinary, Some(work), "unrelated type work");
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and host C linker"]
fn nominal_unsize_conversions_execute_real_fields_metadata_and_dispatch() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join("obj/parallel/nominal-unsize/native");
    std::fs::create_dir_all(&folder)?;
    let client = crate::resolve::fixture(&format!("{PACKET}/observer.c"))?;
    let rows: Vec<_> = cases()?.into_iter().filter(|row| row.3).collect();
    assert_eq!(rows.len(), 29);
    for (name, source, _, _) in rows {
        let program = lower(&source)?;
        std::fs::write(folder.join(format!("{name}.rs")), source)?;
        crate::ffi_program::check(root, &folder, &name, &program, &client, "9 23\n")?;
    }
    Ok(())
}
