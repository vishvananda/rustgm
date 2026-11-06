//! Real source dereference dispatch preserves effects, places and ownership.
use crate::resolve as test_source;
use rustgm::{lower, pipeline};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn program(source: &str) -> TestResult<rustgm::lowir::program::Program> {
    let report = test_source::source_provider(source, "deref_source")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    Ok(lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn actual_source_deref_and_mutable_places_keep_real_bodies_and_proofs() -> TestResult {
    let mut count = 0;
    let mut failures = Vec::new();
    for line in test_source::fixture("pa31/tests/spec/dereference-source/cases.tsv")?.lines() {
        let (path, status) = line.split_once('\t').ok_or("case columns")?;
        let result = program(&test_source::fixture(path)?);
        let failure = match (status, result) {
            ("0", Err(e)) => Some(format!("{path}: {e}")),
            ("1", Ok(_)) => Some(format!("accepted {path}")),
            ("1", Err(e))
                if e.to_string()
                    .contains("semantic contract is not implemented") =>
            {
                Some(format!("{path}: {e}"))
            }
            ("0", Ok(_)) | ("1", Err(_)) => None,
            _ => return Err("invalid prediction".into()),
        };
        if let Some(failure) = failure {
            failures.push(failure);
        }
        count += 1;
    }
    assert_eq!(count, 32);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn ordinary_deref_method_effects_and_one_time_place_evaluation_reach_native() -> TestResult {
    let source = test_source::fixture("pa31/tests/spec/dereference-source/native.rs")?;
    let program = program(&source)?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/deref-source-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), source)?;
    rustgm::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("source.lowir"))?,
    )?;
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn nested_pin_borrow_mutates_the_actual_operand_before_directed_conversion() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/async-iterator-v1/nested-native");
    std::fs::create_dir_all(&folder)?;
    let text = test_source::fixture("pa31/tests/spec/dereference-source/nested-borrow.rs")?;
    crate::ffi_program::check(
        &root,
        &folder,
        "nested-borrow",
        &program(&text)?,
        &test_source::fixture("pa31/tests/spec/dereference-source/nested-observer.c")?,
        "23\n",
    )?;
    Ok(())
}
