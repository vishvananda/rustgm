//! Actual opaque declarations retain identity without inventing fields or layout.
use crate::{
    pipeline,
    resolve::{
        model::{DefId, DefKind},
        unit::Unit,
    },
    sema::declarations,
    support::{names::Names, source::SourceFile},
    syntax::{ast::Kind as NodeKind, gate_sites, item::ItemKind},
    types::{ItemId, Kind},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/foreign-types/{name}.rs"
    ))?)
}
fn provider(name: &str) -> TestResult<crate::resolve::expansion::Report> {
    Ok(crate::test_source::source_provider(
        &source(name)?,
        "foreign_provider",
    )?)
}
#[test]
fn actual_foreign_declarations_keep_type_identity_names_and_external_owner() -> TestResult {
    let report = provider("identity-and-pointee")?;
    let facts = declarations::analyze(&report)?;
    let (index, definition) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.kind == DefKind::ForeignType)
        .ok_or("actual Foreign declaration")?;
    let id = DefId(u32::try_from(index)?);
    let key = definition.node.ok_or("actual Foreign source item")?;
    let block = report
        .resolution
        .foreign_owners
        .get(&id)
        .ok_or("actual external block")?;
    assert!(report.resolution.foreign_blocks.contains(block));
    let unit = report.package.unit(key.file);
    let NodeKind::Item(item) = unit.configured.ast.node(key.node).kind else {
        return Err("actual Foreign item disappeared".into());
    };
    assert_eq!(
        unit.configured.ast.items[item as usize].kind,
        ItemKind::ForeignType
    );
    assert!(definition.members.is_none());
    assert!(facts.aggregates.get(ItemId(id.0)).is_none());
    let ty = facts.definitions[index].ok_or("completed Foreign type")?;
    assert_eq!(facts.types.kind(ty), Some(Kind::Foreign(ItemId(id.0))));
    assert_eq!(facts.types.is_sized(ty), Some(false));
    assert_eq!(facts.types.is_well_formed(ty), Some(true));
    assert_eq!(report.package.names.get(definition.name), "Opaque");
    Ok(())
}
#[test]
fn foreign_metadata_and_real_body_calls_keep_unit_separate_from_sizing() -> TestResult {
    for name in [
        "identity-and-pointee",
        "metadata-unit",
        "metadata-struct-tail",
    ] {
        let report = provider(name)?;
        let analysis = pipeline::analyze_source(&report).map_err(|e| format!("{name}: {e}"))?;
        let contracts = crate::body::regions::collect(&analysis.facts)?;
        crate::borrow::regions::check(
            &analysis.facts.types,
            &analysis.facts.lifetimes.variances,
            &contracts,
        )?;
    }
    for name in [
        "sized-rejected",
        "meta-sized-rejected",
        "tail-meta-sized-rejected",
        "by-value-rejected",
        "distinct-identities",
    ] {
        let report = provider(name)?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or_else(|| format!("{name} gained unsupported proof"))?;
        assert!(
            !error.message.contains("unsupported type declaration"),
            "{name}: {error}"
        );
        assert!(
            !error.message.contains("item is not permitted"),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn foreign_syntax_gates_follow_literal_tokens_and_active_owners() -> TestResult {
    provider("literal-allowance")?;
    let disabled = provider("disabled-missing-feature")?;
    assert!(
        !disabled
            .resolution
            .definitions
            .iter()
            .any(|d| d.kind == DefKind::ForeignType)
    );
    assert!(disabled.pre_expansion_warnings()?.is_empty());
    for name in ["missing-feature", "forwarded-allowance"] {
        let error = provider(name)
            .err()
            .ok_or_else(|| format!("{name} borrowed a false allowance"))?;
        assert!(
            error.to_string().contains("extern_types"),
            "{name}: {error}"
        );
    }
    for name in [
        "generic-form",
        "bound-form",
        "definition-form",
        "privacy",
        "duplicate-name",
    ] {
        assert!(
            provider(name).is_err(),
            "{name} admitted malformed source or invalid names"
        );
    }
    Ok(())
}
#[test]
fn foreign_source_freeze_uses_actual_impl_instead_of_empty_structural_fields() -> TestResult {
    for name in ["direct-freeze-rejected", "pointer-freeze"] {
        let report = provider(name)?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or_else(|| format!("{name} invented Freeze"))?;
        assert!(
            !error.message.contains("unsupported type declaration"),
            "{name}: {error}"
        );
    }
    let report = provider("pointer-freeze-source-impl")?;
    pipeline::analyze_source(&report)?;
    Ok(())
}
#[test]
fn foreign_values_keep_unknown_layout_and_transport_while_pointer_storage_is_real() -> TestResult {
    let report = provider("identity-and-pointee")?;
    let mut facts = declarations::analyze(&report)?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| d.kind == DefKind::ForeignType)
        .ok_or("actual Foreign")?;
    let ty = facts.definitions[index].ok_or("actual Foreign type")?;
    assert_eq!(
        facts
            .layouts
            .prepared_pointer_metadata(&facts.types, &facts.aggregates, ty)?,
        crate::abi::metadata::Metadata::Thin
    );
    let error = facts
        .layouts
        .demand(&facts.types, &facts.aggregates, ty)
        .err()
        .ok_or("Foreign acquired a value layout")?;
    assert!(
        error.message.contains("unknown value size and alignment"),
        "{error}"
    );
    let pointer = facts.types.intern(Kind::Pointer {
        mutable: false,
        pointee: ty,
    })?;
    let layout = facts
        .layouts
        .demand(&facts.types, &facts.aggregates, pointer)?;
    let layout = facts.layouts.at(layout).ok_or("actual pointer layout")?;
    assert_eq!((layout.size, layout.align), (8, 8));
    let reference = facts.types.intern(Kind::Reference {
        mutable: false,
        region: crate::types::RegionId::STATIC,
        referent: ty,
    })?;
    let reference = facts
        .layouts
        .demand(&facts.types, &facts.aggregates, reference)?;
    let reference = facts
        .layouts
        .at(reference)
        .ok_or("actual reference layout")?;
    assert_eq!((reference.size, reference.align), (8, 8));
    let transport = "#![feature(no_core,lang_items,extern_types)]#![no_core]#![allow(internal_features,incomplete_features)]\n#[lang=\"pointee_sized\"]pub trait PointeeSized{}\n#[lang=\"meta_sized\"]pub trait MetaSized:PointeeSized{}\n#[lang=\"sized\"]pub trait Sized:MetaSized{}\nunsafe extern \"C\"{pub type Opaque;}";
    let report = crate::test_source::source_provider(transport, "foreign_transport")?;
    let mut facts = declarations::analyze(&report)?;
    let identity = &report
        .package
        .source_core()
        .ok_or("actual source provider identity")?
        .provider;
    let error = crate::sema::crates::authored_export::Stage::begin(&report, &mut facts, identity)
        .err()
        .ok_or("Foreign transport invented")?;
    assert_eq!(
        error,
        "source declaration kind is outside the initial source profile"
    );
    Ok(())
}
fn replay_foreign_sites() -> TestResult {
    let mut names = Names::default();
    let unit = Unit::parse(
        SourceFile::new(b"#![feature(extern_types)]unsafe extern \"C\"{type Opaque;}".to_vec())?,
        &crate::expand::config::Config::default(),
        &mut names,
    )
    .map_err(|e| e.message)?;
    for replay in [
        unit.fresh_instance().map_err(|e| e.message)?,
        unit.fresh_instance().map_err(|e| e.message)?,
    ] {
        replay
            .configured
            .ast
            .validate_gate_sites_source(replay.source.original_image())
            .map_err(|e| e.message)?;
        assert!(!replay.configured.ast.has_defaultness());
        let sites = replay
            .configured
            .ast
            .gate_sites
            .as_ref()
            .ok_or("actual Foreign site owner")?;
        let rows = sites.entries();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, gate_sites::Kind::ForeignType);
        assert_eq!(
            replay
                .source
                .text()
                .get(rows[0].span.start as usize..rows[0].span.end as usize),
            Some("type")
        );
        assert!(replay.configured.ast.defaultness(rows[0].item).is_none());
    }
    Ok(())
}
#[test]
fn foreign_replay_width_and_hot_layouts_use_only_the_existing_sparse_owner() -> TestResult {
    replay_foreign_sites()?;
    for width in [8, 32, 128] {
        let mut input = String::from("#![feature(extern_types)]unsafe extern \"C\"{");
        use std::fmt::Write;
        for i in 0..width {
            write!(input, "type F{i};")?;
        }
        input.push('}');
        let report = crate::test_source::resolve(&input, true)?;
        let facts = declarations::analyze(&report)?;
        assert_eq!(
            report
                .resolution
                .definitions
                .iter()
                .filter(|d| d.kind == DefKind::ForeignType)
                .count(),
            width
        );
        assert_eq!(
            facts
                .definitions
                .iter()
                .flatten()
                .filter_map(|&t| facts.types.kind(t))
                .filter(|t| matches!(t, Kind::Foreign(_)))
                .count(),
            width
        );
        let sites = report
            .package
            .unit(crate::resolve::unit::FileId(0))
            .configured
            .ast
            .gate_sites
            .as_ref()
            .ok_or("sparse source sites")?;
        assert_eq!(sites.entries().len(), width);
        println!(
            "Foreign width={width} source-storage={:?} canonical-types={}",
            sites.storage(),
            facts.types.len()
        );
    }
    let ordinary = crate::test_source::resolve("fn ordinary(){}", true)?;
    assert!(
        ordinary
            .package
            .unit(crate::resolve::unit::FileId(0))
            .configured
            .ast
            .gate_sites
            .is_none()
    );
    assert_eq!(size_of::<Kind>(), 24);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<gate_sites::Declaration>(), 24);
    assert_eq!(size_of::<gate_sites::Declarations>(), 56);
    println!(
        "Foreign layouts ItemKind={} Item={} TypeKind={} AST={} Node={} Constructor={} Store={} Unit={} Package={} Report={} Definition={} Scope={}",
        size_of::<ItemKind>(),
        size_of::<crate::syntax::item::Item>(),
        size_of::<Kind>(),
        size_of::<crate::syntax::ast::Kind>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::types::infer::Constructor>(),
        size_of::<crate::types::Store>(),
        size_of::<Unit>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::resolve::expansion::Report>(),
        size_of::<crate::resolve::model::Definition>(),
        size_of::<crate::resolve::model::Scope>()
    );
    Ok(())
}
