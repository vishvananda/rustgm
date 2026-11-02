//! Limits constrain genuine substituted SIMD storage, independent of syntax spelling.
use crate::{
    pipeline, test_source,
    types::{self, Kind as Type, TypeId},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa41/tests/spec/simd-lane-limit/{name}.rs"
    ))?)
}
fn analyze(source: &str) -> TestResult<(crate::resolve::expansion::Report, pipeline::Analysis)> {
    let report = if source.contains("#![no_core]") {
        test_source::source_provider(source, "simd_limits")?
    } else {
        test_source::resolve(source, true)?
    };
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
#[test]
fn actual_struct_forms_duplicates_features_and_macro_envelopes_match_pinned_rust() -> TestResult {
    for name in [
        "cfg-removed",
        "source-provider/concrete-at-limit",
        "source-provider/generic-4",
        "source-provider/generic-6",
        "source-provider/generic-8",
        "leading-zero",
        "macro-allowed",
        "maximum",
        "source-provider/ordinary-no-effect",
        "ordinary",
        "plus",
        "raw-string",
        "unused-generic",
        "zero",
    ] {
        analyze(&fixture(name)?).map_err(|e| format!("{name}: {e}"))?;
    }
    for name in [
        "alias",
        "duplicate-different",
        "duplicate-same",
        "empty",
        "enum",
        "field",
        "float",
        "function",
        "hex",
        "impl",
        "integer",
        "list",
        "macro-forwarded",
        "macro-no-allowance",
        "missing-gate",
        "module",
        "negative",
        "overflow",
        "space",
        "trait",
        "underscores",
        "union",
        "unsafe",
        "word",
    ] {
        let error = analyze(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(error.contains("SIMD lane limit"), "{name}: {error}");
    }
    Ok(())
}
fn concrete(
    report: &crate::resolve::expansion::Report,
    facts: &crate::sema::model::Declarations,
    name: &str,
) -> TestResult<TypeId> {
    let item = report
        .resolution
        .definitions
        .iter()
        .position(|d| {
            report.package.names.get(d.name) == name
                && d.kind == crate::resolve::model::DefKind::Struct
        })
        .ok_or("actual struct")?;
    for index in 0..facts.types.len() {
        let ty = TypeId(u32::try_from(index)?);
        if matches!(facts.types.kind(ty), Some(Type::Adt(id, _)) if id.0 as usize==item)
            && facts.types.contains_parameters(ty) == Some(false)
        {
            return Ok(ty);
        }
    }
    Err("actual concrete SIMD instance".into())
}
#[test]
fn generic_counts_enforce_the_same_limit_after_real_substitution() -> TestResult {
    for (name, expected) in [
        ("source-provider/generic-4", Some((16, 16))),
        ("source-provider/generic-6", Some((24, 8))),
        ("source-provider/generic-8", Some((32, 32))),
        ("source-provider/generic-9", None),
        ("source-provider/generic-16", None),
        ("source-provider/generic-24", None),
    ] {
        let analyzed = analyze(&fixture(name)?);
        if expected.is_none() && analyzed.is_err() {
            let error = analyzed
                .err()
                .ok_or("over-limit analysis error")?
                .to_string();
            assert!(error.contains("monomorphization limit"), "{name}: {error}");
            continue;
        }
        let (report, mut analysis) = analyzed?;
        let ty = concrete(&report, &analysis.facts, "V")?;
        let facts = &mut analysis.facts;
        let result = crate::abi::templates::Templates::default().demand(
            &mut facts.types,
            &mut facts.aggregates,
            &mut facts.layouts,
            ty,
        );
        if let Some(expected) = expected {
            assert!(result?, "{name}: actual completed instance");
            let layout = facts.layouts.get(ty).ok_or("concrete SIMD layout")?;
            assert_eq!((layout.size, layout.align), expected, "{name}");
        } else {
            let error = result
                .err()
                .ok_or_else(|| format!("{name}: over-limit layout accepted"))?;
            assert!(
                error.message.contains("monomorphization limit"),
                "{name}: {error}"
            );
            assert!(
                facts.layouts.get(ty).is_none(),
                "failed limit published a layout"
            );
        }
    }
    Ok(())
}
#[test]
fn zero_and_ordinary_structs_have_distinct_layout_behavior() -> TestResult {
    analyze(&fixture("unused-generic")?)?;
    analyze(&fixture("source-provider/ordinary-no-effect")?)?;
    for name in [
        "source-provider/concrete-over-limit",
        "source-provider/zero-concrete",
    ] {
        let error = analyze(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(error.contains("monomorphization limit"), "{name}: {error}");
    }
    let (_, analysis) = analyze(&fixture("ordinary")?)?;
    assert!(analysis.facts.aggregates.simd_limit_bytes() > 0);
    Ok(())
}
#[test]
fn old_metadata_profiles_cannot_silently_drop_limits() -> TestResult {
    let (report, mut analysis) = analyze(&fixture("ordinary")?)?;
    let error = crate::sema::crates::export(
        &report,
        &mut analysis.facts,
        crate::support::crates::CrateIdentity::new("limits", "owned")?,
    )
    .err()
    .ok_or("limit metadata erased")?
    .to_string();
    assert!(
        error.contains("SIMD lane limits require authenticated representation metadata transport"),
        "{error}"
    );
    assert!(
        analysis
            .facts
            .aggregates
            .require_simd_limit_transport()
            .is_err()
    );
    Ok(())
}
#[test]
fn shared_limits_are_linear_and_queries_do_not_allocate_or_expand_hot_records() -> TestResult {
    use std::fmt::Write;
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut source = String::from("#![feature(rustc_attrs,repr_simd)]");
        for index in 0..width {
            write!(
                source,
                "#[repr(simd,packed)]#[rustc_simd_monomorphize_lane_limit=\"8\"]struct V{index}([i32;6]);"
            )?;
        }
        let (report, analysis) = analyze(&source)?;
        let facts = &analysis.facts;
        let before = crate::sema::header_attributes::source_contract_costs(facts);
        let bytes = facts.aggregates.simd_limit_bytes();
        for index in 0..width {
            let ty = concrete(&report, facts, &format!("V{index}"))?;
            assert!(matches!(
                facts.aggregates.simd_shape(&facts.types, ty)?,
                types::aggregate::SimdShape::Ready { count: 6, .. }
            ));
        }
        assert_eq!(facts.aggregates.simd_limit_bytes(), bytes);
        assert_eq!(
            crate::sema::header_attributes::source_contract_costs(facts),
            before
        );
        observations.push(before);
        eprintln!(
            "SIMD limits width={width} work={} bytes={} policy_bytes={bytes}",
            before.0, before.1
        );
    }
    // Geometric table growth stays bounded for real owners; no all-pairs pass.
    assert!(observations[2].0 <= 5 * observations[1].0);
    assert!(observations[2].1 <= 5 * observations[1].1);
    assert_eq!(size_of::<types::aggregate::Repr>(), 20);
    assert_eq!(size_of::<types::aggregate::Definition>(), 36);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn actual_generic_native_layout_body_lowers_with_the_original_limit() -> TestResult {
    let source = fixture("source-provider/native")?;
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
        .join("../obj/parallel/simd-lane-limit-v1/candidate-native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
