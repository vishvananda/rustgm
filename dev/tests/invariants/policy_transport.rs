//! Real staged declarations cross the new wire profile without source syntax in clients.
use crate::{
    lower::instances::SourcePublication,
    module_packets as test, source_packets,
    support::crates::{
        self,
        generics::{self as g, source as s},
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[path = "policy_instances.rs"]
mod instances;
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-availability-transport/{name}.rs"
    ))?)
}
fn provider() -> Result<crates::Interface> {
    test::exported_profile(
        &fixture("provider")?,
        SourcePublication::ModuleAvailability,
        "availability_transport",
    )
}
fn client(artifact: crates::Interface, source: &str) -> Result<crate::lowir::program::Program> {
    let (
        report,
        crate::pipeline::Analysis {
            mut facts,
            prepared,
        },
        _,
    ) = source_packets::client_analysis(artifact, source)?;
    Ok(crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}

#[test]
fn actual_staged_provider_roundtrips_and_fresh_clients_preserve_each_api_gate() -> Result {
    let artifact = provider()?;
    let bytes = crates::encode(&artifact)?;
    assert_eq!(bytes.get(13..15), Some([3, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for name in [
        "stable-client",
        "unstable-client",
        "alias-client",
        "alias-missing",
        "alias-target-only",
        "module-client",
        "unstable_alias_target",
    ] {
        let source = fixture(name)?;
        let program = client(crates::decode(&bytes)?, &source)?;
        let mut lowir = Vec::new();
        crate::lowir::writer::write(&program, &mut lowir)?;
        let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../obj/parallel/availability-transport-v1/native-v3")
            .join(name);
        std::fs::create_dir_all(&output)?;
        std::fs::write(output.join("consumer.lowir"), lowir)?;
        std::fs::write(output.join("provider.rgcrate"), &bytes)?;
        std::fs::write(output.join("provider.rs"), fixture("provider")?)?;
        std::fs::write(output.join("client.rs"), source)?;
    }
    for (name, needle) in [
        ("missing-feature", "experimental_identity"),
        ("unknown-feature", "undefined library feature"),
        ("module-missing", "module_identity"),
        ("module-alias-only", "module_identity"),
        ("stable_alias_none", "experimental_identity"),
        ("unstable_alias_none", "experimental_identity"),
        ("unstable_alias_own", "experimental_identity"),
        ("const-misuse", "const"),
    ] {
        let result = client(crates::decode(&bytes)?, &fixture(name)?);
        let error = result
            .err()
            .ok_or("opposing client unexpectedly accepted")?
            .to_string();
        assert!(error.contains(needle), "{name}: {error}");
    }
    Ok(())
}

fn changed(
    artifact: &crates::Interface,
    mutate: impl FnOnce(&mut s::AvailabilityRows) -> Result,
    base: usize,
    work: &mut g::Work,
) -> Result<g::Payload> {
    let (rows, attrs, modules) = test::raw(artifact)?;
    let mut policies = artifact
        .source_descriptor()
        .ok_or("descriptor")?
        .availability()?
        .clone();
    mutate(&mut policies)?;
    Ok(g::Payload::from_source_module_availability(
        artifact.identity(),
        rows,
        attrs,
        modules,
        policies,
        base,
        work,
    )?)
}

#[test]
fn complete_availability_rejects_missing_foreign_kind_site_and_sibling_origins() -> Result {
    let artifact = provider()?;
    let descriptor = artifact
        .source_descriptor()
        .ok_or("actual source descriptor")?;
    let private = descriptor
        .rows()
        .declarations
        .iter()
        .position(|row| row.name.as_deref() == Some("private_identity"))
        .ok_or("actual private function")?;
    assert_eq!(
        descriptor.availability()?.declarations[private].regular,
        None
    );
    let mutations: [fn(&mut s::AvailabilityRows) -> Result; 7] = [
        |p| {
            p.declarations.pop();
            Ok(())
        },
        |p| {
            p.bindings.pop();
            Ok(())
        },
        |p| {
            p.policies[0].kind = s::AvailabilityKind::Const;
            Ok(())
        },
        |p| {
            p.policies[0].origin = s::AvailabilityOrigin::Declaration {
                owner: crates::owner_catalog::DeclarationRef(u32::MAX),
                site: 0,
            };
            Ok(())
        },
        |p| {
            p.policies[0].origin = s::AvailabilityOrigin::Declaration {
                owner: crates::owner_catalog::DeclarationRef(0),
                site: u32::MAX,
            };
            Ok(())
        },
        |p| {
            let a = p
                .policies
                .iter()
                .position(|p| p.feature == "identity_api")
                .ok_or("identity policy")?;
            let b = p
                .policies
                .iter()
                .find(|p| p.feature == "experimental_identity")
                .ok_or("sibling policy")?
                .origin;
            p.policies[a].origin = b;
            Ok(())
        },
        |p| {
            let policy = p
                .policies
                .iter_mut()
                .find(|policy| policy.feature == "bridge_identity")
                .ok_or("actual binding policy")?;
            let s::AvailabilityOrigin::Binding { site, .. } = &mut policy.origin else {
                return Err("actual import origin".into());
            };
            *site = site.checked_add(1).ok_or("binding site overflow")?;
            Ok(())
        },
    ];
    for mutate in mutations {
        let mut work = g::Work::default();
        assert!(changed(&artifact, mutate, artifact.base_retained_bytes(), &mut work).is_err());
        assert!(work.steps > 0);
    }
    let mut exhausted = g::Work {
        steps: crates::MAX_COMPONENTS,
        ..g::Work::default()
    };
    assert!(changed(&artifact, |_| Ok(()), 0, &mut exhausted).is_err());
    assert!(exhausted.steps > crates::MAX_COMPONENTS);
    assert!(
        changed(
            &artifact,
            |_| Ok(()),
            crates::MAX_BYTES,
            &mut g::Work::default()
        )
        .is_err()
    );
    let mut bytes = crates::encode(&artifact)?;
    bytes[13..15].copy_from_slice(&2u16.to_le_bytes());
    assert!(crates::decode(&bytes).is_err());
    bytes[13..15].copy_from_slice(&3u16.to_le_bytes());
    assert!(crates::decode(&bytes[..bytes.len() - 1]).is_err());
    Ok(())
}

#[test]
fn legacy_availability_remains_unknown_and_hot_records_do_not_grow() -> Result {
    let complete = fixture("provider")?;
    let missing=complete.replacen("#[stable(feature=\"availability_roles\",since=\"1.0.0\")]\nimpl<T: PointeeSized> LegacyReceiver for &T {}","impl<T: PointeeSized> LegacyReceiver for &T {}",1);
    assert_ne!(complete, missing);
    let error = test::exported_profile(
        &missing,
        SourcePublication::ModuleAvailability,
        "missing_impl_policy",
    )
    .err()
    .ok_or("unmarked trait implementation accepted")?
    .to_string();
    assert!(error.contains("implementation availability"), "{error}");
    let original = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    assert!(
        test::exported_profile(
            &original,
            SourcePublication::ModuleAvailability,
            "unstaged_availability"
        )
        .is_err()
    );
    let old = test::exported(&test::fixture("restricted-provider")?)?;
    assert!(
        old.source_descriptor()
            .ok_or("descriptor")?
            .availability()
            .is_err()
    );
    let source = crate::test_source::resolve("fn main(){}", true)?;
    assert!(source.resolution.crates.is_none());
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<s::Columns>(), 592);
    assert_eq!(size_of::<s::SourceRows>(), 616);
    assert_eq!(size_of::<s::ValidatedRows>(), 144);
    assert_eq!(size_of::<crate::resolve::crates::Imported>(), 160);
    assert_eq!(size_of::<crate::resolve::lookup::Observations>(), 64);
    assert_eq!(size_of::<crate::resolve::model::Reference>(), 24);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    println!(
        "availability cold ValidatedRows={} Imported={} trace={} Reference={} Body={}",
        size_of::<s::ValidatedRows>(),
        size_of::<crate::resolve::crates::Imported>(),
        size_of::<crate::resolve::lookup::Observations>(),
        size_of::<crate::resolve::model::Reference>(),
        size_of::<crate::body::ir::Body>()
    );
    Ok(())
}

#[test]
fn availability_owner_cannot_be_copied_between_equal_fresh_catalogs() -> Result {
    let bytes = crates::encode(&provider()?)?;
    let source = fixture("stable-client")?;
    let (mut left, _, _) = source_packets::client_analysis(crates::decode(&bytes)?, &source)?;
    let (mut right, _, _) = source_packets::client_analysis(crates::decode(&bytes)?, &source)?;
    let (site, target) = left
        .resolution
        .references
        .iter()
        .find_map(|reference| {
            let crate::resolve::model::Target::Definition(id) = reference.target else {
                return None;
            };
            let declaration = &left.resolution.definitions[id.0 as usize];
            (declaration.kind == crate::resolve::model::DefKind::Function
                && declaration.node.is_none()
                && left.package.names.get(declaration.name) == "identity")
                .then_some((reference.node, id))
        })
        .ok_or("actual foreign function reference")?;
    super::check_imported_api(&left, site, target, false)?;
    let copied = right
        .resolution
        .crates
        .as_deref_mut()
        .ok_or("right imports")?
        .availability
        .take()
        .ok_or("right owner")?;
    let original = left
        .resolution
        .crates
        .as_deref_mut()
        .ok_or("left imports")?
        .availability
        .replace(copied)
        .ok_or("left owner")?;
    assert!(super::check_imported_api(&left, site, target, false).is_err());
    left.resolution
        .crates
        .as_deref_mut()
        .ok_or("left imports")?
        .availability = Some(original);
    super::check_imported_api(&left, site, target, false)?;
    let held = left
        .resolution
        .crates
        .as_deref_mut()
        .ok_or("left imports")?
        .availability
        .take();
    assert!(super::check_imported_api(&left, site, target, false).is_err());
    left.resolution
        .crates
        .as_deref_mut()
        .ok_or("left imports")?
        .availability = held;
    let imported = left.resolution.crates.take();
    assert!(super::check_imported_api(&left, site, target, false).is_err());
    left.resolution.crates = imported;
    super::check_imported_api(&left, site, target, false)?;
    Ok(())
}

#[test]
fn mixed_legacy_imports_remain_required_without_local_library_requests() -> Result {
    let identity = crates::CrateIdentity::new("legacy", "availability_unknown")?;
    let function = crates::Function::new(
        crates::ExternalDef::new(
            identity.clone(),
            vec!["legacy".into()],
            crates::ExternalKind::Function,
        )?,
        crates::Signature::new(vec![], crates::Scalar::U8, false)?,
        crates::Symbol::Rust,
    )?;
    let legacy = crates::Interface::new(identity, vec![], vec![function])?;
    let source = "#![feature(no_core)]#![no_core]extern crate base;extern crate dependency_0;#[unsafe(no_mangle)]pub extern \"C\" fn answer()->u8{dependency_0::legacy()}";
    let error = source_packets::client_analysis_with(provider()?, vec![legacy], source)
        .err()
        .ok_or("legacy foreign API was treated as local availability absence")?
        .to_string();
    assert!(
        error.contains("imported API availability is unknown"),
        "{error}"
    );
    Ok(())
}
