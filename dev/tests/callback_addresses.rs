//! `FnPtr` source identity, builtin selection and address shims use real pointer types.
use crate::resolve as test_source;
use rustgm::{lower, pipeline};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn program(source: &str) -> TestResult<rustgm::lowir::program::Program> {
    let report = test_source::source_provider(source, "fn_ptr_source")?;
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
fn actual_pointer_contracts_preserve_source_and_builtin_distinctions() -> TestResult {
    let cases = test_source::fixture("pa31/tests/spec/function-pointers/cases.tsv")?;
    let mut checked = 0;
    for line in cases.lines() {
        let (path, status) = line.split_once('\t').ok_or("two case columns")?;
        let outcome = program(&test_source::fixture(path)?);
        match status {
            "0" => {
                outcome.map_err(|error| format!("{path}: {error}"))?;
            }
            "1" => {
                let error = outcome.err().ok_or_else(|| format!("accepted {path}"))?;
                assert!(
                    !error
                        .to_string()
                        .contains("semantic contract is not implemented"),
                    "{path}: {error}"
                );
            }
            _ => return Err("invalid prediction".into()),
        }
        checked += 1;
    }
    assert_eq!(checked, 38);
    Ok(())
}
#[test]
fn real_address_shims_reach_native_input_without_pointer_calls() -> TestResult {
    let source = test_source::fixture("pa31/tests/spec/function-pointers/native-address.rs")?;
    let program = program(&source)?;
    let folder =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../obj/parallel/fn-ptr-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), source)?;
    rustgm::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("source.lowir"))?,
    )?;
    for name in ["direct", "bounded", "unsafe_pointer", "reified"] {
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == name && symbol.body.is_some()),
            "{name}"
        );
    }
    Ok(())
}
