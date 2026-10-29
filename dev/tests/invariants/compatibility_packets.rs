//! Real source rows keep versioned compatibility data separate from library permissions.
use super::*;
use crate::{
    lower::instances::SourcePublication,
    module_packets,
    support::crates::{self, generics as g},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn provider() -> Result<crates::Interface> {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-availability-transport/provider.rs")?;
    module_packets::exported_profile(
        &source,
        SourcePublication::ModuleAvailability,
        "compatibility_packets",
    )
}
fn noted(artifact: &crates::Interface, message: &str) -> Result<s::AvailabilityRows> {
    let mut rows = artifact
        .source_descriptor()
        .ok_or("descriptor")?
        .availability()?
        .clone();
    let policy = rows
        .policies
        .iter_mut()
        .find(|policy| policy.feature == "identity_api")
        .ok_or("actual stable identity policy")?;
    let s::AvailabilityLevel::Stable(since) = policy.level else {
        return Err("actual stable source policy".into());
    };
    policy.level = s::AvailabilityLevel::StableWithCompatibility {
        since,
        message: message.into(),
    };
    Ok(rows)
}
fn stage(
    artifact: &crates::Interface,
    policies: s::AvailabilityRows,
    profile: s::Profile,
    base: usize,
    work: &mut g::Work,
) -> Result<g::Payload> {
    let (rows, attributes, modules) = module_packets::raw(artifact)?;
    let publish = match profile {
        s::Profile::Source2024ModuleAvailability => g::Payload::from_source_module_availability,
        s::Profile::Source2024ModuleFeaturePredicates => {
            g::Payload::from_source_module_feature_predicates
        }
        s::Profile::Source2024ModuleDefinitions => g::Payload::from_source_module_definitions,
        s::Profile::Source2024ModuleCompatibility => g::Payload::from_source_module_compatibility,
        _ => return Err("unexpected test profile".into()),
    };
    Ok(publish(
        artifact.identity(),
        rows,
        attributes,
        modules,
        policies,
        base,
        work,
    )?)
}
fn interface(
    artifact: &crates::Interface,
    policies: s::AvailabilityRows,
) -> Result<crates::Interface> {
    let base = crates::Interface::new(
        artifact.identity().clone(),
        artifact.dependencies().to_vec(),
        artifact.functions().to_vec(),
    )?;
    let payload = stage(
        artifact,
        policies,
        s::Profile::Source2024ModuleCompatibility,
        base.base_retained_bytes(),
        &mut g::Work::default(),
    )?;
    Ok(base.with_generic_payload(payload)?)
}

#[test]
fn version_six_roundtrips_present_empty_and_utf8_notes_with_exact_origins() -> Result {
    let artifact = provider()?;
    for message in ["", "import via std::mem instead", "line\n\u{03bb}\0"] {
        let policies = noted(&artifact, message)?;
        let expected = policies.clone();
        let complete = interface(&artifact, policies)?;
        let bytes = crates::encode(&complete)?;
        assert_eq!(bytes.get(13..15), Some([6, 0].as_slice()));
        let decoded = crates::decode(&bytes)?;
        let descriptor = decoded.source_descriptor().ok_or("decoded descriptor")?;
        assert_eq!(
            descriptor.profile(),
            s::Profile::Source2024ModuleCompatibility
        );
        assert_eq!(descriptor.availability()?, &expected);
        assert_eq!(crates::encode(&decoded)?, bytes);
    }
    Ok(())
}

#[test]
fn older_profiles_reject_compatibility_even_when_its_message_is_empty() -> Result {
    let artifact = provider()?;
    for profile in [
        s::Profile::Source2024ModuleAvailability,
        s::Profile::Source2024ModuleFeaturePredicates,
        s::Profile::Source2024ModuleDefinitions,
    ] {
        let mut work = g::Work::default();
        let error = stage(&artifact, noted(&artifact, "")?, profile, 0, &mut work)
            .err()
            .ok_or("legacy profile dropped a present note")?
            .to_string();
        assert!(error.contains("version-six"), "{profile:?}: {error}");
        assert!(work.steps > 0);
    }
    let complete = interface(&artifact, noted(&artifact, "v6-only")?)?;
    let bytes = crates::encode(&complete)?;
    for version in [3u16, 4, 5] {
        let mut changed = bytes.clone();
        changed[13..15].copy_from_slice(&version.to_le_bytes());
        assert!(
            crates::decode(&changed).is_err(),
            "downgraded version {version}"
        );
    }
    Ok(())
}

#[test]
fn compatibility_rejects_foreign_const_oversized_and_exhausted_policy_inputs() -> Result {
    let artifact = provider()?;
    for mutation in 0..4 {
        let mut rows = noted(&artifact, "original")?;
        let policy = rows
            .policies
            .iter_mut()
            .find(|policy| policy.feature == "identity_api")
            .ok_or("actual policy")?;
        match mutation {
            0 => policy.kind = s::AvailabilityKind::Const,
            1 => {
                policy.origin = s::AvailabilityOrigin::Declaration {
                    owner: DeclarationRef(u32::MAX),
                    site: 0,
                }
            }
            2 => {
                policy.origin = s::AvailabilityOrigin::Declaration {
                    owner: DeclarationRef(0),
                    site: u32::MAX,
                }
            }
            _ => {
                policy.level = s::AvailabilityLevel::StableWithCompatibility {
                    since: s::AvailabilitySince::Current,
                    message: "x".repeat(s::MAX_REASON_BYTES + 1),
                }
            }
        }
        let mut work = g::Work::default();
        assert!(
            stage(
                &artifact,
                rows,
                s::Profile::Source2024ModuleCompatibility,
                0,
                &mut work
            )
            .is_err(),
            "mutation {mutation}"
        );
        assert!(work.steps > 0);
    }
    let mut work = g::Work {
        steps: MAX_COMPONENTS,
        ..g::Work::default()
    };
    assert!(
        stage(
            &artifact,
            noted(&artifact, "budget")?,
            s::Profile::Source2024ModuleCompatibility,
            0,
            &mut work
        )
        .is_err()
    );
    assert!(work.steps > MAX_COMPONENTS);
    assert!(
        stage(
            &artifact,
            noted(&artifact, "budget")?,
            s::Profile::Source2024ModuleCompatibility,
            crates::MAX_BYTES,
            &mut g::Work::default()
        )
        .is_err()
    );
    let bytes = crates::encode(&interface(
        &artifact,
        noted(&artifact, "unique-compatibility-text")?,
    )?)?;
    let at = bytes
        .windows(b"unique-compatibility-text".len())
        .position(|bytes| bytes == b"unique-compatibility-text")
        .ok_or("encoded message")?;
    let mut invalid = bytes.clone();
    invalid[at] = 0xff;
    assert!(crates::decode(&invalid).is_err());
    for end in [12, at + 2, bytes.len() - 1] {
        assert!(crates::decode(&bytes[..end]).is_err());
    }
    Ok(())
}

#[test]
fn existing_stable_and_unstable_policy_wire_bytes_are_unchanged() -> Result {
    let origin = s::AvailabilityOrigin::Declaration {
        owner: DeclarationRef(4),
        site: 5,
    };
    let since = s::AvailabilitySince::Version {
        major: 1,
        minor: 2,
        patch: 3,
    };
    for (level, expected) in [
        (
            s::AvailabilityLevel::Stable(since),
            vec![
                0, 1, 0, 0, 0, b'f', 0, 1, 1, 0, 2, 0, 3, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0,
            ],
        ),
        (
            s::AvailabilityLevel::Unstable {
                issue: None,
                reason: None,
            },
            vec![0, 1, 0, 0, 0, b'f', 1, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0],
        ),
    ] {
        let policy = s::AvailabilityPolicy {
            kind: s::AvailabilityKind::Regular,
            feature: "f".into(),
            level,
            origin,
        };
        for compatibility in [false, true] {
            let mut out = Writer::default();
            policy_write(&mut out, &policy, compatibility)?;
            assert_eq!(out.finish(), expected);
        }
    }
    Ok(())
}
