//! Source queries retain checked calls, actual configuration and ordinary-name opposition.
use crate::resolve as test_source;
use rustgm::{lower, pipeline};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn source(packet: &str, name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/{packet}/{name}.rs"
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
    contract(
        "static-knowledge",
        &[
            ("direct", true),
            ("side-effect", true),
            ("ordinary", true),
            ("noncopy", false),
            ("true-body", true),
            ("native", true),
        ],
    )
}
#[test]
fn declared_overflow_query_keeps_ctfe_checked_profile_and_ordinary_name() -> TestResult {
    contract(
        "overflow-query",
        &[
            ("direct", true),
            ("ordinary", true),
            ("inherited", true),
            ("native", true),
            ("wrong-result", false),
        ],
    )
}
fn contract(packet: &str, cases: &[(&str, bool)]) -> TestResult {
    for &(name, accepted) in cases {
        let result = program(&source(packet, name)?);
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
    native("static-knowledge", "0 7 1\n")
}
#[test]
#[ignore = "requires the locked CPPGM backend and host C compiler"]
fn actual_overflow_query_native_keeps_const_and_checked_runtime_policy() -> TestResult {
    native("overflow-query", "1 1 1\n")
}
fn native(packet: &str, output: &str) -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!("obj/parallel/{packet}/native"));
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "knowledge",
        &program(&source(packet, "native")?)?,
        &test_source::fixture(&format!("pa31/tests/spec/{packet}/observer.c"))?,
        output,
    )?;
    Ok(())
}
