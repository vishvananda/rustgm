//! Unchecked reinterpretation preserves source identity without a typing size premise.
use crate::{
    body,
    resolve::model::DefId,
    sema::values::{Reader, View},
    support::bindings::SourceIntrinsic,
    types::instance::Instance,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa28/tests/spec/source-transmute-unchecked/{name}.rs"
    ))?)
}
#[test]
fn unchecked_instances_type_without_ordinary_transmute_size_premises() -> TestResult {
    let text = source("generic")?;
    let report = crate::test_source::source_provider(&text, "unchecked_generic")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let facts = crate::sema::bodies::analyze(&report, facts)?;
    let catalog = facts.source_bindings.as_deref().ok_or("source catalog")?;
    let mut selected = 0;
    for function in &facts.functions {
        let name = report
            .package
            .names
            .get(report.resolution.definitions[function.definition.0 as usize].name);
        if !matches!(name, "generic" | "unequal") {
            continue;
        }
        // These are declaration/body typing facts, not unparameterized storage proof.
        let root = function.body.ok_or("actual generic source body")?;
        let callee = typed_callee(&report, root)?;
        let ty = facts
            .node(callee)
            .and_then(|fact| fact.ty)
            .ok_or("typed source callee")?;
        let instance = Instance::new(&facts.types, ty)?;
        let actual = catalog
            .reinterpretation_selection(&facts, instance, root.into())?
            .ok_or("unchecked descriptor")?;
        assert_ne!(actual.source, actual.destination);
        assert_eq!(
            catalog.intrinsic_kind(actual.definition)?,
            Some(SourceIntrinsic::TransmuteUnchecked)
        );
        assert!(
            catalog
                .transmute_selection(&facts, instance, root.into())?
                .is_none()
        );
        catalog.require_transmute_report(&report, &facts, &actual)?;
        let foreign = crate::test_source::source_provider(&text, "unchecked_generic")?;
        assert!(
            catalog
                .require_transmute_report(&foreign, &facts, &actual)
                .is_err()
        );
        selected += 1;
    }
    assert_eq!(selected, 2);
    Ok(())
}
#[test]
fn unchecked_execution_retains_validity_unsafe_and_declaration_gates() -> TestResult {
    for (name, needle) in [
        ("invalid-bool", "bool"),
        ("unequal-const", "sizes"),
        ("missing-unsafe", "unsafe"),
        ("wrong-signature", "source transmute"),
        ("missing-gate", "explicit source features"),
        ("const-use-nonconst", "const"),
    ] {
        let error = match crate::test_source::source_provider(&source(name)?, "unchecked_opponent")
        {
            Ok(report) => crate::pipeline::analyze_source(&report)
                .err()
                .map(|error| error.to_string()),
            Err(error) => Some(error),
        }
        .ok_or_else(|| format!("{name}: missing expected denial"))?;
        assert!(error.contains(needle), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn unequal_runtime_execution_is_a_profile_boundary_after_successful_typing() -> TestResult {
    let report = crate::test_source::source_provider(&source("generic")?, "unchecked_unequal")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let definition = named(&report, "unequal")?;
    let result = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[definition],
        },
        prepared,
    );
    let error = result
        .err()
        .ok_or("unequal runtime operation unexpectedly lowered")?;
    assert!(
        error.to_string().contains("explicit execution profile"),
        "{error}"
    );
    Ok(())
}
#[test]
fn ordinary_unchecked_spelling_executes_source_and_keeps_compact_records() -> TestResult {
    let report = crate::test_source::source_provider(&source("lookalike")?, "unchecked_ordinary")?;
    let crate::pipeline::Analysis { facts, .. } = crate::pipeline::analyze_source(&report)?;
    let definition = named(&report, "VALUE")?;
    let reader = Reader::new(&facts);
    let value = reader
        .definition(definition)
        .ok_or("ordinary source constant")?;
    assert!(matches!(
        reader.view(value)?,
        View::Scalar(crate::sema::model::Scalar::Integer(7))
    ));
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<body::ir::Body>(), 456);
    assert_eq!(size_of::<super::TransmuteSelection>(), 16);
    assert!(size_of::<SourceIntrinsic>() <= 2);
    eprintln!("unchecked intrinsic width {}", size_of::<SourceIntrinsic>());
    Ok(())
}
#[test]
fn original_nonzero_new_and_get_use_actual_projection_and_unchecked_calls() -> TestResult {
    let text =
        crate::test_source::fixture("pa31/tests/spec/source-nonzero-methods/source-runtime.rs")?;
    let report = crate::test_source::source_provider(&text, "nonzero_methods")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let reader = Reader::new(&facts);
    let value = reader
        .definition(named(&report, "GET")?)
        .ok_or("actual source get")?;
    assert!(matches!(
        reader.view(value)?,
        View::Scalar(crate::sema::model::Scalar::Integer(255))
    ));
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in [
        "nz_new",
        "nz_get",
        "const_none",
        "const_one",
        "const_max",
        "const_get",
    ] {
        assert!(
            program
                .symbols
                .iter()
                .any(|s| s.name == name && s.body.is_some()),
            "{name}"
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-transmute-unchecked-v1/candidate-v1");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), text)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    let invalid =
        crate::test_source::fixture("pa31/tests/spec/source-nonzero-methods/some-zero.rs")?;
    let bad = crate::test_source::source_provider(&invalid, "nonzero_invalid")?;
    let error = crate::pipeline::analyze_source(&bad)
        .err()
        .ok_or("Some0 accepted")?;
    assert!(error.to_string().contains("scalar pattern"), "{error}");
    Ok(())
}
#[test]
fn unchecked_source_declarations_cannot_silently_enter_legacy_metadata() -> TestResult {
    let text = r#"#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]
#[lang="pointee_sized"]trait PointeeSized{}
#[lang="meta_sized"]trait MetaSized:PointeeSized{}
#[lang="sized"]trait Sized:MetaSized{}
#[rustc_intrinsic]pub const unsafe fn transmute_unchecked<S,D>(value:S)->D;
"#;
    let report = crate::test_source::source_provider(text, "unchecked_export")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let identity = report
        .package
        .source_core()
        .ok_or("provider")?
        .current
        .clone();
    assert_eq!(
        crate::sema::crates::authored_export::Stage::begin(&report, &mut facts, &identity).err(),
        Some("source intrinsic declaration semantics require a wider metadata profile")
    );
    Ok(())
}
fn typed_callee(
    report: &crate::resolve::expansion::Report,
    mut root: crate::resolve::unit::NodeKey,
) -> TestResult<crate::resolve::unit::NodeKey> {
    use crate::syntax::ast::Kind;
    let ast = &report.package.unit(root.file).configured.ast;
    while let Kind::Block {
        tail: Some(tail), ..
    } = ast.node(root.node).kind
    {
        root.node = tail;
    }
    let Kind::Call { callee, .. } = ast.node(root.node).kind else {
        return Err("missing genuine source tail call".into());
    };
    root.node = callee;
    Ok(root)
}
fn named(report: &crate::resolve::expansion::Report, name: &str) -> TestResult<DefId> {
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| report.package.names.get(row.name) == name)
        .ok_or_else(|| format!("missing source {name}"))?;
    Ok(DefId(u32::try_from(index)?))
}
