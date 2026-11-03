//! Genuine private const bodies carry their exact policy to fresh consumers.
use super::{TestResult, fixture};
use crate::{
    lower::instances::{self, SourcePublication},
    pipeline,
    support::crates::{
        self,
        generics::{self as g, source as s},
    },
};
fn provider()
-> Result<(crate::lowir::program::Program, crates::Interface), Box<dyn std::error::Error>> {
    let report =
        crate::test_source::source_provider(&fixture("native-provider")?, "const_allowances")?;
    let identity = report
        .package
        .source_core()
        .ok_or("provider")?
        .provider
        .clone();
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    Ok(instances::build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        SourcePublication::ModuleConversions,
    )?)
}
fn write(name: &str, bytes: impl AsRef<[u8]>) -> TestResult {
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/const-fn-allowances-v1/candidate-v1");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join(name), bytes)?;
    Ok(())
}
#[test]
fn genuine_original_const_bodies_roundtrip_and_execute_without_client_feature_requests()
-> TestResult {
    let (program, interface) = provider()?;
    let bytes = crates::encode(&interface)?;
    assert_eq!(bytes.get(13..15), Some([9, 0].as_slice()));
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    let view = decoded.source_descriptor().ok_or("descriptor")?.rows();
    let contracts = view
        .availability()?
        .const_bodies
        .as_deref()
        .ok_or("body contracts")?;
    assert_eq!(contracts.implicit.len(), 1);
    assert_eq!(contracts.grants.len(), 1);
    assert_eq!(contracts.grants[0].owner, contracts.implicit[0]);
    assert_eq!(contracts.grants[0].feature, "door");
    crate::source_packets::with_client(
        decoded,
        &fixture("consumer")?,
        "minicore",
        |report, _, client| -> TestResult {
            assert_eq!(report.package.units.len(), 1);
            let mut text = Vec::new();
            crate::lowir::writer::write(client, &mut text)?;
            write("client.lowir", text)
        },
    )??;
    let mut text = Vec::new();
    crate::lowir::writer::write(&program, &mut text)?;
    write("provider.lowir", text)?;
    write("provider.rgcrate", bytes)?;
    for opponent in ["consumer-ungated-target", "consumer-grant-not-request"] {
        let error = crate::source_packets::with_client(
            crates::decode(&crates::encode(&interface)?)?,
            &fixture(opponent)?,
            "minicore",
            |_, _, _| (),
        )
        .err()
        .ok_or("grant enabled an imported API feature")?;
        assert!(error.to_string().contains("feature"), "{opponent}: {error}");
    }
    Ok(())
}
fn publish(
    interface: &crates::Interface,
    rows: s::SourceRows,
    availability: s::AvailabilityRows,
    work: &mut g::Work,
) -> Result<g::Payload, &'static str> {
    let descriptor = interface.source_descriptor().ok_or("descriptor")?;
    let view = descriptor.rows();
    g::Payload::from_source_module_conversions(
        interface.identity(),
        rows,
        view.header_attributes().ok_or("attributes")?.clone(),
        descriptor.module_contracts()?.rows().clone(),
        availability,
        0,
        work,
    )
}
#[test]
fn malformed_body_contracts_reject_forged_targets_features_and_wire_versions() -> TestResult {
    let (_, interface) = provider()?;
    let view = interface.source_descriptor().ok_or("descriptor")?.rows();
    let rows = crate::module_packets::raw_rows(view);
    let actual = view.availability()?.clone();
    let edits: [fn(&mut s::AvailabilityRows); 8] = [
        |r| {
            if let Some(c) = r.const_bodies.as_deref_mut() {
                c.grants[0].owner.0 = u32::MAX;
            }
        },
        |r| {
            if let Some(c) = r.const_bodies.as_deref_mut() {
                c.grants.push(c.grants[0].clone());
            }
        },
        |r| {
            if let Some(c) = r.const_bodies.as_deref_mut() {
                c.implicit.push(c.implicit[0]);
            }
        },
        |r| {
            if let Some(c) = r.const_bodies.as_deref_mut() {
                c.grants[0].feature = "x=y".into();
            }
        },
        |r| {
            if let Some(c) = r.const_bodies.as_deref_mut() {
                c.grants[0].feature = "e\u{301}".into();
            }
        },
        |r| {
            if let Some(c) = r.const_bodies.as_deref_mut() {
                c.grants.clear();
                c.implicit.clear();
            }
        },
        |r| {
            if let Some(c) = r.const_bodies.as_deref_mut() {
                c.implicit[0].0 = u32::MAX;
            }
        },
        |r| {
            if let Some(c) = r.const_bodies.as_deref_mut() {
                c.grants[0].feature = "x".repeat(64 * 1024 + 1);
            }
        },
    ];
    for edit in edits {
        let mut changed = actual.clone();
        edit(&mut changed);
        let mut work = g::Work::default();
        assert!(publish(&interface, rows.clone(), changed, &mut work).is_err());
        assert!(work.steps > 0);
    }
    let mut changed = rows.clone();
    let owner = actual.const_bodies.as_deref().ok_or("contracts")?.grants[0].owner;
    let function = changed
        .functions
        .iter_mut()
        .find(|f| f.declaration == owner)
        .ok_or("function")?;
    function.constness = crate::support::constness::Constness::Ordinary;
    assert!(publish(&interface, changed, actual.clone(), &mut g::Work::default()).is_err());
    Ok(())
}
#[test]
fn body_policy_wire_preserves_older_packets_and_rejects_unknown_versions() -> TestResult {
    let (_, interface) = provider()?;
    let bytes = crates::encode(&interface)?;
    let trailer = bytes
        .windows(2)
        .rposition(|w| w == [0xEC, 1])
        .ok_or("versioned trailer")?;
    for length in trailer + 1..bytes.len() {
        assert!(
            crates::decode(&bytes[..length]).is_err(),
            "truncation {length}"
        );
    }
    let mut unknown = bytes.clone();
    unknown[trailer + 1] = 2;
    assert!(crates::decode(&unknown).is_err());
    let old = crates::decode(&bytes[..trailer])?;
    assert_eq!(crates::encode(&old)?, bytes[..trailer]);
    let error =
        crate::source_packets::with_client(old, &fixture("consumer")?, "minicore", |_, _, _| ())
            .err()
            .ok_or("absence fabricated defining-body authority")?;
    assert!(
        error.to_string().contains("const-safe callee policy"),
        "{error}"
    );
    Ok(())
}
