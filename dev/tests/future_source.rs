//! Future source roles retain ordinary associated projection semantics.
use crate::resolve as test_source;
use rustgm::pipeline;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn actual_future_roles_preserve_unused_alternatives_and_projection_consumers() -> TestResult {
    let manifest = test_source::fixture("pa31/tests/spec/future-source/cases.tsv")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in manifest.lines() {
        let (path, status) = line.split_once('\t').ok_or("two case columns")?;
        let result = (|| {
            let source = test_source::fixture(path)?;
            let report = test_source::source_provider(&source, "future_source")?;
            Ok::<_, Box<dyn std::error::Error>>(pipeline::analyze_source(&report)?)
        })();
        match (status, result) {
            ("0", Ok(_)) => {}
            ("0", Err(error)) => failures.push(format!("{path}: {error}")),
            ("1", Ok(_)) => failures.push(format!("accepted {path}")),
            ("1", Err(error)) => {
                if error
                    .to_string()
                    .contains("semantic contract is not implemented")
                {
                    failures.push(format!("{path}: {error}"));
                }
            }
            _ => return Err("invalid declared outcome".into()),
        }
        count += 1;
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(count, 14);
    Ok(())
}

#[test]
fn source_future_output_projection_has_an_ordinary_native_body() -> TestResult {
    let source = test_source::fixture("pa31/tests/spec/future-source/projection-consumer.rs")?;
    let report = test_source::source_provider(&source, "future_native")?;
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
    assert!(
        program
            .symbols
            .iter()
            .any(|symbol| symbol.name == "observe" && symbol.body.is_some())
    );
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/async-signatures/native");
    std::fs::create_dir_all(&folder)?;
    rustgm::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("source.lowir"))?,
    )?;
    Ok(())
}
