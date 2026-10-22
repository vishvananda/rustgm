//! Source reservations precede semantic authority and never share bootstrap owners.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    support::{bindings::LangItem, source::SourceFile},
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = "#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]#[lang=\"pointee_sized\"]pub trait PointeeSized{}#[lang=\"meta_sized\"]pub trait MetaSized:PointeeSized{}#[lang=\"sized\"]pub trait Sized:MetaSized{}#[rustc_intrinsic]pub const fn size_of<T>()->usize{0}";
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn context() -> Result<Context> {
    let identity = CrateIdentity::new("minicore", "bindings_v1")?;
    Ok(Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    })
}
fn input(source: &str) -> Result<InputFile> {
    Ok(InputFile {
        identity: "source.rs".into(),
        path: "source.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    })
}
fn resolve(source: &str, expanded: bool) -> Result<resolve::expansion::Report> {
    let mut report = resolve::expansion::analyze_with_context(
        input(source)?,
        &mut NoFiles,
        &Config::default(),
        &context()?,
    )?;
    if expanded {
        return Ok(resolve::analyze_expanded(report)?);
    }
    report.resolution = resolve::analyze(&mut report.package)?;
    Ok(report)
}
#[test]
fn source_bindings_reserve_real_definitions_in_both_pipelines() -> Result {
    for expanded in [false, true] {
        let report = resolve(SOURCE, expanded)?;
        let registry = report
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("catalog")?;
        assert_eq!(registry.provider.name(), "minicore");
        for kind in [LangItem::Sized, LangItem::MetaSized, LangItem::PointeeSized] {
            let declaration = registry.language(kind).ok_or("language item")?;
            assert!(
                report.resolution.definitions[declaration.definition.0 as usize]
                    .node
                    .is_some()
            );
            assert!(declaration.parent.is_none());
        }
        let intrinsic = registry.intrinsics().next().ok_or("intrinsic")?;
        assert_eq!(registry.intrinsics().count(), 1);
        assert!(
            registry
                .intrinsic(intrinsic.declaration.definition)
                .is_some()
        );
        assert!(report.resolution.builtin_traits.bootstrap_absent());
        assert!(
            crate::support::library_types::LibraryType::ALL
                .into_iter()
                .all(|kind| report.resolution.library_types.get(kind).is_none())
        );
        assert!(report.resolution.macro_prelude.is_empty());
    }
    Ok(())
}
#[test]
fn source_provider_configuration_rejects_spoofing_before_lookup() -> Result {
    let mut wrong = context()?;
    wrong.current = Some(CrateIdentity::new("minicore", "other")?);
    assert!(
        resolve::load::load_with_context(input(SOURCE)?, &mut NoFiles, &Config::default(), &wrong)
            .is_err()
    );
    let no_core = SOURCE.replace("#![no_core]", "");
    assert!(resolve(&no_core, true).is_err());
    assert!(resolve::load::load(input(SOURCE)?, &mut NoFiles, &Config::default()).is_err());
    assert!(crate::test_source::resolve("#[lang=\"sized\"]trait Sized{}fn f(){}", true).is_err());
    assert!(
        resolve(
            &(SOURCE.to_owned() + "fn f(){let _=core::option::Option::Some(1u8);}"),
            true
        )
        .is_err()
    );
    Ok(())
}
#[test]
fn enabled_claims_are_unique_and_disabled_claims_are_absent() -> Result {
    let source = SOURCE.to_owned() + "#[cfg(any())]#[lang=\"sized\"]trait Disabled{}";
    resolve(&source, true)?;
    let enabled = SOURCE.to_owned() + "#[cfg_attr(all(),lang=\"sized\")]trait Duplicate{}";
    assert!(resolve(&enabled, true).is_err());
    let source = SOURCE.replace(
        "#[lang=\"sized\"]pub trait Sized:MetaSized{}",
        "macro_rules! sized{()=>{#[lang=\"sized\"]pub trait Sized:MetaSized{}}}sized!();",
    );
    let report = resolve(&source, true)?;
    assert!(
        report
            .resolution
            .source_bindings
            .as_deref()
            .and_then(|r| r.language(LangItem::Sized))
            .is_some()
    );
    Ok(())
}
#[test]
fn prototype_syntax_does_not_authorize_surviving_declarations() -> Result {
    for expanded in [false, true] {
        for tail in [
            "#[cfg(any())]fn absent<T>()->usize;",
            "struct S;impl S{#[cfg(any())]fn absent()->usize;}",
            "struct S;#[cfg(any())]impl S{fn absent()->usize;}",
            "#[cfg_attr(all(),cfg(any()))]fn absent<T>()->usize;",
        ] {
            resolve(&(SOURCE.to_owned() + tail), expanded)?;
        }
        let bodyless = SOURCE.replace(
            "const fn size_of<T>()->usize{0}",
            "const fn size_of<T>()->usize;",
        );
        resolve(&bodyless, expanded)?;
        let nested = bodyless.replace(
            "#[rustc_intrinsic]",
            "#[cfg_attr(all(),cfg_attr(all(),rustc_intrinsic))]",
        );
        resolve(&nested, expanded)?;
        for tail in [
            "#[cfg(all())]fn absent<T>()->usize;",
            "#[cfg_attr(any(),rustc_intrinsic)]fn absent<T>()->usize;",
            "struct S;#[cfg(all())]impl S{fn absent()->usize;}",
            "struct S;impl S{#[rustc_intrinsic]fn size_of<T>()->usize;}",
        ] {
            assert!(resolve(&(SOURCE.to_owned() + tail), expanded).is_err());
        }
    }
    assert!(crate::syntax::expression::parse_crate("fn absent();").is_err());
    assert!(crate::syntax::expression::parse_crate("#[allow(dead_code)]fn absent();").is_err());
    assert!(crate::test_source::resolve("#[cfg(all())]fn absent();", true).is_err());
    Ok(())
}
#[test]
fn source_configuration_keeps_bootstrap_owner_absent() -> Result {
    let report = crate::test_source::resolve("fn f(){let _=1u8;}", true)?;
    assert!(report.package.source_core.is_none());
    assert!(report.resolution.source_bindings.is_none());
    eprintln!(
        "source reservation sizes package={} resolution={} source={} catalog={} declaration={} intrinsic={} parser={}",
        size_of::<resolve::package::Package>(),
        size_of::<resolve::model::Resolution>(),
        size_of::<SourceCore>(),
        size_of::<Reservations>(),
        size_of::<Declaration>(),
        size_of::<IntrinsicDeclaration>(),
        size_of::<crate::syntax::expression::Parser<'_, '_>>()
    );
    assert_eq!(size_of::<resolve::model::Resolution>(), 2160);
    Ok(())
}
