//! The conversion role uses an explicit new tag, never a legacy alias.
use super::*;
use crate::support::crates::{MAX_BYTES, generics::source::Profile};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn binding(role: LangItem) -> s::RoleBinding {
    s::RoleBinding {
        role,
        declaration: s::DeclarationAddress {
            artifact: s::ArtifactRef(0),
            declaration: DeclarationRef(7),
        },
    }
}
fn read(bytes: &[u8], profile: Profile) -> std::result::Result<s::RoleBinding, &'static str> {
    let mut reader = super::super::Reader::new(bytes);
    role_read(&mut Input {
        reader: &mut reader,
        bytes: 0,
        base: 0,
        profile,
    })
}
#[test]
fn from_tag_requires_version_nine_without_changing_legacy_roles() -> Result {
    let role = binding(LangItem::From);
    let mut legacy = Writer::source(0);
    assert!(role_write(&mut legacy, &role).is_err());
    assert!(legacy.finish().is_empty());
    let mut writer = Writer::source(0);
    conversion_role_write(&mut writer, &role)?;
    let bytes = writer.finish();
    assert_eq!(bytes, [9, 0, 0, 0, 0, 7, 0, 0, 0]);
    let decoded = read(&bytes, Profile::Source2024ModuleConversions)?;
    assert_eq!(decoded.role, role.role);
    assert_eq!(decoded.declaration, role.declaration);
    for profile in [
        Profile::Source2024Functions,
        Profile::Source2024FunctionContracts,
        Profile::Source2024ModuleContracts,
        Profile::Source2024ModuleAvailability,
        Profile::Source2024ModuleFeaturePredicates,
        Profile::Source2024ModuleDefinitions,
        Profile::Source2024ModuleCompatibility,
        Profile::Source2024ModuleCallables,
        Profile::Source2024ModuleSafety,
        Profile::Source2024Provided,
        Profile::Source2024ProvidedContracts,
        Profile::Source2024ProvidedCurrent,
    ] {
        assert!(!profile.conversion_roles());
        assert!(read(&bytes, profile).is_err(), "{profile:?}");
    }
    Ok(())
}
#[test]
fn older_role_bytes_are_identical_in_the_conversion_encoder() -> Result {
    for (index, role) in [
        LangItem::Sized,
        LangItem::MetaSized,
        LangItem::PointeeSized,
        LangItem::LegacyReceiver,
        LangItem::Clone,
        LangItem::CloneFn,
        LangItem::Copy,
        LangItem::Destruct,
        LangItem::Freeze,
    ]
    .into_iter()
    .enumerate()
    {
        let row = binding(role);
        let mut legacy = Writer::source(0);
        role_write(&mut legacy, &row)?;
        let mut current = Writer::source(0);
        conversion_role_write(&mut current, &row)?;
        let legacy = legacy.finish();
        assert_eq!(legacy, current.finish());
        assert_eq!(usize::from(legacy[0]), index);
        assert_eq!(read(&legacy, Profile::Source2024ModuleSafety)?.role, role);
        assert_eq!(
            read(&legacy, Profile::Source2024ModuleConversions)?.role,
            role
        );
    }
    Ok(())
}
#[test]
fn conversion_role_payment_and_truncation_fail_before_publication() -> Result {
    let row = binding(LangItem::From);
    let mut denied = Writer::source(MAX_BYTES);
    assert!(conversion_role_write(&mut denied, &row).is_err());
    assert!(denied.finish().is_empty());
    let mut writer = Writer::source(0);
    conversion_role_write(&mut writer, &row)?;
    let bytes = writer.finish();
    for end in 0..bytes.len() {
        assert!(read(&bytes[..end], Profile::Source2024ModuleConversions).is_err());
    }
    let profile = Profile::Source2024ModuleConversions;
    assert!(profile.call_safety() && profile.unconditional_callables());
    assert!(profile.module_contracts() && profile.availability());
    assert!(profile.definitions() && profile.module_compatibility());
    Ok(())
}
