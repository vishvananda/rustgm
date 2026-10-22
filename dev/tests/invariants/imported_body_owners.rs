//! Source-free ownership preserves exact families without syntax or proof tokens.
use super::*;

fn root(declaration: u32, kind: BodyKind) -> BodyOwner {
    BodyOwner {
        declaration: DeclarationRef(declaration),
        parent: None,
        kind,
        ordinal: 0,
    }
}
fn child(declaration: u32, parent: u32, kind: BodyKind, ordinal: u32) -> BodyOwner {
    BodyOwner {
        parent: Some(BodyRef(parent)),
        ordinal,
        ..root(declaration, kind)
    }
}
fn owner(disambiguator: &str) -> Result<CrateIdentity, &'static str> {
    CrateIdentity::new("provider", disambiguator)
}

#[test]
fn original_families_keep_real_parent_and_defining_crate() -> Result<(), &'static str> {
    let declarations = [DeclarationBody::Function, DeclarationBody::Constant];
    let rows = vec![
        root(0, BodyKind::Function),
        root(1, BodyKind::Constant),
        child(0, 0, BodyKind::Closure, 0),
        child(0, 0, BodyKind::Anonymous, 1),
        child(1, 1, BodyKind::Closure, 0),
        child(0, 2, BodyKind::Anonymous, 0),
    ];
    let left = BodyCatalog::new(owner("one")?, &declarations, rows.clone())?;
    let right = BodyCatalog::new(owner("two")?, &declarations, rows)?;
    let identity = left.identity(BodyRef(5)).ok_or("missing child")?;
    assert_eq!(left.lookup(&identity), left.row(BodyRef(5)));
    assert_eq!(identity.body(), BodyRef(5));
    assert_eq!(identity.owner(), left.owner());
    assert!(right.lookup(&identity).is_none());
    assert_ne!(left.identity(BodyRef(2)), left.identity(BodyRef(4)));
    assert!(left.identity(BodyRef(6)).is_none());
    assert!(left.shares_storage(&left.clone()));
    assert!(!left.shares_storage(&right));
    assert_eq!(left.rows().len(), 6);
    Ok(())
}

#[test]
fn roots_include_anonymous_signature_and_type_expressions() -> Result<(), &'static str> {
    let declarations = [
        DeclarationBody::Function,
        DeclarationBody::Other,
        DeclarationBody::Static,
    ];
    let mut signature = root(0, BodyKind::Anonymous);
    signature.ordinal = 1;
    let rows = vec![
        root(0, BodyKind::Function),
        signature,
        root(1, BodyKind::Anonymous),
        root(2, BodyKind::Static),
        child(0, 1, BodyKind::Closure, 0),
    ];
    assert!(BodyCatalog::new(owner("roots")?, &declarations, rows).is_ok());
    assert!(BodyCatalog::new(owner("empty")?, &[], Vec::new()).is_ok());
    Ok(())
}

#[test]
fn bad_parent_edges_and_forged_categories_never_publish() -> Result<(), &'static str> {
    let declarations = [DeclarationBody::Function, DeclarationBody::Function];
    let original = vec![
        root(0, BodyKind::Function),
        child(0, 0, BodyKind::Closure, 0),
    ];
    for parent in [1, 2, u32::MAX] {
        let mut rows = original.clone();
        rows[1].parent = Some(BodyRef(parent));
        assert!(BodyCatalog::new(owner("bad")?, &declarations, rows).is_err());
    }
    for replacement in [
        child(1, 0, BodyKind::Closure, 0),
        child(2, 0, BodyKind::Closure, 0),
        child(0, 0, BodyKind::Function, 0),
        child(0, 0, BodyKind::Constant, 0),
        child(0, 0, BodyKind::Static, 0),
    ] {
        let mut rows = original.clone();
        rows[1] = replacement;
        assert!(BodyCatalog::new(owner("bad")?, &declarations, rows).is_err());
    }
    for kind in [BodyKind::Constant, BodyKind::Static, BodyKind::Closure] {
        assert!(BodyCatalog::new(owner("bad")?, &declarations, vec![root(0, kind)]).is_err());
    }
    assert!(BodyCatalog::new(owner("good")?, &declarations, original).is_ok());
    Ok(())
}

#[test]
fn duplicates_gaps_and_noncanonical_families_reject() -> Result<(), &'static str> {
    let declarations = [DeclarationBody::Function, DeclarationBody::Function];
    let first = root(0, BodyKind::Function);
    let second = root(1, BodyKind::Function);
    for rows in [
        vec![first, first],
        vec![second, first],
        vec![first, child(0, 0, BodyKind::Closure, 1)],
        vec![first, child(0, 0, BodyKind::Closure, 0), second],
        vec![
            first,
            second,
            child(1, 1, BodyKind::Closure, 0),
            child(0, 0, BodyKind::Closure, 0),
        ],
        vec![
            first,
            child(0, 0, BodyKind::Closure, 0),
            child(0, 0, BodyKind::Anonymous, 0),
        ],
        vec![
            first,
            child(0, 0, BodyKind::Closure, 0),
            child(0, 0, BodyKind::Anonymous, 2),
        ],
        vec![
            first,
            BodyOwner {
                ordinal: 1,
                ..first
            },
        ],
    ] {
        assert!(BodyCatalog::new(owner("bad")?, &declarations, rows).is_err());
    }
    Ok(())
}

#[test]
fn validation_work_tracks_rows_and_parent_edges_not_depth() -> Result<(), &'static str> {
    for count in [8u32, 32, 128, 32_768] {
        let mut chain = vec![root(0, BodyKind::Function)];
        let mut siblings = chain.clone();
        for index in 1..count {
            chain.push(child(0, index - 1, BodyKind::Anonymous, 0));
            siblings.push(child(0, 0, BodyKind::Anonymous, index - 1));
        }
        for rows in [&chain, &siblings] {
            let mut visits = 0usize;
            validate(&[DeclarationBody::Function], rows, || visits += 1)?;
            assert_eq!(visits, 2 * count as usize - 1);
        }
        assert!(BodyCatalog::new(owner("deep")?, &[DeclarationBody::Function], chain).is_ok());
    }
    let rows = [
        root(0, BodyKind::Function),
        child(0, 1, BodyKind::Closure, 0),
    ];
    let mut visits = 0;
    assert!(validate(&[DeclarationBody::Function], &rows, || visits += 1).is_err());
    assert_eq!(visits, 3);
    Ok(())
}

#[test]
fn preflight_bounds_conversion_peak_without_constructing_large_tables() {
    assert!(preflight(1, 1, 1).is_ok());
    assert!(preflight(BodyCatalog::MAX_DECLARATIONS + 1, 0, 0).is_err());
    assert!(preflight(1, BodyCatalog::MAX_BODIES + 1, 0).is_err());
    assert!(preflight(1, 1, usize::MAX).is_err());
    assert!(preflight(1, 1, MAX_BYTES / size_of::<BodyOwner>()).is_err());
    assert!(preflight(1, BodyCatalog::MAX_BODIES, BodyCatalog::MAX_BODIES).is_ok());
}
