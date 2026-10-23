//! Source sizing uses real declarations, explicit levels and scoped ordinary predicates.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        bindings::Context,
        load::{InputFile, Sources},
    },
    sema,
    support::{bindings::Mode, source::SourceFile},
    types::{Kind, parameters::ParameterKind},
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = "#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]#[lang=\"pointee_sized\"]pub trait PointeeSized{}#[lang=\"meta_sized\"]pub trait MetaSized:PointeeSized{}#[lang=\"sized\"]pub trait Sized:MetaSized{}";
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn resolve(source: &str) -> Result<Report> {
    let identity = CrateIdentity::new("minicore", "provider_v1")?;
    let context = Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    };
    let report = resolve::expansion::analyze_with_context(
        InputFile {
            identity: "provider.rs".into(),
            path: "provider.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
        &context,
    )?;
    Ok(resolve::analyze_expanded(report)?)
}
fn analyze(tail: &str) -> Result<sema::model::Declarations> {
    let report = resolve(&(SOURCE.to_owned() + tail))?;
    let facts = sema::declarations::analyze(&report)?;
    Ok(sema::bodies::analyze(&report, facts)?)
}
#[test]
fn source_headers_validate_before_sizing_answers() -> Result {
    let report = resolve(SOURCE)?;
    let facts = sema::model::Declarations::new(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let meta = catalog
        .language(LangItem::MetaSized)
        .ok_or("meta")?
        .definition;
    assert!(catalog.sizing_kind(meta).is_err());
    assert_eq!(catalog.sizing_kind(DefId(u32::MAX))?, None);
    let mut facts = sema::declarations::analyze(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    assert_eq!(catalog.sizing_kind(meta)?, Some(LangItem::MetaSized));
    assert!(validate_sizing(&report, &mut facts).is_err());
    Ok(())
}

#[test]
fn source_catalog_storage_is_absent_from_ordinary_sources() -> Result {
    let report = crate::test_source::resolve("fn ordinary(){}", true)?;
    let facts = sema::declarations::analyze(&report)?;
    assert!(facts.source_bindings.is_none());
    println!(
        "Package={} Resolution={} Declarations={} SourceCatalog={} Reservations={}",
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::resolve::model::Resolution>(),
        size_of::<sema::model::Declarations>(),
        size_of::<Catalog>(),
        size_of::<crate::resolve::bindings::Reservations>()
    );
    Ok(())
}
#[test]
fn explicit_source_levels_suppress_only_the_declared_defaults() -> Result {
    let source = SOURCE.to_owned()
        + "fn ordinary<T>(){}fn meta<T:MetaSized>(){}fn pointee<T:PointeeSized>(){}fn both<T:PointeeSized+Sized>(){}fn relaxed<T:?Sized>(){}mod local{pub trait MetaSized{}}fn lookalike<T:local::MetaSized>(){}";
    let report = resolve(&source)?;
    let facts = sema::declarations::analyze(&report)?;
    for (name, expected) in [
        ("ordinary", true),
        ("meta", false),
        ("pointee", false),
        ("both", true),
        ("relaxed", false),
        ("lookalike", true),
    ] {
        let function = facts
            .functions
            .iter()
            .find(|f| {
                let d = &report.resolution.definitions[f.definition.0 as usize];
                report.package.names.get(d.name) == name
            })
            .ok_or("function")?;
        let argument = facts
            .generics
            .parameters
            .get(&function.definition)
            .and_then(|p| p.first())
            .ok_or("parameter")?
            .argument;
        let crate::types::arguments::Argument::Type(ty) = argument else {
            return Err("type".into());
        };
        let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err("parameter type".into());
        };
        assert_eq!(
            facts.types.parameter(id).ok_or("declaration")?.kind,
            ParameterKind::Type { sized: expected },
            "{name}"
        );
    }
    Ok(())
}
#[test]
fn source_hierarchy_preserves_paid_and_unpaid_parameter_evidence() -> Result {
    for tail in [
        "fn need<T:MetaSized>(){}fn test<T>(){need::<T>();}",
        "fn need<T:MetaSized>(){}fn test<T:?Sized>(){need::<T>();}",
        "fn need<T:MetaSized>(){}fn test<T:PointeeSized+Sized>(){need::<T>();}",
        "fn need<T:MetaSized>(){}fn test(){need::<[u8]>();}",
        "fn need<T:MetaSized>(){}trait Q{fn test(){need::<Self>();}}",
    ] {
        analyze(tail)?;
    }
    for tail in [
        "fn need<T:MetaSized>(){}fn test<T:PointeeSized>(){need::<T>();}",
        "fn need<T:Sized>(){}fn test<T:MetaSized>(){need::<T>();}",
        "fn need<T:Sized>(){}fn test(){need::<[u8]>();}",
    ] {
        assert!(analyze(tail).is_err(), "{tail}");
    }
    Ok(())
}
#[test]
fn actual_source_implementation_restrictions_do_not_follow_names() -> Result {
    let source = SOURCE.replace(
        "#[lang=\"sized\"]",
        "#[rustc_deny_explicit_impl]#[lang=\"sized\"]",
    ) + "struct S;impl Sized for S{}";
    let report = resolve(&source)?;
    assert!(sema::declarations::analyze(&report).is_err());
    analyze("mod local{pub trait Sized{}pub struct S;impl Sized for S{}}")?;
    let source = SOURCE.replace(
        "pub trait MetaSized:PointeeSized{}",
        "pub trait MetaSized{}",
    );
    let report = resolve(&source)?;
    assert!(sema::declarations::analyze(&report).is_err());
    Ok(())
}

#[test]
fn source_header_attribute_claims_do_not_authorize_unprepared_role_queries() -> Result {
    let source = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../pa31/tests/spec/source-header-transport/provider-capabilities/scoped-sizing.rs",
    ))?;
    let report = resolve(&source)?;
    let unprepared = sema::model::Declarations::new(&report)?;
    let catalog = unprepared.source_bindings.as_deref().ok_or("catalog")?;
    let sized = catalog.language(LangItem::Sized).ok_or("sized")?.definition;
    assert!(catalog.forces_dyn_incompatible(sized).is_err());
    Ok(())
}
