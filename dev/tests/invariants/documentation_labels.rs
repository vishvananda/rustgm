//! Documentation labels preserve ordinary type, body, privacy and budget rules.
use super::*;
use crate::{pipeline, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa40/tests/spec/primitive-documentation/{name}.rs"
    ))?)
}
fn analyze(source: &str) -> TestResult<(Report, pipeline::Analysis)> {
    let report = test_source::resolve(source, true)?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn fully_checked(source: &str) -> TestResult {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(source)?;
    crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    Ok(())
}
#[test]
fn forms_owners_duplicates_and_macro_envelopes_match_independent_rust() -> TestResult {
    for name in [
        "bool",
        "cfg-removed",
        "empty",
        "escape",
        "inner-module",
        "macro-allowed-meta",
        "multiple-modules",
        "native",
        "raw",
        "semantic-type",
        "unicode",
        "unknown",
    ] {
        analyze(&fixture(name)?).map_err(|e| format!("{name}: {e}"))?;
    }
    for name in [
        "alias",
        "bool-value",
        "byte-string",
        "char",
        "constant",
        "duplicate-different",
        "duplicate-same",
        "enum",
        "field",
        "function",
        "impl",
        "inner-crate",
        "inner-outer-duplicate",
        "integer",
        "list",
        "macro-forwarded-item",
        "macro-no-allowance",
        "missing-gate",
        "static",
        "struct",
        "trait",
        "union",
        "unsafe-wrapper",
        "word",
    ] {
        let error = analyze(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(
            error.contains("primitive documentation") || error.contains("rustc_doc_primitive"),
            "{name}: {error}"
        );
        assert!(!error.contains("budget"), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn documentation_never_grants_primitive_identity_or_body_privileges() -> TestResult {
    let (report, analysis) = analyze(&fixture("semantic-type")?)?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == "Real")
        .ok_or("real source struct")?;
    let ty = analysis.facts.definitions[index].ok_or("actual struct type")?;
    assert!(matches!(
        analysis.facts.types.kind(ty),
        Some(crate::types::Kind::Adt(_, _))
    ));
    let layout = analysis.facts.layouts.get(ty).ok_or("real struct layout")?;
    assert_eq!((layout.size, layout.align), (16, 8));
    for (name, words) in [
        ("body-type-error", &["type"][..]),
        ("unsafe-denied", &["unsafe"]),
        ("const-denied", &["const"]),
        ("move-denied", &["move", "moved", "initialized"]),
        ("private-field", &["private"]),
    ] {
        let error = fully_checked(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(
            words.iter().any(|word| error.contains(word)),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn documentation_does_not_add_a_semantic_metadata_fence() -> TestResult {
    let (report, mut analysis) = analyze(&fixture("native")?)?;
    crate::sema::crates::export(
        &report,
        &mut analysis.facts,
        crate::support::crates::CrateIdentity::new("doc_module", "owned")?,
    )?;
    Ok(())
}
#[test]
fn per_owner_work_is_linear_storage_stays_cold_and_failures_are_sticky() -> TestResult {
    use std::fmt::Write;
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut input = String::from("#![feature(rustc_attrs)]");
        for i in 0..width {
            write!(input, "#[rustc_doc_primitive=\"bool\"]mod m{i}{{}}")?;
        }
        let (_, analysis) = analyze(&input)?;
        let work = analysis.facts.must_use_work();
        observations.push((work.contract_input_steps, work.contract_input_bytes));
        eprintln!(
            "primitive docs width={width} work={} bytes={}",
            work.contract_input_steps, work.contract_input_bytes
        );
        assert_eq!(work.retained_contracts, 0);
    }
    assert_eq!(observations[2].0, 4 * observations[1].0);
    assert_eq!(observations[2].1, 4 * observations[1].1);
    let (report, mut analysis) = analyze(&fixture("bool")?)?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .find(|d| report.package.names.get(d.name) == "docs")
        .ok_or("module")?;
    let key = definition.node.ok_or("actual module node")?;
    let unit = report.package.unit(key.file);
    let (attribute, forest) = unit
        .retained_attributes(0..unit.configured.effects.len())
        .find(|(attribute, forest)| {
            attribute.name(forest, unit.source.text()) == Some("rustc_doc_primitive")
        })
        .ok_or("actual attribute")?;
    analysis
        .facts
        .must_use_contracts
        .as_deref_mut()
        .ok_or("work ledger")?
        .work
        .contract_input_steps = 10_000_000;
    for _ in 0..2 {
        let error = consume(
            &report,
            &mut analysis.facts,
            key,
            attribute,
            forest,
            &mut report.macro_allowances(),
            &mut Draft::default(),
        )
        .err()
        .ok_or("exhausted producer resumed")?;
        assert!(error.message.contains("budget"), "{error}");
    }
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn original_documented_module_bodies_lower_for_the_native_observer() -> TestResult {
    let source = fixture("native")?;
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(&source)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/primitive-documentation-v1/candidate-native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
