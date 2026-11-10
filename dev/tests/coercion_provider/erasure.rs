//! Projected source erasure pays every ordinary obligation before producing evidence.
use crate::provider_traits::{TestResult, lower};
const PACKET: &str = "pa31/tests/spec/projected-erasure";

fn cases() -> TestResult<Vec<(String, String, bool)>> {
    let mut rows = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (name, status) = line.split_once('\t').ok_or("erasure columns")?;
        let accepted = match status {
            "0" => true,
            "1" => false,
            _ => return Err("invalid erasure outcome".into()),
        };
        rows.push((
            name.trim_end_matches(".rs").to_owned(),
            crate::resolve::fixture(&format!("{PACKET}/{name}"))?,
            accepted,
        ));
    }
    assert_eq!(rows.len(), 26);
    Ok(rows)
}

#[test]
fn projected_erasure_requires_real_traits_sizing_equalities_and_lifetimes() -> TestResult {
    let mut failures = Vec::new();
    for (name, source, accepted) in cases()? {
        match (accepted, lower(&source)) {
            (true, Ok(_)) => {}
            (false, Err(error))
                if ![
                    "normalization",
                    "not implemented",
                    "budget",
                    "exhausted",
                    "invalid",
                ]
                .iter()
                .any(|part| error.to_string().contains(part)) => {}
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|error| error.to_string(), |_| "accepted".into())
            )),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

// Rust accepts and executes these three original inputs. Their desired LowIR
// result remains acceptance; this records an independent generated-owner gap.
#[test]
fn generic_tuple_factories_retain_the_separate_cleanup_owner_requirement() -> TestResult {
    for name in ["intrinsic-field", "assumed-intrinsic", "generic-control"] {
        let source = crate::resolve::fixture(&format!("{PACKET}/required/{name}-factory.rs"))?;
        let error = lower(&source)
            .err()
            .ok_or("required constructor gap disappeared")?;
        assert!(
            error
                .to_string()
                .contains("defining cleanup function header is pending"),
            "{name}: {error}"
        );
    }
    Ok(())
}

#[test]
fn projected_erasure_work_follows_calls_without_unrelated_type_scans() -> TestResult {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut ordinary = None;
        for noise in [0, 128] {
            let source =
                crate::resolve::fixture(&format!("{PACKET}/width-{width}-noise-{noise}.rs"))?;
            let report = crate::resolve::source_provider(&source, "erasure_work")?;
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
            println!("projected erasure width={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(actual <= 4 * old + 1024, "{before:?}->{work:?}");
                    }
                }
                previous = Some(work);
                ordinary = Some(work);
            } else {
                assert_eq!(ordinary, Some(work), "unrelated type query work");
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and host C linker"]
fn projected_erasure_and_intrinsic_tags_execute_actual_virtual_values() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join("obj/parallel/projected-erasure/native");
    std::fs::create_dir_all(&folder)?;
    let client = crate::resolve::fixture(&format!("{PACKET}/observer.c"))?;
    let rows: Vec<_> = cases()?
        .into_iter()
        .filter(|(_, _, accepted)| *accepted)
        .collect();
    assert_eq!(rows.len(), 19);
    for (name, source, _) in rows {
        let program = lower(&source)?;
        std::fs::write(folder.join(format!("{name}.rs")), source)?;
        crate::ffi_program::check(root, &folder, &name, &program, &client, "9 23\n")?;
    }
    Ok(())
}
