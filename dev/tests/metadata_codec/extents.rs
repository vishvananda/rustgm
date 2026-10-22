//! Fixed extents are type facts, never decoder allocation counts.
use rustgm::support::crates::{
    self, Abi, CrateIdentity, ExternalDef, ExternalKind, Function, Interface, Lifetime, Nominals,
    PublicBinding, PublicModule, PublicNames, Record, RecordField, RecordRef, RecordRepr, Scalar,
    Signature, Symbol, TypeGraph, TypeNode, TypeRef,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn identity() -> Result<CrateIdentity> {
    Ok(CrateIdentity::new("arrays", "one")?)
}
fn array(element: u16, length: u64) -> TypeNode {
    TypeNode::Array {
        element: TypeRef(element),
        length,
    }
}
fn function(signature: Signature) -> Result<Function> {
    Ok(Function::new(
        ExternalDef::new(identity()?, vec!["pass".into()], ExternalKind::Function)?,
        signature,
        Symbol::Rust,
    )?)
}
fn signature(length: u64) -> Result<Signature> {
    Ok(Signature::with_types(
        vec![TypeNode::Scalar(Scalar::U8), array(0, length)],
        0,
        vec![TypeRef(1)],
        TypeRef(1),
        false,
    )?)
}
fn artifact(length: u64) -> Result<Interface> {
    Ok(Interface::new(
        identity()?,
        vec![],
        vec![function(signature(length)?)?],
    )?)
}

#[test]
fn extents_keep_constant_wire_size_and_compose_with_public_names() -> Result {
    let mut size = None;
    for length in [0, 8, 32, 128, u64::MAX] {
        let interface = artifact(length)?;
        let bytes = crates::encode(&interface)?;
        assert_eq!(&bytes[12..14], &[7, 2]);
        assert_eq!(*size.get_or_insert(bytes.len()), bytes.len());
        assert_eq!(crates::decode(&bytes)?, interface);
        let names = PublicNames::new(vec![PublicModule::new(
            vec![],
            vec![PublicBinding::new(
                "renamed".into(),
                interface.functions()[0].definition().clone(),
            )?],
        )?])?;
        let exposed = interface.with_public_names(names)?;
        let bytes = crates::encode(&exposed)?;
        assert_eq!(&bytes[12..15], &[7, 6, 2]);
        assert_eq!(crates::decode(&bytes)?, exposed);
    }
    Ok(())
}

#[test]
fn array_edges_preserve_bound_lifetimes_and_canonical_order() -> Result {
    let graph = vec![
        TypeNode::Scalar(Scalar::U8),
        TypeNode::Reference {
            region: Lifetime::Bound(0),
            mutable: true,
            referent: TypeRef(0),
        },
        array(1, 2),
    ];
    let valid = Signature::with_types(graph.clone(), 1, vec![TypeRef(2)], TypeRef(2), false)?;
    let interface = Interface::new(identity()?, vec![], vec![function(valid)?])?;
    assert_eq!(crates::decode(&crates::encode(&interface)?)?, interface);
    assert!(Signature::with_types(graph.clone(), 0, vec![TypeRef(2)], TypeRef(2), false).is_err());
    assert!(Signature::with_types(graph, 1, vec![TypeRef(0)], TypeRef(2), false).is_err());
    assert!(TypeGraph::new(vec![array(0, 0)]).is_err());
    assert!(TypeGraph::new(vec![array(1, 1), TypeNode::Scalar(Scalar::U8)]).is_err());
    assert!(TypeGraph::new(vec![TypeNode::Scalar(Scalar::U8), array(0, 1), array(0, 1)]).is_err());
    assert!(
        Signature::with_types(
            vec![TypeNode::Scalar(Scalar::U8), array(0, 1)],
            0,
            vec![TypeRef(0)],
            TypeRef(0),
            false
        )
        .is_err()
    );
    Ok(())
}

fn record(field: u16, repr: RecordRepr) -> Result<Record> {
    Ok(Record::new(
        ExternalDef::new(identity()?, vec!["Packet".into()], ExternalKind::Record)?,
        true,
        vec![RecordField::new("items".into(), TypeRef(field), true)?],
    )?
    .with_repr(repr))
}

#[test]
fn arrays_preserve_storage_cycles_even_when_the_length_is_zero() -> Result {
    for length in [0, 2, u64::MAX] {
        let graph = TypeGraph::new(vec![TypeNode::Record(RecordRef(0)), array(0, length)])?;
        assert!(
            Nominals::new(graph, vec![record(1, RecordRepr::Rust)?])
                .is_err_and(|error| error.contains("cyclic by-value"))
        );
    }
    let graph = TypeGraph::new(vec![
        TypeNode::Record(RecordRef(0)),
        array(0, 0),
        TypeNode::Reference {
            region: Lifetime::Static,
            mutable: false,
            referent: TypeRef(1),
        },
    ])?;
    let interface = Interface::with_nominals(
        identity()?,
        vec![],
        vec![],
        vec![],
        Nominals::new(graph, vec![record(2, RecordRepr::Rust)?])?,
    )?;
    assert_eq!(crates::decode(&crates::encode(&interface)?)?, interface);
    Ok(())
}

#[test]
fn record_representation_is_separate_from_callable_abi() -> Result {
    for abi in [Abi::Rust, Abi::C] {
        let graph = TypeGraph::new(vec![
            TypeNode::Record(RecordRef(0)),
            TypeNode::Scalar(Scalar::U64),
        ])?;
        let signature =
            Signature::with_shared_types(graph.clone(), 0, vec![TypeRef(0)], TypeRef(0), false)?
                .with_abi(abi);
        let interface = Interface::with_nominals(
            identity()?,
            vec![],
            vec![function(signature)?],
            vec![],
            Nominals::new(graph, vec![record(1, RecordRepr::C)?])?,
        )?;
        let bytes = crates::encode(&interface)?;
        assert_eq!(&bytes[12..14], &[7, if abi == Abi::Rust { 3 } else { 4 }]);
        assert_eq!(crates::decode(&bytes)?, interface);
        let offset = bytes
            .windows(6)
            .position(|part| part == b"Packet")
            .ok_or("record path")?
            + 6;
        let mut invalid = bytes;
        invalid[offset] = 2;
        assert!(crates::decode(&invalid).is_err_and(|error| error.contains("representation")));
    }
    Ok(())
}

#[test]
fn extended_wire_rejects_truncation_forward_edges_and_redundant_envelopes() -> Result {
    let bytes = crates::encode(&artifact(31)?)?;
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    let mut legacy = bytes.clone();
    legacy.remove(12);
    assert!(crates::decode(&legacy).is_err());
    let mut nested = bytes.clone();
    nested.insert(12, 7);
    assert!(crates::decode(&nested).is_err());
    let mut array_bytes = vec![3, 0, 0];
    array_bytes.extend_from_slice(&31u64.to_le_bytes());
    let start = bytes
        .windows(array_bytes.len())
        .position(|part| part == array_bytes)
        .ok_or("array node")?;
    let mut forward = bytes;
    forward[start + 1] = 1;
    assert!(crates::decode(&forward).is_err_and(|error| error.contains("forward or cyclic")));
    let scalar = Interface::new(
        identity()?,
        vec![],
        vec![function(Signature::new(
            vec![Scalar::U8],
            Scalar::U8,
            false,
        )?)?],
    )?;
    let mut redundant = crates::encode(&scalar)?;
    redundant.insert(12, 7);
    assert!(crates::decode(&redundant).is_err_and(|error| error.contains("requires array")));
    Ok(())
}

#[test]
fn shared_array_graphs_grow_with_nodes_instead_of_extents() -> Result {
    for width in [8u16, 32, 128] {
        let mut nodes = vec![TypeNode::Scalar(Scalar::U8)];
        nodes.extend((0..width).map(|n| array(0, u64::MAX - u64::from(n))));
        let parameters = (1..=width).map(TypeRef).collect();
        let signature = Signature::with_types(nodes, 0, parameters, TypeRef(width), false)?;
        let interface = Interface::new(identity()?, vec![], vec![function(signature)?])?;
        let bytes = crates::encode(&interface)?;
        assert!(bytes.len() < 256 + usize::from(width) * 14);
        let decoded = crates::decode(&bytes)?;
        assert_eq!(
            decoded.functions()[0].signature().types().nodes().len(),
            usize::from(width) + 1
        );
        assert_eq!(decoded, interface);
    }
    Ok(())
}

#[test]
fn storage_envelopes_preserve_c_values_foreign_arrays_and_public_aliases() -> Result {
    let types = TypeGraph::new(vec![
        TypeNode::Scalar(Scalar::U8),
        TypeNode::Record(RecordRef(0)),
        array(0, 128),
    ])?;
    let nominal = record(0, RecordRepr::C)?;
    let entries = vec![
        crates::Static::new(
            ExternalDef::new(identity()?, vec!["A".into()], ExternalKind::Static)?,
            TypeRef(1),
            Symbol::Rust,
            true,
            false,
            crates::StorageOrigin::Defined(crates::ValueRef(1)),
        )?,
        crates::Static::new(
            ExternalDef::new(identity()?, vec!["B".into()], ExternalKind::Static)?,
            TypeRef(2),
            Symbol::Exact("external_bytes".into()),
            true,
            true,
            crates::StorageOrigin::Foreign { unsafe_: true },
        )?,
    ];
    let mut aliases = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            PublicBinding::new(format!("alias{index}"), entry.definition().clone())
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    aliases.push(PublicBinding::new(
        "Packet".into(),
        nominal.definition().clone(),
    )?);
    let values = vec![
        crates::StoredValue {
            ty: TypeRef(0),
            kind: crates::StoredKind::Scalar(crates::ScalarValue::new(Scalar::U8, 7)?),
        },
        crates::StoredValue {
            ty: TypeRef(1),
            kind: crates::StoredKind::Product(vec![crates::ValueRef(0)].into_boxed_slice()),
        },
    ];
    let interface = Interface::with_storage(
        identity()?,
        vec![],
        vec![],
        vec![],
        Nominals::for_storage(types, vec![nominal])?,
        crates::Storage::new(entries, values)?,
    )?;
    let bytes = crates::encode(&interface)?;
    assert_eq!(&bytes[12..14], &[7, 5]);
    assert_eq!(crates::decode(&bytes)?, interface);
    let named = interface
        .with_public_names(PublicNames::new(vec![PublicModule::new(vec![], aliases)?])?)?;
    let bytes = crates::encode(&named)?;
    assert_eq!(&bytes[12..15], &[7, 6, 5]);
    assert_eq!(crates::decode(&bytes)?, named);
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    Ok(())
}
