//! Actual invocation publication is atomic; physical scopes have no side owner.
use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn physical_scopes_and_rejected_invocations_allocate_no_item_owner() -> TestResult {
    let mut introductions = Introductions::default();
    introductions.introduce_scope(ScopeId(0), ExpansionId(0));
    assert_eq!(introductions.scope(ScopeId(0)), ExpansionId(0));
    assert!(introductions.item_scopes.is_none());
    assert!(
        introductions
            .invocation(ExpansionId(2), ExpansionId(0), ScopeId(0), None)
            .is_err()
    );
    assert!(
        introductions
            .invocation(ExpansionId(1), ExpansionId(9), ScopeId(0), None)
            .is_err()
    );
    assert!(introductions.item_scopes.is_none());
    assert!(!introductions.ancestor(ExpansionId(1), ExpansionId(1)));
    let owner = Some(NodeKey {
        file: crate::resolve::unit::FileId(2),
        node: crate::syntax::ast::NodeId::new(3),
    });
    introductions.invocation(ExpansionId(1), ExpansionId(0), ScopeId(3), owner)?;
    assert!(
        introductions
            .invocation(ExpansionId(1), ExpansionId(0), ScopeId(8), None)
            .is_err()
    );
    assert_eq!(introductions.definition_scope(ExpansionId(1))?, ScopeId(3));
    assert_eq!(introductions.definition_owner(ExpansionId(1))?, owner);
    assert!(introductions.definition_owner(ExpansionId(2)).is_err());
    assert!(introductions.definition_scope(ExpansionId(2)).is_err());
    assert!(introductions.definition_scope(ExpansionId(0)).is_err());
    Ok(())
}

#[test]
fn scope_storage_is_linear_and_hot_records_do_not_gain_provenance() -> TestResult {
    for count in [8_u32, 32, 128, 512] {
        let mut introductions = Introductions::default();
        for id in 1..=count {
            introductions.invocation(ExpansionId(id), ExpansionId(id - 1), ScopeId(id), None)?;
            introductions.introduce_scope(ScopeId(id), ExpansionId(id));
        }
        let sites = introductions
            .item_scopes
            .as_ref()
            .ok_or("missing scope owner")?;
        assert_eq!(sites.definitions.len(), usize::try_from(count)?);
        assert_eq!(sites.introduced.len(), usize::try_from(count)?);
        for id in (1..=count).rev() {
            assert_eq!(
                introductions.definition_scope(ExpansionId(id))?,
                ScopeId(id)
            );
            assert_eq!(introductions.scope(ScopeId(id)), ExpansionId(id));
        }
    }
    eprintln!(
        "item scope sizes resolution={} introductions={} scope_owner={} scope={} definition={} package={} path={} saved_path={}",
        size_of::<crate::resolve::model::Resolution>(),
        size_of::<Introductions>(),
        size_of::<ItemScopes>(),
        size_of::<crate::resolve::model::Scope>(),
        size_of::<crate::resolve::model::Definition>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::resolve::paths::Buffer>(),
        size_of::<crate::resolve::paths::Saved>()
    );
    assert_eq!(size_of::<crate::resolve::model::Resolution>(), 2032);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
    assert_eq!(size_of::<crate::types::Kind>(), 24);
    assert_eq!(size_of::<DefinitionSite>(), 12);
    Ok(())
}
