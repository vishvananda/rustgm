//! Array extent is not decoder work; only real stored edges and bytes consume quotas.
use rustgm::support::crates::{
    self, CrateIdentity, ExternalDef, ExternalKind, Interface, Lifetime, Nominals, Scalar,
    ScalarValue, Static, StaticRef, Storage, StorageOrigin, StoredKind, StoredProjection,
    StoredValue, Symbol, TypeGraph, TypeNode, TypeRef, ValueRef,
};

fn artifact(
    scalar: Scalar,
    length: u64,
    values: Vec<StoredValue>,
    root: u32,
    reference: bool,
) -> Result<Interface, &'static str> {
    let owner = CrateIdentity::new("array_storage", "one")?;
    let mut types = vec![
        TypeNode::Scalar(scalar),
        TypeNode::Array {
            element: TypeRef(0),
            length,
        },
    ];
    let entry = |name: &str, ty, value| {
        Static::new(
            ExternalDef::new(owner.clone(), vec![name.into()], ExternalKind::Static)?,
            TypeRef(ty),
            Symbol::Rust,
            true,
            false,
            StorageOrigin::Defined(ValueRef(value)),
        )
    };
    let mut entries = vec![entry("A", 1, root)?];
    if reference {
        types.push(TypeNode::Reference {
            region: Lifetime::Static,
            mutable: false,
            referent: TypeRef(0),
        });
        entries.push(entry(
            "P",
            2,
            u32::try_from(values.len() - 1).map_err(|_| "values")?,
        )?);
    }
    Interface::with_storage(
        owner,
        vec![],
        vec![],
        vec![],
        Nominals::for_storage(TypeGraph::new(types)?, vec![])?,
        Storage::new(entries, values)?,
    )
}
fn scalar() -> Result<StoredValue, &'static str> {
    Ok(StoredValue {
        ty: TypeRef(0),
        kind: StoredKind::Scalar(ScalarValue::new(Scalar::U8, 7)?),
    })
}
fn array(kind: StoredKind) -> StoredValue {
    StoredValue {
        ty: TypeRef(1),
        kind,
    }
}
fn pointer(path: Vec<StoredProjection>) -> StoredValue {
    StoredValue {
        ty: TypeRef(2),
        kind: StoredKind::ProjectedReference {
            target: StaticRef(0),
            path: path.into(),
        },
    }
}
fn roundtrip(value: &Interface) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = crates::encode(value)?;
    assert_eq!(&bytes[12..14], &[7, 5]);
    assert_eq!(crates::decode(&bytes)?, *value);
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err());
    }
    Ok(bytes)
}

#[test]
fn array_values_preserve_products_repeats_and_byte_recipes()
-> Result<(), Box<dyn std::error::Error>> {
    for kind in [
        StoredKind::Product(vec![ValueRef(0); 3].into()),
        StoredKind::Repeat(ValueRef(0)),
    ] {
        roundtrip(&artifact(
            Scalar::U8,
            3,
            vec![scalar()?, array(kind)],
            1,
            false,
        )?)?;
    }
    for bytes in [vec![], vec![0, 128, 255]] {
        roundtrip(&artifact(
            Scalar::U8,
            bytes.len() as u64,
            vec![array(StoredKind::Bytes(bytes.into()))],
            0,
            false,
        )?)?;
    }
    roundtrip(&artifact(
        Scalar::U8,
        0,
        vec![array(StoredKind::Product(Box::new([])))],
        0,
        false,
    )?)?;
    Ok(())
}

#[test]
fn repeat_extents_and_element_indices_remain_full_width_constant_size_facts()
-> Result<(), Box<dyn std::error::Error>> {
    let mut size = None;
    for length in [8, 32, 128, u64::from(u32::MAX) + 2, u64::MAX] {
        let values = vec![
            StoredValue {
                ty: TypeRef(0),
                kind: StoredKind::Product(Box::new([])),
            },
            array(StoredKind::Repeat(ValueRef(0))),
            pointer(vec![StoredProjection::Element(length - 1)]),
        ];
        let value = artifact(Scalar::Unit, length, values, 1, true)?;
        let bytes = roundtrip(&value)?;
        if let Some(previous) = size {
            assert_eq!(bytes.len(), previous);
        }
        size = Some(bytes.len());
        assert_eq!(value.storage().ok_or("storage")?.values().len(), 3);
    }
    Ok(())
}

#[test]
fn array_recipes_reject_wrong_extent_element_edges_and_byte_types()
-> Result<(), Box<dyn std::error::Error>> {
    for kind in [
        StoredKind::Product(vec![ValueRef(0); 2].into()),
        StoredKind::Product(vec![ValueRef(2); 3].into()),
        StoredKind::Repeat(ValueRef(1)),
        StoredKind::Repeat(ValueRef(u32::MAX)),
        StoredKind::Bytes(vec![7; 2].into()),
    ] {
        assert!(artifact(Scalar::U8, 3, vec![scalar()?, array(kind)], 1, false).is_err());
    }
    assert!(
        artifact(
            Scalar::Bool,
            3,
            vec![array(StoredKind::Bytes(vec![1; 3].into()))],
            0,
            false
        )
        .is_err()
    );
    assert!(
        artifact(
            Scalar::U8,
            3,
            vec![
                array(StoredKind::Bytes(vec![1; 3].into())),
                array(StoredKind::Repeat(ValueRef(0)))
            ],
            1,
            false
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn array_projection_steps_cannot_be_relabelled_as_fields_or_leave_storage()
-> Result<(), Box<dyn std::error::Error>> {
    for path in [
        vec![],
        vec![StoredProjection::Field(0)],
        vec![StoredProjection::Element(3)],
        vec![StoredProjection::Element(0), StoredProjection::Element(0)],
    ] {
        assert!(
            artifact(
                Scalar::U8,
                3,
                vec![
                    scalar()?,
                    array(StoredKind::Repeat(ValueRef(0))),
                    pointer(path)
                ],
                1,
                true
            )
            .is_err()
        );
    }
    let values = vec![
        scalar()?,
        array(StoredKind::Repeat(ValueRef(0))),
        pointer(vec![StoredProjection::Element(2)]),
    ];
    let value = artifact(Scalar::U8, 3, values, 1, true)?;
    roundtrip(&value)?;
    Ok(())
}
