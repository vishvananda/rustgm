//! A compatibility note is a leaf-owned warning policy, never module permission.
use super::*;
use crate::{
    lower::instances::SourcePublication,
    module_packets, pipeline, source_packets,
    support::crates::{self, generics::source as s},
    test_source,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BASE: &str = "pa31/tests/spec/source-module-compatibility/";
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!("{BASE}{name}.rs"))?)
}
fn provider() -> TestResult<crates::Interface> {
    module_packets::exported_profile(
        &fixture("ordinary-provider")?,
        SourcePublication::ModuleCompatibility,
        "module_compatibility",
    )
}
fn named(report: &Report, name: &str) -> TestResult<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|row| report.package.names.get(row.name) == name)
        .ok_or("actual named declaration")?;
    Ok(DefId(u32::try_from(index)?))
}
#[test]
fn direct_compatibility_notes_keep_last_empty_owner_and_gate_semantics() -> TestResult {
    for (name, expected) in [
        ("decl-duplicate", "last"),
        ("decl-empty", ""),
        ("decl-stable-struct", "note"),
    ] {
        let report = test_source::source_provider(&fixture(name)?, &name.replace('-', "_"))?;
        let inventory = report
            .package
            .library_policy()
            .and_then(crate::resolve::stability::Library::inventory)
            .ok_or("completed inventory")?;
        let notes = inventory
            .policies()
            .iter()
            .filter_map(|p| match &p.level {
                crate::resolve::stability::Level::StableWithCompatibility { note, .. } => {
                    Some(note)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(notes.len(), 1, "{name}");
        assert_eq!(&*notes[0].message, expected);
    }
    for name in [
        "word",
        "list",
        "numeric",
        "missing-pair",
        "unstable-pair",
        "const-only",
        "missing-gate",
        "unsafe-wrapper",
        "wrong-target",
    ] {
        assert!(
            test_source::source_provider(
                &fixture(&format!("decl-{name}"))?,
                &name.replace('-', "_")
            )
            .is_err(),
            "{name}"
        );
    }
    assert_eq!(size_of::<crate::resolve::stability::Level>(), 24);
    assert_eq!(size_of::<crate::resolve::stability::Policy>(), 56);
    Ok(())
}
#[test]
fn actual_v6_provider_fresh_clients_preserve_leaf_prefix_and_lint_protocols() -> TestResult {
    let artifact = provider()?;
    let bytes = crates::encode(&artifact)?;
    assert_eq!(bytes.get(13..15), Some([6, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for (name, deprecations, unfulfilled) in [
        ("marked-warning", 1, 0),
        ("marked-allow", 0, 0),
        ("marked-expect", 0, 0),
        ("marked-enabled", 0, 0),
        ("stable-alias", 0, 0),
        ("local-body", 0, 0),
        ("unfulfilled-stable", 0, 1),
        ("nested-warning", 2, 0),
        ("nested-partly-enabled", 1, 0),
        ("nested-enabled", 0, 0),
        ("import-alias", 0, 0),
    ] {
        let (_, analysis, _) =
            source_packets::client_analysis(crates::decode(&bytes)?, &fixture(name)?)?;
        assert_eq!(
            analysis
                .facts
                .warnings()
                .filter(|w| w.code == WarningCode::Deprecated)
                .count(),
            deprecations,
            "{name}"
        );
        assert_eq!(
            analysis
                .facts
                .warnings()
                .filter(|w| w.code == WarningCode::UnfulfilledLintExpectations)
                .count(),
            unfulfilled,
            "{name}"
        );
    }
    for (name, needle) in [
        ("marked-deny", "use the stable alias"),
        ("marked-forbid", "use the stable alias"),
        ("plain-denied", "compat_mod"),
        ("leaf-denied", "compat_leaf_gate"),
        ("two-import-groups", "compat_mod"),
    ] {
        let error = source_packets::client_analysis(crates::decode(&bytes)?, &fixture(name)?)
            .err()
            .ok_or("opposing client accepted")?;
        assert!(error.to_string().contains(needle), "{name}: {error}");
    }
    write_native_input(bytes)
}
fn write_native_input(bytes: Vec<u8>) -> TestResult {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
        _,
    ) = source_packets::client_analysis(crates::decode(&bytes)?, &fixture("native-client")?)?;
    assert_eq!(
        facts
            .warnings()
            .filter(|w| w.code == WarningCode::Deprecated)
            .count(),
        1
    );
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
        .join("../obj/parallel/module-path-compat-v1/candidate-v1");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("client.rs"), fixture("native-client")?)?;
    std::fs::write(folder.join("provider.rgcrate"), bytes)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("client.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn checked_uses_retain_pending_bodies_distinct_prefixes_and_resumed_dedup() -> TestResult {
    let bytes = crates::encode(&provider()?)?;
    let (report, _, _) =
        source_packets::client_analysis(crates::decode(&bytes)?, &fixture("selected-body")?)?;
    let declarations = crate::sema::declarations::analyze(&report)?;
    assert!(declarations.warnings().next().is_none());
    for name in ["checked", "untouched"] {
        let (facts, _) = pipeline::prepare_local_runtime(&report, &[named(&report, name)?])?;
        assert!(facts.warnings().next().is_none(), "{name}");
    }
    let text = fixture("nested-warning")?;
    let (report, mut analysis, _) =
        source_packets::client_analysis(crates::decode(&bytes)?, &text)?;
    let offset = u32::try_from(text.find("base::gated").ok_or("source path")?)?;
    let reference=report.resolution.references.iter().find(|r|
        report.package.unit(r.node.file).configured.ast.node(r.node.node).span.start==offset
        && matches!(r.target,crate::resolve::model::Target::Definition(id) if report.resolution.definitions[id.0 as usize].kind==crate::resolve::model::DefKind::Function)).ok_or("actual nested path")?;
    let crate::resolve::model::Target::Definition(target) = reference.target else {
        return Err("actual leaf".into());
    };
    for _ in 0..4 {
        check(&report, &mut analysis.facts, reference.node, target)?;
    }
    assert_eq!(
        analysis
            .facts
            .warnings()
            .filter(|w| w.code == WarningCode::Deprecated)
            .count(),
        2
    );
    assert!(analysis.facts.warnings().all(|w| {
        w.diagnostic.span.start
            == report
                .package
                .unit(reference.node.file)
                .configured
                .ast
                .node(reference.node.node)
                .span
                .start
    }));
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn old_origin_deprecation_stays_required_and_old_profiles_cannot_drop_notes() -> TestResult {
    let required = module_packets::exported_profile(
        &fixture("provider")?,
        SourcePublication::ModuleCompatibility,
        "wider_callable_profile",
    )
    .err()
    .ok_or("nonordinary source declarations silently admitted")?;
    assert!(
        required
            .to_string()
            .contains("source nonordinary callable effect/safety/convention"),
        "{required}"
    );
    let source = fixture("ordinary-provider")?;
    let report = test_source::source_provider(&source, "old_note_profile")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let identity = report
        .package
        .source_core()
        .ok_or("provider")?
        .provider
        .clone();
    let error = crate::lower::instances::build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        SourcePublication::ModuleDefinitions,
    )
    .err()
    .ok_or("v5 dropped real note")?;
    assert!(error.to_string().contains("version-six"), "{error}");
    let old = module_packets::exported_profile(
        &test_source::fixture("pa31/tests/spec/source-availability-transport/provider.rs")?,
        SourcePublication::ModuleAvailability,
        "old_origin",
    )?;
    let old_bytes = crates::encode(&old)?;
    for path in ["base::identity(7u8)", "legacy(7u8)"] {
        let use_ = if path.starts_with("legacy") {
            "use base::identity as legacy;"
        } else {
            ""
        };
        let text = format!(
            "#![feature(no_core)]#![no_core]extern crate base;{use_}#[expect(deprecated)]pub fn probe()->u8{{{path}}}"
        );
        let error = source_packets::client_analysis(crates::decode(&old_bytes)?, &text)
            .err()
            .ok_or("old leaf deprecation knowledge was cleared")?;
        assert!(
            error
                .to_string()
                .contains("imported deprecation contract transport"),
            "{error}"
        );
    }
    let dependency = source_packets::client_analysis_with(
        provider()?, vec![crates::decode(&old_bytes)?],
        "#![feature(no_core)]#![no_core]extern crate base;extern crate dependency_0;#[expect(deprecated)]pub fn probe()->u8{dependency_0::identity(7u8)}",
    ).err().ok_or("multiple source providers were silently admitted")?;
    assert!(
        dependency
            .to_string()
            .contains("source function profile requires one complete source binding provider"),
        "{dependency}"
    );
    let artifact = provider()?;
    let descriptor = artifact.source_descriptor().ok_or("v6 descriptor")?;
    assert_eq!(
        descriptor.profile(),
        s::Profile::Source2024ModuleCompatibility
    );
    Ok(())
}
