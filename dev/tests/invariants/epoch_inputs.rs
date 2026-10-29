//! Legacy rows roundtrip as data, while semantic entry rejects old auto defaults.
use super::*;
use crate::support::crates as codec;
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[test]
fn historical_provided_bytes_keep_exact_profile_and_never_supply_current_premises() -> TestResult {
    let bytes =
        crate::module_packets::hex("pa31/tests/spec/provided-contracts-v1/provided-v0.hex")?;
    let interface = codec::decode(&bytes)?;
    assert_eq!(codec::encode(&interface)?, bytes);
    let rows = interface
        .source_descriptor()
        .ok_or("historical rows")?
        .rows();
    assert_eq!(rows.descriptor().profile(), s::Profile::Source2024Provided);
    let mut paid = 0;
    let legacy_error = require(rows, |steps| {
        paid += steps;
        Ok(())
    })
    .err()
    .ok_or("legacy inputs became semantic premises")?;
    assert!(legacy_error.message.contains("legacy Provided-v0"));
    assert_eq!(paid, 1);
    let mut attempted = 0;
    assert!(
        require(rows, |steps| {
            attempted += steps;
            Err(error("fixture budget exhausted"))
        })
        .is_err()
    );
    assert_eq!(attempted, 1);
    Ok(())
}
#[test]
fn historical_auto_prefix_cannot_hide_in_newer_profiles_or_replace_authored_bounds() -> TestResult {
    let bytes =
        crate::module_packets::hex("pa31/tests/spec/provided-contracts-v1/provided-v0.hex")?;
    let interface = codec::decode(&bytes)?;
    let rows = interface
        .source_descriptor()
        .ok_or("historical rows")?
        .rows();
    let role = |role| {
        rows.roles
            .iter()
            .find(|binding| binding.role == role)
            .map(|binding| binding.declaration.declaration)
            .ok_or("actual role")
    };
    let meta = rows
        .roles
        .iter()
        .find(|binding| binding.role == LangItem::MetaSized)
        .ok_or("actual MetaSized role")?
        .declaration;
    let freeze = role(LangItem::Freeze)?;
    let header = rows
        .traits
        .iter()
        .find(|header| header.declaration == freeze)
        .ok_or("Freeze header")?;
    let inputs = header.inputs.slice(&rows.clauses)?;
    let mut paid = 0;
    let failure = auto_prefix(
        header,
        inputs,
        &rows.trait_applications,
        Some(meta),
        &mut |steps| {
            paid += steps;
            Ok(())
        },
    )
    .err()
    .ok_or("historical implicit auto prefix accepted")?;
    assert!(failure.message.contains("implicit auto-trait Self"));
    assert!(paid > 0);
    // Source-site/optional classification cannot hide the actual sizing premise.
    for clear_subject in [false, true] {
        let mut shifted = inputs.to_vec();
        if clear_subject {
            shifted[0].self_subject = None;
        } else {
            shifted[0].site = 0;
        }
        assert!(
            auto_prefix(
                header,
                &shifted,
                &rows.trait_applications,
                Some(meta),
                &mut |_| Ok(()),
            )
            .is_err()
        );
    }
    let clone = role(LangItem::Clone)?;
    let ordinary = rows
        .traits
        .iter()
        .find(|header| header.declaration == clone)
        .ok_or("Clone header")?;
    auto_prefix(
        ordinary,
        ordinary.inputs.slice(&rows.clauses)?,
        &rows.trait_applications,
        Some(meta),
        &mut |_| Ok(()),
    )?;
    other_requirements(header, inputs, &rows.trait_applications, meta)
}

fn other_requirements(
    header: &s::TraitHeader,
    inputs: &[s::Clause],
    applications: &[s::TraitApplication],
    meta: s::DeclarationAddress,
) -> TestResult {
    let mut changed = inputs.to_vec();
    for requirement in [
        s::Requirement::UnstableFeature("fixture_feature".into()),
        s::Requirement::TypeOutlives {
            subject: header.self_type,
            shorter: s::RegionRef(0),
        },
    ] {
        changed[0].requirement = requirement;
        auto_prefix(header, &changed, applications, Some(meta), &mut |_| Ok(()))?;
    }
    let s::Requirement::Trait { application, .. } = inputs[0].requirement else {
        return Err("actual historical sizing application".into());
    };
    let mut foreign = applications.to_vec();
    foreign[application.0 as usize].declaration.artifact = s::ArtifactRef(1);
    let failure = auto_prefix(header, inputs, &foreign, None, &mut |_| Ok(()))
        .err()
        .ok_or("external role absence hid a sizing premise")?;
    assert!(failure.message.contains("complete role provenance"));
    Ok(())
}
