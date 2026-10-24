//! Root language requests are retained without creating library/provider state.
use super::{Context, Features, MacroFeatures, SourceCore};
use crate::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
        package::Package,
    },
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
};
use std::path::Path;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn package(source: &str, context: &Context) -> Result<Package> {
    Ok(resolve::load::load_with_context(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
        context,
    )?)
}

#[test]
fn ordinary_requests_do_not_allocate_a_provider_or_library_policy() -> Result {
    for (source, bits) in [
        ("", 0),
        ("#![feature(decl_macro)]", 1),
        ("#![feature(rustc_attrs)]", 2),
        ("#![feature(decl_macro,rustc_attrs)]", 3),
        ("#![feature(decl_macro,decl_macro)]", 1),
    ] {
        let package = package(source, &Context::default())?;
        assert_eq!(package.macro_features.0, bits, "{source}");
        assert!(!package.uses_source_bindings());
        assert!(package.library.is_none());
    }
    assert_eq!(size_of::<Features>(), 2);
    assert_eq!(size_of::<MacroFeatures>(), 1);
    assert_eq!(size_of::<SourceCore>(), 72);
    assert_eq!(size_of::<Package>(), 256);
    Ok(())
}

#[test]
fn only_enabled_crate_root_requests_supply_macro_gates() -> Result {
    for (source, bits) in [
        ("#![cfg_attr(all(),feature(decl_macro,rustc_attrs))]", 3),
        ("#![cfg_attr(any(),feature(decl_macro,rustc_attrs))]", 0),
        ("mod inner{#![feature(decl_macro,rustc_attrs)]}", 0),
        ("#[feature(decl_macro,rustc_attrs)]fn ordinary(){}", 0),
    ] {
        assert_eq!(
            package(source, &Context::default())?.macro_features.0,
            bits,
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn provider_ownership_and_macro_requests_are_independent() -> Result {
    let identity = CrateIdentity::new("minicore", "macro_requests")?;
    let context = Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    };
    let package = package(
        "#![no_core]#![feature(no_core,decl_macro,rustc_attrs)]",
        &context,
    )?;
    assert_eq!(package.macro_features.0, 3);
    assert!(package.source_core().is_some());
    assert!(package.library.is_none());
    Ok(())
}

#[test]
fn scoped_declarations_require_the_gate_before_semantic_admission() -> Result {
    for (request, message) in [
        ("", "scoped macros require decl_macro"),
        (
            "#![feature(rustc_attrs)]",
            "scoped macros require decl_macro",
        ),
        (
            "#![feature(decl_macro)]",
            "scoped declarative macro hygiene is not implemented",
        ),
    ] {
        let source = format!("{request}macro m(){{}}fn ordinary(){{}}");
        let mut loaded = package(&source, &Context::default())?;
        let direct = resolve::analyze(&mut loaded)
            .err()
            .ok_or("direct resolution accepted unsupported scoped hygiene")?;
        assert_eq!(direct.message, message);
        let expanded = crate::test_source::resolve(&source, true)
            .err()
            .ok_or("expansion accepted unsupported scoped hygiene")?;
        assert!(expanded.contains(message), "{expanded}");
    }
    Ok(())
}

#[test]
fn disabled_scoped_declarations_do_not_require_a_request() -> Result {
    let source = "#[cfg(any())]macro m(){}fn ordinary(){}";
    let mut loaded = package(source, &Context::default())?;
    resolve::analyze(&mut loaded)?;
    crate::test_source::resolve(source, true)?;
    Ok(())
}
