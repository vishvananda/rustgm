//! A version-thirteen inherent impl row is rejected by name under every older
//! profile. Version thirteen admits it and charges each implementation row.
use super::{Profile, require_profile};
use crate::{
    lower::instances::SourcePublication,
    module_packets::{Result, exported_profile, raw_rows},
    support::crates::generics::Work,
};

#[test]
fn inherent_rows_require_their_wider_member_profile() -> Result {
    let source = crate::test_source::fixture("pa29/tests/spec/inherent-methods/provider.rs")?;
    let interface = exported_profile(&source, SourcePublication::ModuleBodies, "traitless")?;
    let rows = raw_rows(
        interface
            .source_descriptor()
            .ok_or("actual module descriptor")?
            .rows(),
    );
    let inherent = rows
        .implementations
        .iter()
        .filter(|row| row.trait_application.is_none())
        .count();
    assert_eq!(inherent, 3);
    let mut work = Work::default();
    require_profile(&rows, Profile::Source2024ModuleBodies, &mut work)?;
    assert_eq!(work.steps, 2 * rows.implementations.len() + 1);
    for profile in [
        Profile::Source2024ModuleBranches,
        Profile::Source2024ModuleNominals,
    ] {
        assert_eq!(
            require_profile(&rows, profile, &mut Work::default()),
            Err("inherent source impl requires its wider member profile")
        );
    }
    Ok(())
}
