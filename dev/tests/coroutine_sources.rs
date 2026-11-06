//! Real coroutine declarations retain ordinary enum, implementation and body proof.
use crate::resolve as source;
use rustgm::pipeline;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn ordinary_coroutine_contracts_keep_real_headers_bodies_and_failures() -> Result {
    let cases = source::fixture("pa31/tests/spec/source-coroutines/cases.tsv")?;
    let mut failures = Vec::new();
    let mut count = 0;
    for row in cases.lines() {
        let (path, expected) = row.split_once('\t').ok_or("coroutine case columns")?;
        let text = source::fixture(path)?;
        let result = source::source_provider(&text, "coroutine_source").and_then(|report| {
            let pipeline::Analysis {
                mut facts,
                prepared,
            } = pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
            rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
                .map_err(|e| e.to_string())
        });
        match (expected, result) {
            ("0", Ok(())) => {}
            ("1", Err(error))
                if ![
                    "not implemented",
                    "budget",
                    "invalid or exhausted",
                    "unknown feature",
                    "unsupported active source-core feature",
                    "changed its actual",
                    "invalid lifetime constraint identity",
                ]
                .iter()
                .any(|cause| error.contains(cause)) => {}
            (_, result) => failures.push(format!("{path}: {result:?}")),
        }
        count += 1;
    }
    assert_eq!(count, 12);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn manual_coroutine_resume_executes_actual_source_state_and_variants() -> Result {
    let text = source::fixture("pa31/tests/spec/source-coroutines/native.rs")?;
    let report = source::source_provider(&text, "coroutine_native")?;
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
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/coroutine-source-contracts/candidate-native");
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "manual",
        &program,
        &source::fixture("pa31/tests/spec/source-coroutines/observer.c")?,
        "23 47\n",
    )?;
    Ok(())
}
