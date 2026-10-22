//! Imported storage preserves real identity and validates values before local allocation.
use rustgm::support::crates::{
    self, CrateIdentity, ExternalDef, ExternalKind, Interface, Lifetime, Nominals, Record,
    RecordField, RecordRef, Scalar, ScalarValue, Static, StaticRef, Storage, StorageOrigin,
    StoredKind, StoredValue, Symbol, TypeGraph, TypeNode, TypeRef, ValueRef,
};

fn owner() -> Result<CrateIdentity, &'static str> {
    CrateIdentity::new("storage", "one")
}
fn entry(
    name: &str,
    ty: u16,
    origin: StorageOrigin,
    public: bool,
    mutable: bool,
) -> Result<Static, &'static str> {
    Static::new(
        ExternalDef::new(owner()?, vec![name.to_owned()], ExternalKind::Static)?,
        TypeRef(ty),
        if matches!(origin, StorageOrigin::Foreign { .. }) {
            Symbol::Exact(format!("external_{name}").into())
        } else {
            Symbol::Rust
        },
        public,
        mutable,
        origin,
    )
}
fn scalar() -> Result<StoredValue, &'static str> {
    Ok(StoredValue {
        ty: TypeRef(0),
        kind: StoredKind::Scalar(ScalarValue::new(Scalar::U32, 7)?),
    })
}
fn reference(target: u32, fields: &[u32]) -> StoredValue {
    StoredValue {
        ty: TypeRef(2),
        kind: StoredKind::Reference {
            target: StaticRef(target),
            fields: fields.into(),
        },
    }
}
fn interface(entries: Vec<Static>, values: Vec<StoredValue>) -> Result<Interface, &'static str> {
    let types = TypeGraph::new(vec![
        TypeNode::Scalar(Scalar::U32),
        TypeNode::Record(RecordRef(0)),
        TypeNode::Reference {
            region: Lifetime::Static,
            mutable: false,
            referent: TypeRef(0),
        },
    ])?;
    let record = Record::new(
        ExternalDef::new(owner()?, vec!["Record".into()], ExternalKind::Record)?,
        false,
        vec![RecordField::new("field".into(), TypeRef(0), false)?],
    )?;
    Interface::with_storage(
        owner()?,
        vec![],
        vec![],
        vec![],
        Nominals::for_storage(types, vec![record])?,
        Storage::new(entries, values)?,
    )
}
fn backing() -> Result<Static, &'static str> {
    entry("A", 1, StorageOrigin::Defined(ValueRef(1)), false, false)
}
fn pointer() -> Result<Static, &'static str> {
    entry("B", 2, StorageOrigin::Defined(ValueRef(2)), true, false)
}
fn product(child: u32) -> StoredValue {
    StoredValue {
        ty: TypeRef(1),
        kind: StoredKind::Product(vec![ValueRef(child)].into_boxed_slice()),
    }
}

#[test]
fn hidden_storage_roundtrips_by_identity_and_field_path() -> Result<(), Box<dyn std::error::Error>>
{
    let artifact = interface(
        vec![backing()?, pointer()?],
        vec![scalar()?, product(0), reference(0, &[0])],
    )?;
    let bytes = crates::encode(&artifact)?;
    assert_eq!(bytes[12], 5);
    let decoded = crates::decode(&bytes)?;
    assert_eq!(decoded, artifact);
    assert_eq!(crates::encode(&decoded)?, bytes);
    let storage = decoded.storage().ok_or("missing storage")?;
    assert!(!storage.statics()[0].public());
    assert_eq!(storage.values()[2], reference(0, &[0]));
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(crates::decode(&trailing).is_err());
    Ok(())
}

#[test]
fn static_values_reject_forged_paths_types_and_unreachable_backing()
-> Result<(), Box<dyn std::error::Error>> {
    for bad in [reference(2, &[0]), reference(0, &[1]), reference(0, &[])] {
        assert!(
            interface(
                vec![backing()?, pointer()?],
                vec![scalar()?, product(0), bad]
            )
            .is_err()
        );
    }
    for child in [1, 2, u32::MAX] {
        assert!(
            interface(
                vec![backing()?, pointer()?],
                vec![scalar()?, product(child), reference(0, &[0])]
            )
            .is_err()
        );
    }
    let wrong_root = entry("B", 2, StorageOrigin::Defined(ValueRef(0)), true, false)?;
    assert!(
        interface(
            vec![backing()?, wrong_root],
            vec![scalar()?, product(0), reference(0, &[0])]
        )
        .is_err()
    );
    assert!(interface(vec![backing()?], vec![scalar()?, product(0)]).is_err());
    assert!(
        interface(
            vec![backing()?, pointer()?],
            vec![scalar()?, product(0), reference(0, &[0]), scalar()?]
        )
        .is_err()
    );
    assert!(Storage::new(vec![pointer()?, backing()?], vec![]).is_err());
    assert!(Storage::new(vec![pointer()?, pointer()?], vec![]).is_err());
    Ok(())
}

#[test]
fn scalar_only_storage_preserves_foreign_safety_mutability_and_exact_symbols()
-> Result<(), Box<dyn std::error::Error>> {
    let types =
        Nominals::for_storage(TypeGraph::new(vec![TypeNode::Scalar(Scalar::U32)])?, vec![])?;
    let entries = vec![
        entry("A", 0, StorageOrigin::Defined(ValueRef(0)), true, true)?,
        entry(
            "B",
            0,
            StorageOrigin::Foreign { unsafe_: false },
            true,
            false,
        )?,
        entry(
            "C",
            0,
            StorageOrigin::Foreign { unsafe_: true },
            true,
            false,
        )?,
        entry("D", 0, StorageOrigin::Foreign { unsafe_: true }, true, true)?,
    ];
    let artifact = Interface::with_storage(
        owner()?,
        vec![],
        vec![],
        vec![],
        types,
        Storage::new(entries, vec![scalar()?])?,
    )?;
    let decoded = crates::decode(&crates::encode(&artifact)?)?;
    assert_eq!(decoded, artifact);
    assert!(decoded.records().is_empty());
    let entries = decoded.storage().ok_or("missing storage")?.statics();
    assert!(entries[0].mutable());
    assert_eq!(entries[1].symbol(), &Symbol::Exact("external_B".into()));
    assert_ne!(entries[1].origin(), entries[2].origin());
    assert!(entries[3].mutable());
    assert!(
        Static::new(
            entries[1].definition().clone(),
            TypeRef(0),
            Symbol::Rust,
            true,
            false,
            StorageOrigin::Foreign { unsafe_: false }
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn storage_graphs_reject_open_lifetimes_and_value_namespace_collisions()
-> Result<(), Box<dyn std::error::Error>> {
    let types = Nominals::for_storage(
        TypeGraph::new(vec![
            TypeNode::Scalar(Scalar::U32),
            TypeNode::Reference {
                region: Lifetime::Bound(0),
                mutable: false,
                referent: TypeRef(0),
            },
        ])?,
        vec![],
    )?;
    let storage = Storage::new(
        vec![entry(
            "A",
            1,
            StorageOrigin::Foreign { unsafe_: true },
            true,
            false,
        )?],
        vec![],
    )?;
    assert!(Interface::with_storage(owner()?, vec![], vec![], vec![], types, storage).is_err());
    let constant = crates::Constant::new(
        ExternalDef::new(owner()?, vec!["A".into()], ExternalKind::Constant)?,
        ScalarValue::new(Scalar::U32, 7)?,
    )?;
    let types =
        Nominals::for_storage(TypeGraph::new(vec![TypeNode::Scalar(Scalar::U32)])?, vec![])?;
    let storage = Storage::new(
        vec![entry(
            "A",
            0,
            StorageOrigin::Defined(ValueRef(0)),
            true,
            false,
        )?],
        vec![scalar()?],
    )?;
    assert!(
        Interface::with_storage(owner()?, vec![], vec![], vec![constant], types, storage).is_err()
    );
    Ok(())
}
