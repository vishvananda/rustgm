//! Real provided destructor bodies keep their source owner and normal body proof.
use crate::resolve as source;
use rustgm::pipeline;
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn complete_drop_source_and_consumers_match_pinned_language_outcomes() -> Result {
    let manifest = source::fixture("pa31/tests/spec/full-drop-source/cases.tsv")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in manifest.lines() {
        let (name, expected) = line.split_once('\t').ok_or("two outcome columns")?;
        let text = source::fixture(&format!("pa31/tests/spec/full-drop-source/{name}"))?;
        let result = source::source_provider(&text, "provided_drop").and_then(|report| {
            let pipeline::Analysis {
                mut facts,
                prepared,
            } = pipeline::analyze_source(&report).map_err(|error| error.to_string())?;
            rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
                .map_err(|error| error.to_string())
        });
        match (expected, result) {
            ("0", Ok(())) => {}
            ("1", Err(error))
                if ![
                    "not implemented",
                    "unsupported",
                    "invalid or exhausted",
                    "unknown feature",
                ]
                .iter()
                .any(|fence| error.contains(fence)) => {}
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|error| error, |()| "accepted".into())
            )),
        }
        count += 1;
    }
    assert_eq!(count, 12);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a POSIX C compiler"]
fn full_drop_source_executes_provided_and_override_bodies() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/full-drop-source-v1/native");
    std::fs::create_dir_all(&folder)?;
    let observer = source::fixture("pa31/tests/spec/full-drop-source/observe.c")?;
    for (name, expected) in [
        ("drop", "23 7\n"),
        ("pin-drop", "23 7\n"),
        ("generic-drop", "17 7\n"),
        ("generic-pin", "19 7\n"),
    ] {
        let text = source::fixture(&format!("pa31/tests/spec/full-drop-source/{name}.rs"))?;
        let report = source::source_provider(&text, "full_drop_native")?;
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
        crate::ffi_program::check(&root, &folder, name, &program, &observer, expected)?;
    }
    Ok(())
}
