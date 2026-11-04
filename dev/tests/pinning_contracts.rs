//! Real Pin declarations preserve ordinary bodies, ownership and source permissions.
use crate::resolve as test_source;
use rustgm::{lower, pipeline};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn program(source: &str) -> TestResult<rustgm::lowir::program::Program> {
    let report = test_source::source_provider(source, "pin_source")?;
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
fn actual_pin_roles_keep_source_body_and_ordinary_ownership_contracts() -> TestResult {
    let manifest = test_source::fixture("pa31/tests/spec/pin-source/cases.tsv")?;
    let mut count = 0;
    for line in manifest.lines() {
        let (path, status) = line.split_once('\t').ok_or("two case columns")?;
        let result = program(&test_source::fixture(path)?);
        match status {
            "0" => {
                result.map_err(|error| format!("{path}: {error}"))?;
            }
            "1" => {
                let error = result.err().ok_or_else(|| format!("accepted {path}"))?;
                assert!(
                    !error
                        .to_string()
                        .contains("semantic contract is not implemented"),
                    "{path}: {error}"
                );
            }
            _ => return Err("invalid declared outcome".into()),
        }
        count += 1;
    }
    assert_eq!(count, 25);
    Ok(())
}

#[test]
fn source_pin_constructor_mutation_and_field_drop_reach_native_input() -> TestResult {
    let source = test_source::fixture("pa31/tests/spec/pin-source/native-ordinary-body.rs")?;
    let program = program(&source)?;
    for name in ["scalar", "borrowed", "owned"] {
        assert!(
            program
                .symbols
                .iter()
                .any(|s| s.name == name && s.body.is_some()),
            "{name}"
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pin-source-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), source)?;
    rustgm::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("source.lowir"))?,
    )?;
    Ok(())
}
