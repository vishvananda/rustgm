//! Declaration clauses have real owners even when no executable body exists.
use super::{
    catalog::Limits,
    tests::{definitions, input},
    *,
};
use crate::{
    resolve::model::{DefId, DefKind},
    support::crates::owner_catalog::{BodyCatalog, DeclarationBody},
};

#[test]
fn declarations_without_bodies_retain_exact_clause_ownership() -> Result<(), &'static str> {
    let mut defs = definitions(2);
    defs[0].kind = DefKind::Trait;
    defs[1].kind = DefKind::SelfType;
    let mut request = input("declarations", 0, 2)?;
    request.owners = BodyCatalog::new(
        request.owners.owner().clone(),
        &[DeclarationBody::Other; 2],
        Vec::new(),
    )?;
    request.bindings.clear();
    request.labels.clear();
    request.declarations[0].kind = DefKind::Trait;
    request.declarations[1].kind = DefKind::SelfType;
    request.declaration_labels[0] = Some(Label {
        file: 0,
        start: 11,
        end: 19,
    });
    let mut catalog = Catalog::new(Limits::default())?;
    catalog.install(&defs, request)?;
    assert!(catalog.rows.is_empty());
    let owner = catalog.declaration(DefId(0))?;
    assert!(
        catalog
            .validate_declaration(owner, DefId(0), DefKind::Trait)?
            .source()
            .is_none()
    );
    assert!(
        catalog
            .validate_declaration(owner, DefId(1), DefKind::Trait)
            .is_err()
    );
    assert!(
        catalog
            .validate_declaration(owner, DefId(0), DefKind::SelfType)
            .is_err()
    );
    let site = catalog.declaration_site(owner, 0)?;
    let proof = catalog.declaration_proof_site(owner, site)?;
    assert!(proof.source().is_none());
    let label = catalog
        .location(proof)?
        .ok_or("missing declaration label")?;
    assert_eq!(
        (label.file, label.start, label.end),
        ("provider.rs", 11, 19)
    );
    assert!(catalog.declaration_site(owner, 1).is_err());
    assert!(catalog.declaration(DefId(2)).is_err());
    let (identity, ordinal, edition) = catalog.declaration_coordinate(owner)?;
    assert_eq!(identity.disambiguator(), "declarations");
    assert_eq!(ordinal.0, 0);
    assert_eq!(edition, Edition::Rust2021);
    assert_eq!(format!("{owner:?}"), "ImportedDeclarationId(..)");
    Ok(())
}

#[test]
fn equal_remapped_ids_do_not_cross_live_or_retired_catalogs() -> Result<(), &'static str> {
    let defs = definitions(1);
    let mut left = Catalog::new(Limits::default())?;
    left.install(&defs, input("same", 0, 1)?)?;
    let owner = left.declaration(DefId(0))?;
    let site = left.declaration_site(owner, 0)?;
    let mut right = Catalog::new(Limits::default())?;
    right.install(&defs, input("same", 0, 1)?)?;
    assert_ne!(owner, right.declaration(DefId(0))?);
    assert!(
        right
            .validate_declaration(owner, DefId(0), DefKind::Function)
            .is_err()
    );
    assert!(right.declaration_proof_site(owner, site).is_err());
    drop(left);
    let mut replacement = Catalog::new(Limits::default())?;
    replacement.install(&defs, input("same", 0, 1)?)?;
    assert!(replacement.declaration_coordinate(owner).is_err());
    Ok(())
}

#[test]
fn malformed_declaration_partitions_never_publish_partial_owners() -> Result<(), &'static str> {
    let defs = definitions(2);
    for case in 0..8 {
        let mut request = input("invalid", 0, 2)?;
        match case {
            0 => {
                request.declarations.pop();
            }
            1 => request.declarations[0].kind = DefKind::Trait,
            2 => request.declarations[0].sites = 1..2,
            3 => request.declarations[0].sites = 0..0,
            4 => request.declarations[1].sites = 0..1,
            5 => request.declarations[1].sites = 1..3,
            6 => request.declaration_labels.push(None),
            _ => {
                request.declaration_labels[0] = Some(Label {
                    file: 0,
                    start: 0,
                    end: 81,
                });
            }
        }
        let mut catalog = Catalog::new(Limits::default())?;
        let bytes = catalog.retained_bytes();
        assert!(catalog.install(&defs, request).is_err());
        assert!(
            catalog.rows.is_empty()
                && catalog.definitions.is_empty()
                && catalog.artifacts.is_empty()
        );
        assert_eq!(catalog.retained_bytes(), bytes);
        assert!(catalog.work() > 0);
    }
    Ok(())
}

#[test]
fn declaration_lookup_work_is_indexed_and_storage_stays_cold() -> Result<(), &'static str> {
    assert_eq!(size_of::<ImportedDeclarationId>(), 8);
    assert_eq!(size_of::<DeclarationOrigin>(), 12);
    assert_eq!(size_of::<DeclarationSite>(), 4);
    assert_eq!(size_of::<ProofSite>(), 16);
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
    assert_eq!(size_of::<catalog::CatalogSlot>(), size_of::<usize>());
    for width in [8, 32, 128] {
        let mut catalog = Catalog::new(Limits::default())?;
        catalog.install(&definitions(width), input("lookup", 0, width)?)?;
        let start = catalog.work();
        for index in 0..width {
            let id = catalog.declaration(DefId(index))?;
            catalog.validate_declaration(id, DefId(index), DefKind::Function)?;
            catalog.declaration_site(id, 0)?;
        }
        assert_eq!(catalog.work() - start, 4 * width as usize);
    }
    Ok(())
}
