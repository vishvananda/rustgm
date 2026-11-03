//! Independently observed declaration targets remain separate from reflection operations.
use super::*;
use crate::{pipeline, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/type-reflection/setup-v3/{name}.rs"
    ))?)
}
fn analyze(name: &str) -> TestResult<(Report, pipeline::Analysis)> {
    let report = test_source::source_provider(&input(name)?, "type_reflection")?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
#[test]
fn actual_reflection_declarations_keep_unrestricted_formals_and_real_nominal_identity() -> TestResult
{
    for name in [
        "actual-pair",
        "info-unit",
        "info-tuple",
        "info-type",
        "info-lifetime",
        "info-const",
        "info-many",
        "id-unit",
        "id-tuple",
        "id-type",
        "id-lifetime",
        "id-const",
        "id-many",
        "same-spellings",
        "cfg-removed",
        "nested-owner",
    ] {
        let (_, analysis) = analyze(name)?;
        let catalog = analysis
            .facts
            .source_bindings
            .as_deref()
            .ok_or("source catalog")?;
        for role in [LangItem::TypeInfo, LangItem::TypeId] {
            let expected = name == "actual-pair"
                || role == LangItem::TypeInfo
                    && (name.starts_with("info-") || name == "nested-owner")
                || role == LangItem::TypeId && name.starts_with("id-");
            let declaration = catalog.validated_language(role)?;
            assert_eq!(declaration.is_some(), expected, "{name}, {role:?}");
            if let Some(declaration) = declaration {
                assert_eq!(
                    analysis
                        .facts
                        .aggregates
                        .get(ItemId(declaration.definition.0))
                        .ok_or("actual nominal")?
                        .kind,
                    Kind::Struct
                );
            }
        }
    }
    assert_eq!(LangItem::TypeInfo.index(), 85);
    assert_eq!(LangItem::TypeId.index(), 86);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<super::super::Catalog>(), 1680);
    assert_eq!(size_of::<crate::resolve::bindings::Reservations>(), 2648);
    Ok(())
}
#[test]
fn wrong_targets_duplicates_and_missing_features_cannot_complete_reflection_roles() -> TestResult {
    for name in [
        "info-wrong-enum",
        "info-wrong-union",
        "info-wrong-trait",
        "info-wrong-function",
        "info-duplicate",
        "id-wrong-enum",
        "id-wrong-union",
        "id-wrong-trait",
        "id-wrong-function",
        "id-duplicate",
        "missing-feature",
    ] {
        let error = analyze(name)
            .err()
            .ok_or("invalid declaration accepted")?
            .to_string();
        let expected = if name.ends_with("duplicate") {
            "duplicate source language item"
        } else if name == "missing-feature" {
            "lang requires its explicit source feature"
        } else {
            "required nominal kind"
        };
        assert!(error.contains(expected), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn source_pair_lowers_real_field_access_without_granting_reflection_intrinsics() -> TestResult {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyze("actual-pair")?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let declaration = catalog
        .validated_language(LangItem::TypeId)?
        .ok_or("actual identity declaration")?;
    let ty = facts.types.intern(crate::types::Kind::Adt(
        ItemId(declaration.definition.0),
        crate::types::GenericArgs::EMPTY,
    ))?;
    facts.layouts.demand(&facts.types, &facts.aggregates, ty)?;
    assert_eq!(facts.layouts.get(ty).ok_or("identity layout")?.size, 16);
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    assert!(
        program
            .symbols
            .iter()
            .any(|symbol| symbol.name == "observed_kind" && symbol.body.is_some())
    );
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/type-reflection-v1/native");
    std::fs::create_dir_all(&out)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("ordinary-fields.lowir"))?,
    )?;
    for authorized in [false, true] {
        let mut source = input("actual-pair")?;
        if authorized {
            source = source.replace("no_core,lang_items,", "no_core,lang_items,intrinsics,");
        }
        source.push_str(
            "\n#[rustc_intrinsic] const fn type_of(_:Identity)->Reflected{Reflected{kind:7}}\n",
        );
        let attempted = test_source::source_provider(&source, "type_reflection");
        let error = match attempted {
            Ok(report) => pipeline::analyze_source(&report)
                .err()
                .ok_or("unsupported reflection intrinsic accepted")?
                .to_string(),
            Err(error) => error,
        };
        let expected = if authorized {
            "unsupported source intrinsic"
        } else {
            "intrinsic binding requires explicit source features"
        };
        assert!(error.contains(expected), "{error}");
    }
    Ok(())
}
#[test]
fn source_only_roles_reject_unencoded_transport_duplicate_publication_and_failed_payment()
-> TestResult {
    let (report, pipeline::Analysis { mut facts, .. }) = analyze("actual-pair")?;
    let identity = report
        .package
        .source_core()
        .ok_or("provider")?
        .provider
        .clone();
    let error = crate::sema::crates::authored_export::Stage::begin_module_conversions(
        &report, &mut facts, &identity,
    )
    .err()
    .ok_or("unencoded reflection roles exported")?;
    assert_eq!(
        error,
        "source type reflection contract transport is not implemented"
    );
    assert!(complete(&report, &mut facts).is_err());
    assert!(
        header_attributes::charge_source_contract_query(
            &facts,
            crate::support::crates::MAX_COMPONENTS
        )
        .is_err()
    );
    assert!(complete(&report, &mut facts).is_err());
    Ok(())
}
