//! Contract tags retain ordinary generic bodies, constness and call obligations.
use super::*;
use crate::{pipeline, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/contract-helpers/{name}.rs"
    ))?)
}
fn analyze(source: &str) -> TestResult<(Report, pipeline::Analysis)> {
    let report = test_source::source_provider(source, "contract_helpers")?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn lower(source: &str) -> TestResult<crate::lowir::program::Program> {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(source)?;
    Ok(crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn actual_contract_helpers_keep_generic_bodies_and_native_callable_values() -> TestResult {
    for name in [
        "native-item",
        "native-closure",
        "const-identity",
        "ordinary-nonconst",
        "tag-has-no-arity",
        "all-function-claims",
        "original-header-native",
        "original-header-const",
        "original-header-const-no-feature",
        "cfg-removed-claim",
        "reference-mention",
        "unsafe-declaration",
    ] {
        lower(&fixture(name)?).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn contract_tags_do_not_grant_constness_or_accept_wrong_owners_and_predicates() -> TestResult {
    for name in [
        "nonconst-const-use",
        "wrong-struct-owner",
        "wrong-trait-member",
        "wrong-predicate",
        "duplicate-claim",
        "lookalike-copy",
        "provided-member",
        "inherent-member",
        "foreign-declaration",
        "unsafe-call",
    ] {
        let error = lower(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("{name} gained authority"))?;
        assert!(
            !error.to_string().contains("not implemented"),
            "{name}: {error}"
        );
        assert!(!error.to_string().contains("budget"), "{name}: {error}");
        assert!(
            !error.to_string().contains("unsupported source language"),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn actual_closure_marker_fields_keep_source_impl_and_lifetime_requirements() -> TestResult {
    for name in ["captured-bool", "captured-nominal"] {
        lower(&fixture(name)?).map_err(|e| format!("{name}: {e}"))?;
    }
    for (name, category) in [
        ("captured-noncopy", "trait"),
        ("captured-borrow-escape", "lifetime"),
        ("captured-mutable-reference", "trait"),
    ] {
        let error = lower(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("{name} gained marker proof"))?;
        assert!(error.to_string().contains(category), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn static_call_evidence_keeps_region_debts_and_rejects_escaping_arguments() -> TestResult {
    for name in ["plain-static", "plain-unbounded"] {
        lower(&fixture(name)?)?;
    }
    for name in [
        "untagged-item",
        "untagged-no-static",
        "untagged-no-copy",
        "untagged-plain",
    ] {
        lower(&fixture(name)?)?;
    }
    let error = lower(&fixture("plain-static-escape")?)
        .err()
        .ok_or("escaping argument gained static proof")?;
    assert!(
        error.to_string().contains("lifetime") || error.to_string().contains("outlive"),
        "{error}"
    );
    Ok(())
}
#[test]
fn declaration_roles_keep_actual_function_identity_and_ordinary_record_sizes() -> TestResult {
    let (report, analysis) = analyze(&fixture("all-function-claims")?)?;
    let catalog = analysis
        .facts
        .source_bindings
        .as_deref()
        .ok_or("actual catalog")?;
    for role in [
        LangItem::ContractBuildCheckEnsures,
        LangItem::ContractCheckRequires,
        LangItem::ContractCheckEnsures,
    ] {
        let declaration = catalog.language(role).ok_or("actual source role")?;
        assert!(declaration.parent.is_none());
        assert_eq!(
            report.resolution.definitions[declaration.definition.0 as usize].kind,
            crate::resolve::model::DefKind::Function
        );
        assert_eq!(
            report
                .resolution
                .source_bindings
                .as_deref()
                .and_then(|r| r.language(role)),
            Some(declaration)
        );
        assert!(
            analysis
                .facts
                .declaration_header(declaration.definition)?
                .is_some()
        );
    }
    assert_eq!(LangItem::Iterator.index(), 72);
    assert_eq!(LangItem::ContractBuildCheckEnsures.index(), 75);
    assert_eq!(LangItem::ContractCheckEnsures.index(), 77);
    assert_eq!(LangItem::ALL.len(), 209);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<Catalog>(), 3752);
    assert_eq!(size_of::<crate::resolve::bindings::Reservations>(), 5208);
    println!(
        "contract-role layouts Catalog={} Reservations={} banks={:?}",
        size_of::<Catalog>(),
        size_of::<crate::resolve::bindings::Reservations>(),
        Catalog::binding_layouts()
    );
    Ok(())
}
#[test]
fn capture_marker_work_scales_with_stored_fields_without_growing_search_frames() -> TestResult {
    use std::fmt::Write;
    let full = fixture("native-closure")?;
    let prefix = full
        .split_once("#[unsafe(no_mangle)]")
        .ok_or("actual source foundation")?
        .0;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = format!("{prefix}pub fn formation(){{");
        for index in 0..width {
            write!(source, "let a{index}=true;")?;
        }
        source.push_str("let _condition=make_predicate::<u8,_>(move|_:&u8|{");
        for index in 0..width {
            write!(source, "if a{index}{{}}else{{return false;}}")?;
        }
        source.push_str("true});}");
        let (_, analysis) = analyze(&source)?;
        assert_eq!(
            analysis
                .facts
                .closures
                .environments
                .values()
                .map(|e| e.field_types.len())
                .max(),
            Some(width)
        );
        let work = analysis.facts.body_work.inference_steps;
        if let Some(before) = previous {
            assert!(work <= before * 6, "width={width}: {work} > {before} * 6");
        }
        previous = Some(work);
        println!(
            "source-capture width={width} inference={work} canonical={} selections={} layouts={:?}",
            analysis.facts.body_work.canonical_steps,
            analysis.facts.body_work.trait_selections,
            crate::sema::traits::selection::callable_layouts()
        );
    }
    Ok(())
}
#[test]
fn actual_contract_builders_emit_native_exports_from_unchanged_source() -> TestResult {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-contract-helpers-v1/candidate-native");
    for name in [
        "native-item",
        "native-closure",
        "const-identity",
        "captured-bool",
        "captured-nominal",
        "original-header-native",
        "original-header-const",
        "reference-mention",
    ] {
        let source = fixture(name)?;
        let program = lower(&source)?;
        let out = folder.join(name);
        std::fs::create_dir_all(&out)?;
        std::fs::write(out.join("source.rs"), source)?;
        crate::lowir::writer::write(
            &program,
            &mut std::fs::File::create(out.join("provider.lowir"))?,
        )?;
    }
    Ok(())
}
