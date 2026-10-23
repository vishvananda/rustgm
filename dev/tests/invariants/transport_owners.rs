//! Artifact data must never manufacture syntax or checked execution authority.
use super::catalog::{Binding, CatalogSlot, Expected, Limits};
use super::*;
use crate::{
    resolve::{
        model::{Access, DefId, DefKind, Definition, Namespace, ScopeId},
        names::NameId,
        unit::FileId,
    },
    support::crates::{
        CrateIdentity,
        owner_catalog::{
            BodyCatalog, BodyKind, BodyOwner, BodyRef, DeclarationBody, DeclarationRef,
        },
    },
};

pub(super) fn definitions(count: u32) -> Vec<Definition> {
    (0..count)
        .map(|index| Definition {
            name: NameId(index),
            namespace: Namespace::Value,
            kind: DefKind::Function,
            scope: ScopeId(0),
            node: None,
            access: Access::ExternalPrivate,
            members: None,
        })
        .collect()
}
pub(super) fn input(name: &str, first: u32, count: u32) -> Result<Input, &'static str> {
    let owners = BodyCatalog::new(
        CrateIdentity::new("provider", name)?,
        &vec![DeclarationBody::Function; count as usize],
        (0..count)
            .map(|index| BodyOwner {
                declaration: DeclarationRef(index),
                parent: None,
                kind: BodyKind::Function,
                ordinal: 0,
            })
            .collect(),
    )?;
    Ok(Input {
        owners,
        edition: Edition::Rust2021,
        definitions: (first..first + count).map(DefId).collect(),
        declarations: (0..count)
            .map(|index| DeclarationBinding {
                kind: DefKind::Function,
                sites: index..index + 1,
            })
            .collect(),
        bindings: (0..count)
            .map(|index| Binding {
                unit: BodyRef(index),
                sites: index..index + 1,
            })
            .collect(),
        files: vec![ProviderFile {
            path: "provider.rs".into(),
            bytes: 80,
        }],
        labels: vec![
            Some(Label {
                file: 0,
                start: 2,
                end: 8
            });
            count as usize
        ],
        declaration_labels: vec![None; count as usize],
    })
}
fn expected(definition: u32, unit: ImportedBodyId) -> Expected {
    Expected {
        definition: DefId(definition),
        kind: BodyKind::Function,
        parent: None,
        unit,
    }
}

#[test]
fn live_and_retired_catalogs_never_share_rejection_identity() -> Result<(), &'static str> {
    let defs = definitions(1);
    let mut left = Catalog::new(Limits::default())?;
    let body = left
        .install(&defs, input("same", 0, 1)?)?
        .body(BodyRef(0))?;
    let mut right = Catalog::new(Limits::default())?;
    let other = right
        .install(&defs, input("same", 0, 1)?)?
        .body(BodyRef(0))?;
    assert_ne!(body, other);
    assert!(right.validate(body, &expected(0, body)).is_err());
    let stale = body;
    drop(left);
    let mut replacement = Catalog::new(Limits::default())?;
    replacement.install(&defs, input("same", 0, 1)?)?;
    assert!(replacement.site(stale, 0).is_err());
    assert_eq!(format!("{body:?}"), "ImportedBodyId(..)");
    let exhausted = AtomicU32::new(u32::MAX - 1);
    assert_eq!(Epoch::allocate(&exhausted)?.0.get(), u32::MAX - 1);
    assert!(Epoch::allocate(&exhausted).is_err());
    assert_eq!(exhausted.load(Ordering::Relaxed), u32::MAX);
    assert!(Epoch::allocate(&exhausted).is_err());
    Ok(())
}

#[test]
fn source_nodes_and_provider_labels_remain_separate() -> Result<(), &'static str> {
    let node = NodeId::new(3);
    let source = NodeKey {
        file: FileId(9),
        node,
    };
    let origin = BodyOrigin::Source(source);
    assert_eq!(origin.source_site(node)?.source(), Some(source));
    assert_eq!(BodySite::source(node)?.index(), node.get());
    let mut catalog = Catalog::new(Limits::default())?;
    let body = catalog
        .install(&definitions(1), input("sites", 0, 1)?)?
        .body(BodyRef(0))?;
    assert!(BodyOrigin::Imported(body).source().is_none());
    assert!(BodyOrigin::Imported(body).source_site(node).is_err());
    let site = catalog.proof_site(body, catalog.site(body, 0)?)?;
    assert!(site.source().is_none());
    let location = catalog.location(site)?.ok_or("missing diagnostic label")?;
    assert_eq!(
        (location.file, location.start, location.end),
        ("provider.rs", 2, 8)
    );
    assert!(catalog.site(body, 1).is_err());
    let missing = ImportedBodyId {
        row: u32::MAX,
        ..body
    };
    assert!(catalog.site(missing, 0).is_err());
    assert!(BodySite::checked(u32::MAX).is_err());
    assert!(catalog.location(ProofSite::Source(source))?.is_none());
    Ok(())
}

#[test]
fn exact_remaps_and_failed_installs_are_transactional() -> Result<(), &'static str> {
    let defs = definitions(3);
    let mut catalog = Catalog::new(Limits::default())?;
    let first = catalog
        .install(&defs, input("one", 0, 1)?)?
        .body(BodyRef(0))?;
    let bytes = catalog.retained_bytes();
    let mut bad = input("two", 1, 2)?;
    bad.definitions[1] = DefId(1);
    assert!(catalog.install(&defs, bad).is_err());
    assert_eq!(
        (
            catalog.rows.len(),
            catalog.artifacts.len(),
            catalog.retained_bytes()
        ),
        (1, 1, bytes)
    );
    assert!(catalog.install(&defs, input("two", 0, 1)?).is_err());
    assert!(catalog.install(&defs, input("one", 1, 1)?).is_err());
    let second = catalog
        .install(&defs, input("two", 1, 1)?)?
        .body(BodyRef(0))?;
    assert!(catalog.validate(first, &expected(1, first)).is_err());
    assert!(catalog.validate(first, &expected(0, second)).is_err());
    assert_eq!(catalog.coordinate(second)?.1, BodyRef(0));
    let mut bad_defs = definitions(3);
    bad_defs[2].kind = DefKind::Const;
    assert!(catalog.install(&bad_defs, input("three", 2, 1)?).is_err());
    bad_defs[2].kind = DefKind::Function;
    bad_defs[2].node = Some(NodeKey {
        file: FileId(0),
        node: NodeId::new(0),
    });
    assert!(catalog.install(&bad_defs, input("three", 2, 1)?).is_err());
    assert!(catalog.install(&defs, input("three", 3, 1)?).is_err());
    assert_eq!(catalog.rows.len(), 2);
    Ok(())
}

#[test]
fn invalid_site_partitions_and_labels_never_publish() -> Result<(), &'static str> {
    let defs = definitions(2);
    for case in 0..9 {
        let mut catalog = Catalog::new(Limits::default())?;
        let mut bad = input("bad", 0, 2)?;
        match case {
            0 => bad.bindings[0].sites = 1..2,
            1 => bad.bindings[1].sites = 0..1,
            2 => bad.bindings[0].sites = 0..0,
            3 => bad.bindings[1].sites = 1..3,
            4 => bad.labels.push(None),
            5 => {
                bad.labels[0] = Some(Label {
                    file: 4,
                    start: 0,
                    end: 0,
                });
            }
            6 => {
                bad.labels[0] = Some(Label {
                    file: 0,
                    start: 8,
                    end: 2,
                });
            }
            7 => {
                bad.labels[0] = Some(Label {
                    file: 0,
                    start: 0,
                    end: 81,
                });
            }
            _ => bad.files[0].path = "bad\npath".into(),
        }
        assert!(catalog.install(&defs, bad).is_err());
        assert!(
            catalog.rows.is_empty()
                && catalog.artifacts.is_empty()
                && catalog.definitions.is_empty()
        );
        assert!(catalog.work() > 0);
        assert!(catalog.install(&defs, input("bad", 0, 2)?).is_ok());
    }
    Ok(())
}

#[test]
fn unit_ownership_requires_the_exact_parent_partition() -> Result<(), &'static str> {
    let defs = definitions(1);
    for foreign in [false, true] {
        let mut request = input("units", 0, 1)?;
        request.owners = BodyCatalog::new(
            CrateIdentity::new("provider", "units")?,
            &[DeclarationBody::Function],
            vec![
                BodyOwner {
                    declaration: DeclarationRef(0),
                    parent: None,
                    kind: BodyKind::Function,
                    ordinal: 0,
                },
                BodyOwner {
                    declaration: DeclarationRef(0),
                    parent: Some(BodyRef(0)),
                    kind: BodyKind::Closure,
                    ordinal: 0,
                },
                BodyOwner {
                    declaration: DeclarationRef(0),
                    parent: Some(BodyRef(0)),
                    kind: BodyKind::Anonymous,
                    ordinal: 1,
                },
            ],
        )?;
        request.bindings.extend([
            Binding {
                unit: BodyRef(1),
                sites: 1..2,
            },
            Binding {
                unit: BodyRef(u32::from(foreign)),
                sites: 2..3,
            },
        ]);
        request.labels.extend([None, None]);
        let mut catalog = Catalog::new(Limits::default())?;
        let installed = catalog.install(&defs, request);
        if foreign {
            assert!(installed.is_err());
        } else {
            let installed = installed?;
            let root = installed.body(BodyRef(0))?;
            let child = installed.body(BodyRef(2))?;
            assert_eq!(catalog.get(child)?.unit, root);
            assert!(
                catalog
                    .validate(
                        child,
                        &Expected {
                            definition: DefId(0),
                            kind: BodyKind::Anonymous,
                            parent: None,
                            unit: root
                        }
                    )
                    .is_err()
            );
        }
    }
    Ok(())
}

#[test]
fn quotas_are_preflighted_and_warm_queries_are_constant_work() -> Result<(), &'static str> {
    let mut slot = CatalogSlot::default();
    assert!(slot.get().is_none());
    assert_eq!(slot.ensure()?.rows.len(), 0);
    for limits in [
        Limits {
            bytes: size_of::<Catalog>() + 1,
            ..Limits::default()
        },
        Limits {
            steps: 1,
            ..Limits::default()
        },
        Limits {
            bodies: 0,
            ..Limits::default()
        },
        Limits {
            artifacts: 0,
            ..Limits::default()
        },
    ] {
        let mut catalog = Catalog::new(limits)?;
        assert!(
            catalog
                .install(&definitions(1), input("budget", 0, 1)?)
                .is_err()
        );
        assert!(catalog.rows.is_empty() && catalog.artifacts.is_empty());
        assert!(catalog.work() > 0);
    }
    for noise in [0, 128, 4096] {
        let defs = definitions(noise + 128);
        for width in [8, 32, 128] {
            let mut catalog = Catalog::new(Limits::default())?;
            if noise != 0 {
                catalog.install(&defs, input("noise", 0, noise)?)?;
            }
            let start = catalog.work();
            let installed = catalog.install(&defs, input("wanted", noise, width)?)?;
            assert_eq!(catalog.work() - start, 28 + 4 * width as usize);
            let body = installed.body(BodyRef(width - 1))?;
            let retained = catalog.retained_bytes();
            let start = catalog.work();
            for _ in 0..width {
                catalog.validate(body, &expected(noise + width - 1, body))?;
                catalog.coordinate(body)?;
                catalog.site(body, 0)?;
            }
            assert_eq!(catalog.work() - start, 4 * width as usize);
            assert_eq!(catalog.retained_bytes(), retained);
        }
    }
    Ok(())
}

#[test]
fn simultaneous_catalog_epochs_are_unique() -> Result<(), &'static str> {
    let workers: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| Catalog::new(Limits::default()).map(|catalog| catalog.epoch.0))
        })
        .collect();
    let mut epochs = crate::support::hash::HashSet::default();
    for worker in workers {
        let epoch = worker
            .join()
            .map_err(|_| "catalog identity worker failed")??;
        assert!(epochs.insert(epoch));
    }
    Ok(())
}
