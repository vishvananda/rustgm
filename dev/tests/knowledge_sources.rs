//! Actual optimization hints retain argument evaluation, Copy and ordinary-name opposition.
use crate::resolve as test_source;
use rustgm::{lower, pipeline};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/static-knowledge/{name}.rs"
    ))?)
}
fn program(text: &str) -> TestResult<rustgm::lowir::program::Program> {
    let report = test_source::source_provider(text, "static_knowledge")?;
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
fn declared_value_knowledge_keeps_source_authority_and_copy_obligations() -> TestResult {
    for (name, accepted) in [
        ("direct", true),
        ("side-effect", true),
        ("ordinary", true),
        ("noncopy", false),
        ("true-body", true),
        ("native", true),
    ] {
        let result = program(&source(name)?);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {:?}",
            result.err().map(|error| error.to_string())
        );
    }
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM backend and host C compiler"]
fn actual_value_knowledge_native_preserves_single_evaluation_and_both_hint_branches() -> TestResult
{
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/static-knowledge/native");
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "knowledge",
        &program(&source("native")?)?,
        &test_source::fixture("pa31/tests/spec/static-knowledge/observer.c")?,
        "0 7 1\n",
    )?;
    Ok(())
}
