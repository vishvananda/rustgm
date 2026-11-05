//! Source `FusedIterator` retains ordinary implementation and supertrait evidence.
use crate::resolve as source;
use rustgm::pipeline;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn fused_source_roles_and_consumers_match_actual_language_outcomes() -> Result {
    let manifest = source::fixture("pa31/tests/spec/fused-iterator/cases.tsv")?;
    let mut failures = Vec::new();
    let mut count = 0;
    for line in manifest.lines() {
        let (name, status) = line.split_once('\t').ok_or("two case columns")?;
        let text = source::fixture(&format!("pa31/tests/spec/fused-iterator/{name}"))?;
        let result = source::source_provider(&text, "fused_source").and_then(|report| {
            let pipeline::Analysis {
                mut facts,
                prepared,
            } = pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
            rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
                .map_err(|e| e.to_string())
        });
        match (status, result) {
            ("0", Ok(())) => {}
            ("1", Err(error))
                if ![
                    "semantic contract is not implemented",
                    "invalid or exhausted",
                    "unknown feature",
                    "not implemented",
                ]
                .iter()
                .any(|fence| error.contains(fence)) => {}
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|e| e, |()| "accepted".into())
            )),
        }
        count += 1;
    }
    assert_eq!(count, 17);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn fused_iterator_calls_execute_real_next_and_exhaustion_bodies() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/fused-iterator-v1/native");
    std::fs::create_dir_all(&folder)?;
    for (name, observer, expected) in [
        ("native", "observe.c", "23 0 0\n"),
        ("ordinary", "ordinary.c", "23\n"),
    ] {
        let text = source::fixture(&format!("pa31/tests/spec/fused-iterator/{name}.rs"))?;
        let report = source::source_provider(&text, "fused_native")?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        let program = rustgm::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            rustgm::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        crate::ffi_program::check(
            &root,
            &folder,
            name,
            &program,
            &source::fixture(&format!("pa31/tests/spec/fused-iterator/{observer}"))?,
            expected,
        )?;
    }
    Ok(())
}
