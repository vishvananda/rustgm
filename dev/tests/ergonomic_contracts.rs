//! Ordinary source contracts do not fabricate copy, clone or capture operations.
use crate::resolve as test_source;
use rustgm::{lower, pipeline};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn fixture(name: &str) -> std::io::Result<String> {
    test_source::fixture(&format!("pa31/tests/spec/use-cloned/{name}.rs"))
}

fn program(source: &str) -> TestResult<rustgm::lowir::program::Program> {
    let report = test_source::source_provider(source, "ergonomic_source")?;
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
fn actual_trait_headers_bounds_members_and_implementation_rules() -> TestResult {
    let cases = test_source::fixture("pa31/tests/spec/use-cloned/cases.tsv")?;
    let mut checked = 0;
    for line in cases.lines() {
        let (path, status) = line.split_once('\t').ok_or("two outcome columns")?;
        let source = test_source::fixture(path)?;
        let outcome = program(&source)
            .map(|_| ())
            .map_err(|error| error.to_string());
        match status {
            "0" => outcome.map_err(|error| format!("{path}: {error}"))?,
            "1" => {
                let error = outcome.err().ok_or_else(|| format!("accepted {path}"))?;
                assert!(
                    !error.contains("semantic contract is not implemented"),
                    "{path}: {error}"
                );
            }
            _ => return Err(format!("invalid prediction: {line}").into()),
        }
        checked += 1;
    }
    assert_eq!(checked, 27);
    Ok(())
}

#[test]
fn ordinary_clone_and_provided_source_bodies_reach_lowir() -> TestResult {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/use-cloned-v1/native");
    std::fs::create_dir_all(&folder)?;
    for name in ["source-native", "provided-native"] {
        let source = fixture(name)?;
        let program = program(&source)?;
        for function in ["selected", "ordinary"] {
            assert!(
                program
                    .symbols
                    .iter()
                    .any(|symbol| symbol.name == function && symbol.body.is_some()),
                "{name}: {function}"
            );
        }
        std::fs::write(folder.join(format!("{name}.rs")), source)?;
        rustgm::lowir::writer::write(
            &program,
            &mut std::fs::File::create(folder.join(format!("{name}.lowir")))?,
        )?;
    }
    Ok(())
}
