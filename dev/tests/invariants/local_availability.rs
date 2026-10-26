//! Cold storage and exact resolved ownership are separate from API permission.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ROOT: &str = "#![feature(staged_api)]#![stable(feature=\"root\",since=\"1.0.0\")]";

#[test]
fn ordinary_sources_allocate_no_library_policy() -> Result {
    let report = crate::test_source::resolve("fn main(){}", true)?;
    assert!(report.package.library.is_none());
    assert_eq!(size_of::<super::super::package::Package>(), 256);
    assert_eq!(size_of::<crate::sema::associated::Owner>(), 32);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    Ok(())
}

#[test]
fn inherited_policy_belongs_only_to_actual_api_owners() -> Result {
    let report = crate::test_source::resolve(
        "#![feature(staged_api)]#![unstable(feature=\"local\",issue=\"none\")]pub fn f(x:u8)->u8{let y=x;y}",
        true,
    )?;
    let inventory = report
        .package
        .library_policy()
        .and_then(Library::inventory)
        .ok_or("inventory")?;
    let mut ordinary = 0;
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        let id = DefId(u32::try_from(index)?);
        if !matches!(
            definition.kind,
            super::super::model::DefKind::Function | super::super::model::DefKind::Module
        ) {
            ordinary += 1;
            assert!(inventory.declaration(id).is_none(), "{:?}", definition.kind);
        }
    }
    assert!(ordinary > 3);
    Ok(())
}

#[test]
fn policy_inventory_follows_owner_and_use_axes() -> Result {
    for (family, sizes, growth) in [
        ("owner-width", [8, 32, 128], 6),
        ("repeated-use", [8, 32, 128], 6),
        ("owner-depth", [8, 16, 32], 3),
    ] {
        let mut observations = Vec::new();
        for n in sizes {
            let source = crate::test_source::fixture(&format!(
                "pa31/tests/spec/library-stability/scaling/{family}-{n}.rs"
            ))?;
            let report = crate::test_source::resolve(&source, true)?;
            let steps = report
                .package
                .library_policy()
                .and_then(Library::inventory)
                .ok_or("inventory")?
                .steps();
            eprintln!("library policy {family}/{n}: {steps} steps");
            assert!(steps <= 128 * n + 4096);
            observations.push(steps);
        }
        let first = observations[1]
            .checked_sub(observations[0])
            .ok_or("nonmonotone first work interval")?;
        let second = observations[2]
            .checked_sub(observations[1])
            .ok_or("nonmonotone second work interval")?;
        assert!(
            first > 0 && second <= growth * first,
            "{family}: {observations:?}"
        );
    }
    Ok(())
}

#[test]
fn failed_attribute_work_preserves_unpublished_state() -> Result {
    use crate::syntax::ast::Kind;
    let report = crate::test_source::resolve(&format!("{ROOT}fn f(){{}}"), true)?;
    let unit = &report.package.units[0];
    let owner = unit.configured.owners.first().ok_or("root owner")?;
    let Kind::Attributed { value, .. } = unit.configured.ast.node(owner.node).kind else {
        return Err("root owner is not the actual attribute wrapper".into());
    };
    let (attribute, forest) = unit
        .retained_attributes(owner.attributes.clone())
        .find(|(attribute, forest)| attribute.name(forest, unit.source.text()) == Some("stable"))
        .ok_or("root stability")?;
    let key = NodeKey {
        file: FileId(0),
        node: value,
    };
    for (steps, bytes) in [(1_000_000, 0), (0, 64 * 1024 * 1024)] {
        let mut draft = Some(Draft {
            steps,
            bytes,
            ..Default::default()
        });
        for _ in 0..2 {
            let error = collect(
                report.package.library_policy(),
                &mut draft,
                unit,
                key,
                attribute,
                forest,
                None,
            )
            .err()
            .ok_or("exhausted actual attribute collection succeeded")?;
            assert!(error.to_string().contains("work or storage budget"));
            let state = draft.as_ref().ok_or("failed work disappeared")?;
            assert!(state.policies.is_empty() && state.attributes.is_empty());
            assert!(state.steps > 1_000_000 || state.bytes > 64 * 1024 * 1024);
        }
    }
    Ok(())
}

#[test]
fn public_alias_uses_its_own_policy_and_the_actual_target() -> Result {
    let source = format!(
        "{ROOT}mod hidden{{#[stable(feature=\"target\",since=\"1.0\")]pub fn f()->u8{{7}}}}#[stable(feature=\"alias\",since=\"CURRENT_RUSTC_VERSION\")]pub use hidden::f;"
    );
    let report = crate::test_source::resolve(&source, true)?;
    let inventory = report
        .package
        .library
        .as_deref()
        .and_then(Library::inventory)
        .ok_or("inventory")?;
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, definition)| {
            definition.kind == super::super::model::DefKind::Function
                && report.package.names.get(definition.name) == "f"
        })
        .ok_or("target")?;
    let target = inventory
        .declaration(DefId(u32::try_from(index)?))
        .ok_or("target policy")?;
    assert_eq!(target.feature.as_ref(), "target");
    assert!(matches!(
        target.level,
        Level::Stable(Since::Version {
            major: 1,
            minor: 0,
            patch: 0
        })
    ));
    let alias = inventory.import(0).ok_or("import policy")?;
    assert_eq!(alias.feature.as_ref(), "alias");
    assert!(matches!(alias.level, Level::Stable(Since::Current)));
    assert_ne!(alias.origin, target.origin);
    Ok(())
}

#[test]
fn failed_library_publication_preserves_requests_and_cannot_fall_back() -> Result {
    use crate::{
        expand::config::Config,
        resolve::{
            expansion,
            load::{InputFile, Sources},
        },
        support::source::SourceFile,
    };
    struct NoFiles;
    impl Sources for NoFiles {
        fn read(&mut self, _: &std::path::Path) -> std::result::Result<Option<InputFile>, String> {
            Ok(None)
        }
    }
    let text = format!("{ROOT}#![feature(missing)]pub fn f(){{}}");
    let mut report = expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(text.into_bytes())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?;
    for _ in 0..2 {
        assert!(crate::resolve::analyze(&mut report.package).is_err());
        let library = report.package.library.as_deref().ok_or("requests lost")?;
        assert_eq!(library.requests.len(), 1);
        assert!(library.inventory().is_none());
        assert!(report.resolution.source_bindings.is_none());
    }
    Ok(())
}

#[test]
fn const_policy_axes_validate_local_owners_and_keep_execution_distinct() -> Result {
    let source = format!(
        "{ROOT}#[unstable(feature=\"api\",issue=\"none\")]pub struct Local;#[unstable(feature=\"api\",issue=\"none\")]pub const fn local()->u8{{7}}fn use_local()->u8{{local()}}"
    );
    let report = crate::test_source::resolve(&source, true)?;
    crate::pipeline::analyze_source(&report)?;
    let inventory = report
        .package
        .library_policy()
        .and_then(Library::inventory)
        .ok_or("inventory")?;
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| report.package.names.get(d.name) == "local")
        .ok_or("actual local")?;
    let local = DefId(u32::try_from(index)?);
    assert_eq!(
        inventory
            .declaration(local)
            .ok_or("regular")?
            .feature
            .as_ref(),
        "api"
    );
    assert_eq!(
        inventory
            .const_declaration(local)
            .ok_or("const")?
            .feature
            .as_ref(),
        "api"
    );
    let ordinary = crate::test_source::resolve(&format!("{ROOT}pub fn ordinary(){{}}"), true);
    assert!(
        ordinary.is_err(),
        "stable crate does not annotate its public children"
    );
    let no_const = crate::test_source::resolve(&format!("{ROOT}fn ordinary(){{}}"), true)?;
    assert!(
        no_const
            .package
            .library_policy()
            .and_then(Library::inventory)
            .ok_or("ordinary inventory")?
            .consts
            .is_none()
    );
    invalid_const_attributes()?;
    for (target, accepted) in [
        (
            "#[rustc_const_stable(feature=\"target\",since=\"1.0\")]",
            true,
        ),
        (
            "#[rustc_const_unstable(feature=\"target\",issue=\"none\")]",
            false,
        ),
    ] {
        let source = format!(
            "{ROOT}#[stable(feature=\"root\",since=\"1.0\")]{target}const fn target()->u8{{7}}#[stable(feature=\"root\",since=\"1.0\")]#[rustc_const_stable(feature=\"caller\",since=\"1.0\")]const fn caller()->u8{{target()}}"
        );
        let report = crate::test_source::resolve(&source, true)?;
        let result = crate::pipeline::analyze_source(&report);
        assert_eq!(
            result.is_ok(),
            accepted,
            "stable body: {:?}",
            result.err().map(|e| e.to_string())
        );
    }
    assert_eq!(size_of::<Library>(), 128);
    assert_eq!(size_of::<Inventory>(), 88);
    assert_eq!(size_of::<super::const_policies::ConstPolicies>(), 24);
    assert_eq!(size_of::<Policy>(), 48);
    eprintln!(
        "stability layouts Library={} Inventory={} ConstPolicies={} Policy={} Level={} Package={} Report={}",
        size_of::<Library>(),
        size_of::<Inventory>(),
        size_of::<super::const_policies::ConstPolicies>(),
        size_of::<Policy>(),
        size_of::<Level>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::resolve::expansion::Report>()
    );
    Ok(())
}

fn invalid_const_attributes() -> Result {
    for (name, suffix, expected) in [
        (
            "missing staged",
            "#[rustc_const_stable(feature=\"c\",since=\"1.0\")]const fn f(){}",
            "requires staged_api",
        ),
        (
            "wrong target",
            "#[rustc_const_stable(feature=\"c\",since=\"1.0\")]struct X;",
            "requires a const function",
        ),
        (
            "malformed",
            "#[rustc_const_stable(feature=\"c\",since=\"bad\")]const fn f(){}",
            "invalid stable library version",
        ),
        (
            "conflicting",
            "#[rustc_const_stable(feature=\"c\",since=\"1.0\")]#[rustc_const_unstable(feature=\"u\",issue=\"none\")]const fn f(){}",
            "duplicate or conflicting",
        ),
    ] {
        let text = if name == "missing staged" {
            suffix.to_owned()
        } else {
            format!("{ROOT}{suffix}")
        };
        let error = crate::test_source::resolve(&text, true)
            .err()
            .ok_or("invalid const policy accepted")?;
        assert!(error.contains(expected), "{name}: {error}");
    }
    Ok(())
}
