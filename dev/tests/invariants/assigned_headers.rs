//! The owning activation API shares exact construction and never accepts a member as owner.
use super::*;
#[test]
fn revisited_assignment_builder_is_idempotent_and_rejects_member_as_owner()
-> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve("fn main(){let mut x=1u32;x+=&2u32;}", true)?;
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    let declaration = report
        .resolution
        .builtin_traits
        .language(Hook::AddAssign)
        .ok_or("actual demanded owner")?;
    let before = facts.types.len();
    assert_eq!(facts.associated.language_functions().count(), 1);
    for _ in 0..8 {
        prepare(&report, &mut facts, declaration.definition)?;
    }
    assert_eq!(facts.types.len(), before);
    assert_eq!(facts.associated.language_functions().count(), 1);
    assert!(prepare(&report, &mut facts, declaration.method).is_err());
    assert_eq!(facts.types.len(), before);
    Ok(())
}
