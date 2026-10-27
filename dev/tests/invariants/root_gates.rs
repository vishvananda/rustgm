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
    assert_eq!(size_of::<Features>(), 8);
    assert_eq!(size_of::<MacroFeatures>(), 1);
    // The cold provider adds 104 bytes of macro-contract state to its 72-byte base.
    assert_eq!(size_of::<SourceCore>(), 176);
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
    let source = "#![feature(decl_macro)]macro m(){}fn ordinary(){}";
    let mut loaded = package(source, &Context::default())?;
    resolve::analyze(&mut loaded)?;
    let report = crate::test_source::resolve(source, true)?;
    assert!(report.definitions.iter().any(|definition| matches!(
        definition.body,
        crate::resolve::expansion::Body::Scoped { .. }
    )));
    let invoked = crate::test_source::resolve(
        "#![feature(decl_macro)]macro m(){7u32}fn ordinary()->u32{m!()}",
        true,
    )?;
    assert_eq!(invoked.invocations.len(), 1);
    crate::pipeline::analyze_source(&invoked)?;
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

#[test]
fn compiler_root_options_preserve_independent_requests_in_both_source_modes() -> Result {
    for (name, coherence, preserve, cfg_request) in [
        ("ordinary-none", false, false, false),
        ("coherence", true, false, false),
        ("preserve-ub", false, true, false),
        ("all-bits", true, true, true),
        ("cfg-disabled", false, false, false),
        ("cfg-enabled", true, true, false),
        ("cfg-request-alone", false, false, true),
        ("disabled-module", false, false, false),
    ] {
        let source =
            crate::test_source::fixture(&format!("pa31/tests/spec/crate-options/{name}.rs"))?;
        let ordinary = crate::test_source::resolve(&source, true)?;
        assert_eq!(ordinary.package.coherence_is_core(), coherence, "{name}");
        assert_eq!(ordinary.package.preserves_ub_checks(), preserve, "{name}");
        assert_eq!(
            ordinary.package.requested_cfg_ub_checks(),
            cfg_request,
            "{name}"
        );
        let provider = crate::test_source::source_provider(
            &format!("#![no_core]#![feature(no_core)]{source}"),
            "root_options",
        )?;
        assert_eq!(
            provider.package.coherence_is_core(),
            coherence,
            "{name}/provider"
        );
        assert_eq!(
            provider.package.preserves_ub_checks(),
            preserve,
            "{name}/provider"
        );
        assert_eq!(
            provider.package.requested_cfg_ub_checks(),
            cfg_request,
            "{name}/provider"
        );
        if name == "all-bits" {
            assert_eq!(ordinary.package.macro_features.0, 63);
        }
    }
    assert_eq!(size_of::<MacroFeatures>(), 1);
    assert_eq!(size_of::<Features>(), 8);
    assert_eq!(size_of::<Package>(), 256);
    Ok(())
}
#[test]
fn compiler_root_options_reject_malformed_misplaced_and_duplicate_active_sources() -> Result {
    for (name, expected) in [
        ("coherence-missing-gate", "requires rustc_attrs"),
        ("coherence-arguments", "inner word attribute"),
        ("coherence-name-value", "inner word attribute"),
        ("coherence-function", "inner word attribute"),
        ("coherence-module", "actual crate root owner"),
        ("coherence-duplicate", "duplicate compiler crate option"),
        ("coherence-unsafe", "inner word attribute"),
        ("coherence-generated-misplaced", "inner word attribute"),
        ("preserve-missing-gate", "requires rustc_attrs"),
        ("preserve-arguments", "inner word attribute"),
        ("preserve-name-value", "inner word attribute"),
        ("preserve-function", "inner word attribute"),
        ("preserve-module", "actual crate root owner"),
        ("preserve-duplicate", "duplicate compiler crate option"),
        ("preserve-unsafe", "inner word attribute"),
        ("preserve-generated-misplaced", "inner word attribute"),
    ] {
        let source =
            crate::test_source::fixture(&format!("pa31/tests/spec/crate-options/{name}.rs"))?;
        let error = crate::test_source::resolve(&source, true)
            .err()
            .ok_or("invalid crate option accepted")?;
        assert!(error.contains(expected), "{name}: {error}");
    }
    Ok(())
}
