//! Weak runtime tags retain actual foreign declarations and ordinary call rules.
use crate::resolve as source;
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/weak-symbols";

fn report(name: &str) -> Result<rustgm::resolve::expansion::Report> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    Ok(source::source_provider_config(
        &source::fixture(&format!("{PACKET}/{name}.rs"))?,
        "weak_symbols",
        &config,
    )?)
}
fn program() -> Result<rustgm::lowir::program::Program> {
    let report = report("native-v2")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
fn analyze(name: &str) -> Result<pipeline::Analysis> {
    pipeline::analyze_source(&report(name)?).map_err(Into::into)
}
#[test]
fn actual_foreign_headers_preserve_unused_ordinary_legality() -> Result {
    for name in [
        "source-v2",
        "safe-header-v2",
        "changed-prototype-v2",
        "rust-abi-v2",
        "lifetime-header-v2",
        "ordinary-call-v2",
    ] {
        analyze(name)?;
    }
    Ok(())
}
#[test]
fn source_unsafe_type_const_generic_duplicate_and_target_checks_remain() -> Result {
    for (name, causes) in [
        ("safe-call-v2", &["unsafe"][..]),
        ("wrong-argument-v2", &["type", "mismatch"][..]),
        ("wrong-result-v2", &["type", "mismatch"][..]),
        ("const-call-v2", &["const"][..]),
        ("generic-header-v2", &["foreign functions cannot"][..]),
        ("duplicate-v2", &["duplicate"][..]),
        ("wrong-target-v2", &["declaration target"][..]),
        ("defined-item-v2", &["declaration target"][..]),
    ] {
        let error = match analyze(name) {
            Err(error) => error.to_string(),
            Ok(_) => return Err(format!("{name} unexpectedly accepted").into()),
        };
        assert!(
            causes.iter().any(|cause| error.contains(cause)),
            "{name}: {error}"
        );
        assert!(
            !error.contains("semantic contract is not implemented"),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn real_source_foreign_calls_reach_existing_abi_lowering() -> Result {
    let program = program()?;
    assert_eq!(
        program
            .symbols
            .iter()
            .filter(|symbol| symbol.body.is_some())
            .count(),
        6
    );
    Ok(())
}
#[test]
fn actual_c_void_runtime_and_builtin_macro_intervals_are_retained() -> Result {
    let original = source::fixture(&format!("{PACKET}/original-ffi.rs"))?;
    let start = original
        .find("#[doc = include_str!(\"c_void.md\")]")
        .ok_or("c_void interval")?;
    let end = start + original[start..].find("\n}\n").ok_or("c_void end")? + 3;
    let block = original[original
        .find("    unsafe extern \"C\" {")
        .ok_or("runtime block")?..]
        .trim_end();
    let block = block
        .strip_suffix('}')
        .ok_or("runtime module end")?
        .trim_end();
    let macro_ = source::fixture(&format!("{PACKET}/original-include-str.rs"))?;
    for name in ["source-v2", "native-v2"] {
        let actual = source::fixture(&format!("{PACKET}/{name}.rs"))?;
        assert!(actual.contains(&original[start..end]), "{name}");
        assert!(actual.contains(block), "{name}");
        assert!(actual.contains(&macro_), "{name}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn ordinary_foreign_calls_preserve_real_runtime_effects() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/weak-symbols/native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "weak-symbols",
        &program()?,
        &source::fixture(&format!("{PACKET}/observer.c"))?,
        "runtime 1 1 1 1 1 5\n",
    )?;
    Ok(())
}
