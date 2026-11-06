//! Validation tags keep actual builtin derivation distinct from ordinary traits.
use crate::resolve as source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/coerce-pointee-source";

#[test]
fn actual_validation_marker_preserves_headers_and_forbids_user_assertions() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for line in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (name, expected) = line.split_once('\t').ok_or("case columns")?;
        if name == "auto-header.rs" {
            // This independently valid input keeps its separate required-gap test.
            assert_eq!(expected, "0");
            continue;
        }
        let text = source::fixture(&format!("{PACKET}/{name}"))?;
        let result = source::source_provider(&text, "pointee_validation").and_then(|report| {
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
            rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
                .map_err(|e| e.to_string())
        });
        match (expected, result) {
            ("0", Ok(())) => {}
            ("1", Err(error))
                if ![
                    "not implemented",
                    "budget",
                    "exhausted",
                    "lost its",
                    "lacks its",
                ]
                .iter()
                .any(|fence| error.contains(fence)) => {}
            (_, result) => failures.push(format!("{name}: {result:?}")),
        }
        count += 1;
    }
    assert_eq!(count, 16);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn valid_custom_auto_header_retains_its_required_generic_trait_gap() -> Result {
    let text = source::fixture(&format!("{PACKET}/auto-header.rs"))?;
    let report = source::source_provider(&text, "auto_validation")?;
    let error = rustgm::pipeline::analyze_source(&report)
        .err()
        .ok_or("custom source auto traits still require their general implementation")?;
    assert!(
        error
            .to_string()
            .contains("custom source auto traits are not implemented"),
        "{error}"
    );
    Ok(())
}

#[test]
fn genuine_coerce_pointee_derive_retains_its_required_producer_gap() -> Result {
    let text = source::fixture(&format!("{PACKET}/actual-derive.rs"))?;
    let error = source::source_provider(&text, "actual_coerce_derive")
        .err()
        .ok_or("actual builtin derive producer still requires implementation")?;
    assert!(
        error.contains("builtin derive macro execution is not implemented"),
        "{error}"
    );
    Ok(())
}

#[test]
fn existing_const_parameter_derive_still_checks_and_emits_its_actual_body() -> Result {
    let text = source::fixture("pa31/tests/spec/const-parameter-derive/actual-source.rs")?;
    let report = source::source_provider(&text, "const_parameter_control")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
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
            .any(|symbol| { symbol.name == "observed" && symbol.body.is_some() })
    );
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn ordinary_same_spelling_trait_retains_real_native_method_dispatch() -> Result {
    let text = source::fixture(&format!("{PACKET}/ordinary-native.rs"))?;
    let report = source::source_provider(&text, "ordinary_pointee_trait")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
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
    let folder = root.join("obj/parallel/coerce-pointee-contracts/candidate-native");
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "ordinary_marker",
        &program,
        &source::fixture(&format!("{PACKET}/observer.c"))?,
        "23 47\n",
    )?;
    Ok(())
}
