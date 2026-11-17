//! Real checked roots and actual metadata imports retain separate policy authority.
use super::*;
#[path = "../helpers/retired_data.rs"]
mod controls;
use crate::support::crates::{
    self, CrateIdentity, ExternalDef, ExternalKind, Function, Interface, LoaderFlags,
    RemovedFeatures, Scalar, Signature, Symbol,
};
use controls::Result;

#[test]
fn removed_data_preserves_the_actual_root_policy_and_decoded_strings() -> Result {
    let report = controls::source("actual.rs")?;
    let inventory = report
        .package
        .library_policy()
        .and_then(super::super::Library::inventory)
        .ok_or("checked inventory")?;
    assert!(!inventory.policies.is_empty());
    for policy in &inventory.policies {
        assert_eq!(&*policy.feature, "rust1");
        assert!(matches!(policy.level, super::super::Level::Stable(_)));
    }
    let rows = report
        .package
        .removed_features()?
        .ok_or("actual removed declaration")?;
    assert_eq!(rows.entries().len(), 1);
    let row = &rows.entries()[0];
    assert_eq!(row.feature, "concat_idents");
    assert_eq!(
        row.reason,
        "Replaced by the macro_metavar_expr_concat feature"
    );
    assert_eq!(
        row.link,
        "https://github.com/rust-lang/rust/issues/29599#issuecomment-2986866250"
    );
    assert_eq!(
        row.since,
        RemovedVersion {
            major: 1,
            minor: 90,
            patch: 0
        }
    );
    drop(report);
    let owned = controls::source("strings.rs")?
        .package
        .removed_features()?
        .ok_or("owned strings")?;
    assert_eq!(owned.entries()[0].feature, "escaped_probe");
    assert_eq!(owned.entries()[0].reason, "first\nsecond");
    assert_eq!(
        owned.entries()[0].link,
        "https://example.invalid/?q=\"retired\""
    );
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::resolve::bindings::SourceCore>(), 248);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    Ok(())
}
#[test]
fn repeated_removed_declarations_combine_without_becoming_active_definitions() -> Result {
    for (file, count) in [
        ("multiple.rs", 2),
        ("repeated.rs", 2),
        ("empty-data.rs", 1),
        ("arbitrary-feature.rs", 1),
    ] {
        let report = controls::source(file)?;
        let inventory = report
            .package
            .library_policy()
            .and_then(super::super::Library::inventory)
            .ok_or("inventory")?;
        assert_eq!(inventory.policies.len(), 1);
        assert_eq!(&*inventory.policies[0].feature, "removed_root");
        let rows = report.package.removed_features()?.ok_or("records")?;
        assert_eq!(rows.entries().len(), count);
        let interface = Interface::new(rows.owner().clone(), Vec::new(), Vec::new())?
            .with_removed_features(rows)?;
        let bytes = crates::encode(&interface)?;
        assert_eq!(crates::decode(&bytes)?, interface);
    }
    assert!(controls::source("local-request.rs").is_err());
    Ok(())
}
#[test]
fn fallback_requests_reject_removed_names_and_preserve_active_first_lookup() -> Result {
    for (file, expected, diagnostic) in [
        ("client-request.rs", false, "has been removed since 1.90.0"),
        ("client-active.rs", true, ""),
        ("client-plain.rs", true, ""),
        ("client-other.rs", false, "undefined library feature"),
    ] {
        let bytes = crates::encode(&controls::provider()?)?;
        let result = controls::client(crates::decode(&bytes)?, file);
        assert_eq!(result.is_ok(), expected, "{file}");
        if let Ok(report) = &result {
            assert!(
                !report
                    .resolution
                    .crates
                    .as_deref()
                    .ok_or("removed-data import owner")?
                    .has_availability(),
                "data-only imports are not API availability owners"
            );
        }
        if !expected {
            assert!(
                result
                    .err()
                    .ok_or("accepted opposing request")?
                    .to_string()
                    .contains(diagnostic),
                "{file}"
            );
        }
    }
    Ok(())
}
#[test]
fn catalogless_removed_imports_cannot_carry_a_callable_or_gain_permission_from_flags() -> Result {
    let original = controls::provider()?;
    let owner = original.identity().clone();
    let rows = original.removed_features().ok_or("data")?.clone();
    let function = Function::new(
        ExternalDef::new(owner.clone(), vec!["forged".into()], ExternalKind::Function)?,
        Signature::new(Vec::new(), Scalar::Unit, false)?,
        Symbol::Rust,
    )?;
    let forged = Interface::new(owner, Vec::new(), vec![function])?
        .with_removed_features(rows)?
        .with_loader_flags(LoaderFlags::from_bits(7)?)?;
    let result = controls::client(
        crates::decode(&crates::encode(&forged)?)?,
        "client-plain.rs",
    );
    let error = result
        .err()
        .ok_or("catalogless callable was accepted")?
        .to_string();
    assert!(error.contains("primitive empty artifacts"), "{error}");
    Ok(())
}
#[test]
fn removed_metadata_owner_quota_and_plain_bytes_stay_closed() -> Result {
    let artifact = controls::provider()?;
    let table = artifact.removed_features().ok_or("table")?.clone();
    let foreign = CrateIdentity::new("foreign", "retired_controls")?;
    assert!(
        Interface::new(foreign, Vec::new(), Vec::new())?
            .with_removed_features(table)
            .is_err()
    );
    assert!(RemovedFeatures::new(artifact.identity().clone(), Vec::new()).is_err());
    let plain = Interface::new(CrateIdentity::new("core", "x")?, Vec::new(), Vec::new())?;
    let mut expected = b"RGMETA\r\n\x01\x00\x00\x00\x00".to_vec();
    expected.extend(4_u32.to_le_bytes());
    expected.extend(b"core");
    expected.extend(1_u32.to_le_bytes());
    expected.extend(b"x");
    expected.extend([0_u8; 8]);
    assert_eq!(crates::encode(&plain)?, expected);
    let mut bytes = crates::encode(&artifact)?;
    assert_eq!(&bytes[12..14], &[16, 1]);
    bytes[13] = 2;
    assert!(crates::decode(&bytes).is_err());
    let original = crates::encode(&artifact)?;
    for end in 0..original.len() {
        assert!(crates::decode(&original[..end]).is_err(), "prefix {end}");
    }
    let offset =
        14 + 4 + artifact.identity().name().len() + 4 + artifact.identity().disambiguator().len();
    let mut excessive = original.clone();
    excessive[offset..offset + 4].copy_from_slice(&1_000_001_u32.to_le_bytes());
    assert!(crates::decode(&excessive).is_err());
    let mut oversized = original.clone();
    oversized[offset + 4..offset + 8].copy_from_slice(&8_000_001_u32.to_le_bytes());
    assert!(crates::decode(&oversized).is_err());
    Ok(())
}
fn inner_offset(artifact: &Interface) -> Result<usize> {
    let table = artifact.removed_features().ok_or("table")?;
    Ok(14
        + 8
        + table.owner().name().len()
        + table.owner().disambiguator().len()
        + 4
        + table
            .entries()
            .iter()
            .map(|row| 18 + row.feature.len() + row.reason.len() + row.link.len())
            .sum::<usize>())
}
#[test]
fn flags_and_removed_envelopes_compose_once_at_the_outermost_owner() -> Result {
    let artifact = controls::provider()?;
    let bytes = crates::encode(&artifact)?;
    let offset = inner_offset(&artifact)?;
    let mut recursive = bytes[..offset].to_vec();
    recursive.extend(u32::try_from(bytes.len())?.to_le_bytes());
    recursive.extend(&bytes);
    assert!(crates::decode(&recursive).is_err());
    let flagged = controls::provider()?.with_loader_flags(LoaderFlags::from_bits(7)?)?;
    let mixed = crates::encode(&flagged)?;
    assert_eq!(&mixed[12..16], &[15, 7, 16, 1]);
    assert_eq!(crates::decode(&mixed)?, flagged);
    assert_eq!(crates::encode(&crates::decode(&mixed)?)?, mixed);
    let mut foreign = bytes.clone();
    foreign[18] = b'x';
    assert!(crates::decode(&foreign).is_err());
    let mut noncanonical = bytes[..offset].to_vec();
    noncanonical.extend(u32::try_from(mixed.len())?.to_le_bytes());
    noncanonical.extend(mixed);
    assert!(crates::decode(&noncanonical).is_err());
    Ok(())
}

#[test]
fn removed_publication_obeys_the_existing_sticky_source_payer() -> Result {
    use crate::{
        expand::config::Config,
        resolve::{
            bindings::Context,
            load::{InputFile, Sources},
        },
        support::{bindings::Mode, source::SourceFile},
    };
    struct NoFiles;
    impl Sources for NoFiles {
        fn read(&mut self, _: &std::path::Path) -> std::result::Result<Option<InputFile>, String> {
            Ok(None)
        }
    }
    let identity = CrateIdentity::new("removed_provider", "unfinished_inventory")?;
    let context = Context {
        crate_name: Some(identity.name().into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Context::default()
    };
    let unfinished = crate::resolve::load::load_with_context(
        InputFile {
            identity: "unfinished.rs".into(),
            path: "unfinished.rs".into(),
            source: SourceFile::new(controls::fixture("provider.rs")?.into_bytes())?,
        },
        &mut NoFiles,
        &Config::default(),
        &context,
    )?;
    let error = unfinished
        .removed_features()
        .err()
        .ok_or("unfinished inventory published absence")?
        .to_string();
    assert!(error.contains("inventory is incomplete"), "{error}");
    let report = controls::source("provider.rs")?;
    let source = report.package.source_core().ok_or("source owner")?;
    assert!(source.macros.charge(usize::MAX, usize::MAX).is_err());
    assert!(report.package.removed_features().is_err());
    let inventory = report
        .package
        .library_policy()
        .and_then(super::super::Library::inventory)
        .ok_or("inventory")?;
    assert_eq!(
        inventory
            .retirements
            .as_ref()
            .ok_or("retained declaration")?
            .entries
            .len(),
        1
    );
    Ok(())
}
#[test]
fn source_clients_cannot_republish_a_root_declaration_by_importing_data() -> Result {
    let provider = crate::module_packets::exported_profile(
        &crate::test_source::fixture("pa31/tests/spec/source-availability-transport/provider.rs")?,
        crate::lower::instances::SourcePublication::ModuleAvailability,
        "retired_source_client",
    )?;
    let result = crate::source_packets::client_analysis(provider, &controls::fixture("actual.rs")?);
    let error = result
        .err()
        .ok_or("source client declaration gained source-core authority")?
        .to_string();
    assert!(
        error.contains("removed library declarations require the actual source-core profile"),
        "{error}"
    );
    Ok(())
}
#[test]
fn removed_envelopes_stay_outside_native_dependency_inner_metadata() -> Result {
    use crate::support::linker_libraries::{Kind, Library, Plan};
    let make_plan = || -> Result<Plan> {
        let mut plan = Plan::default();
        plan.push(Library::new("c", Kind::Dylib)?)?;
        Ok(plan)
    };
    let artifact = controls::provider()?;
    let bytes = crates::encode(&artifact)?;
    let owner = artifact.identity().clone();
    let wrapped = artifact
        .with_native_links(make_plan()?)?
        .with_loader_flags(LoaderFlags::PANIC_RUNTIME)?;
    let encoded = crates::encode(&wrapped)?;
    assert_eq!(&encoded[12..16], &[15, 2, 16, 1]);
    assert_eq!(crates::decode(&encoded)?, wrapped);
    let native = Interface::new(owner, Vec::new(), Vec::new())?.with_native_links(make_plan()?)?;
    let mut nested = crates::encode(&native)?[..23].to_vec();
    nested.extend(u32::try_from(bytes.len())?.to_le_bytes());
    nested.extend(bytes);
    let error = crates::decode(&nested)
        .err()
        .ok_or("inner removed declarations were accepted")?;
    assert!(error.contains("outermost artifact"), "{error}");
    Ok(())
}

#[test]
fn removed_rows_and_publication_pay_bounded_work_and_storage() -> Result {
    use std::fmt::Write;
    let foundation = controls::fixture("provider.rs")?;
    for distinct in [false, true] {
        let mut observations = Vec::new();
        for width in [8usize, 32, 128] {
            let mut text = foundation.clone();
            for index in 0..width {
                let feature = if distinct {
                    format!("retired_{index}")
                } else {
                    "shared_retired".into()
                };
                writeln!(
                    text,
                    "#![unstable_removed(feature={feature:?},reason=\"r\",link=\"l\",since=\"1.90.0\")]"
                )?;
            }
            let report = crate::test_source::source_provider(&text, "removed_growth")?;
            let inventory = report
                .package
                .library_policy()
                .and_then(super::super::Library::inventory)
                .ok_or("inventory")?;
            assert_eq!(
                inventory
                    .retirements
                    .as_ref()
                    .ok_or("removed rows")?
                    .entries
                    .len(),
                width + 1
            );
            assert_eq!(inventory.policies.len(), 1);
            let retained = inventory.removed_bytes();
            let source = report.package.source_core().ok_or("source payer")?;
            let before = source.macros.usage();
            let table = report
                .package
                .removed_features()?
                .ok_or("owned publication")?;
            let after = source.macros.usage();
            assert_eq!(table.entries().len(), width + 1);
            assert!(after.0 > before.0 && after.1 > before.1);
            let copied = after
                .1
                .checked_sub(before.1)
                .ok_or("publication payment decreased")?;
            let stats = [inventory.steps(), retained, copied];
            for amount in stats {
                assert!(
                    amount <= 1024 * (width + 1) + 4096,
                    "{distinct}/{width}: {stats:?}"
                );
            }
            observations.push(stats);
        }
        for metric in 0..3 {
            let first = observations[1][metric]
                .checked_sub(observations[0][metric])
                .ok_or("first growth decreased")?;
            let second = observations[2][metric]
                .checked_sub(observations[1][metric])
                .ok_or("second growth decreased")?;
            assert!(
                first > 0 && second <= first * 6 + 2048,
                "{distinct}/{metric}: {observations:?}"
            );
        }
    }
    Ok(())
}
