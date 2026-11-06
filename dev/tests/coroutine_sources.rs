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

fn inspect_source(text: &str) -> Result {
    let report = source::source_provider(text, "coroutine_original")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())?;
    Ok(())
}

#[test]
fn complete_original_coroutine_derives_and_forwarders_keep_real_types_and_lifetimes() -> Result {
    let original = source::fixture("pa31/tests/spec/source-coroutines/full/original.rs")?;
    let cases = source::fixture("pa31/tests/spec/source-coroutines/full/cases.tsv")?;
    let mut failures = Vec::new();
    let mut count = 0;
    for row in cases.lines() {
        let (path, expected) = row.split_once('\t').ok_or("full coroutine columns")?;
        let text = source::fixture(path)?;
        assert!(text.contains(&original), "{path}: changed original module");
        let result = inspect_source(&text).map_err(|error| error.to_string());
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
    assert_eq!(count, 9);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn comparison_diagnostic_hints_do_not_supply_operator_language_roles() -> Result {
    let text = source::fixture("pa31/tests/spec/source-coroutines/full/only-hints.rs")?;
    let report = source::source_provider(&text, "comparison_hints")?;
    let reserved = report
        .resolution
        .source_bindings
        .as_deref()
        .ok_or("source reservations")?;
    for role in [
        rustgm::support::bindings::LangItem::PartialEq,
        rustgm::support::bindings::LangItem::PartialOrd,
    ] {
        assert!(reserved.language(role).is_none());
    }
    inspect_source(&text)?;
    let bad = source::fixture("pa31/tests/spec/source-coroutines/full/hint-not-operation.rs")?;
    assert!(inspect_source(&bad).is_err());
    Ok(())
}

#[test]
fn fixed_factory_result_unsizing_retains_its_required_nominal_validation_gap() -> Result {
    let text = source::fixture("pa31/tests/spec/source-coroutines/full/result-coercion.rs")?;
    let error = inspect_source(&text).err().ok_or("nominal boundary")?;
    assert!(
        error
            .to_string()
            .contains("source CoerceUnsized nominal implementation validation is not implemented"),
        "{error}"
    );
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn original_pin_and_reference_forwarders_execute_actual_provided_bodies() -> Result {
    let text = source::fixture("pa31/tests/spec/source-coroutines/full/forwarders.rs")?;
    let report = source::source_provider(&text, "coroutine_forwarders")?;
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
    let folder = root.join("obj/parallel/coroutine-full-module/candidate-native");
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "original",
        &program,
        &source::fixture("pa31/tests/spec/source-coroutines/full/observer.c")?,
        "23 47\n",
    )?;
    Ok(())
}

#[test]
fn diagnostic_only_comparison_default_body_keeps_ordinary_method_proof() -> Result {
    let text = source::fixture(
        "pa31/tests/spec/source-diagnostic-comparisons/diagnostic-only-default.rs",
    )?;
    let report = source::source_provider(&text, "diagnostic_comparison_default")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())?;
    Ok(())
}
