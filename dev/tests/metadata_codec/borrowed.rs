//! A validated reference graph cannot smuggle local type or lifetime identities.
use rustgm::support::crates::{
    self, CrateIdentity, ExternalDef, ExternalKind, Function, Interface, Lifetime, Scalar,
    Signature, Symbol, TypeGraph, TypeNode, TypeRef,
};
fn reference(slot: u16, referent: u16) -> TypeNode {
    TypeNode::Reference {
        region: Lifetime::Bound(slot),
        mutable: false,
        referent: TypeRef(referent),
    }
}
fn signature() -> Result<Signature, &'static str> {
    Signature::with_types(
        vec![
            TypeNode::Scalar(Scalar::U8),
            reference(0, 0),
            reference(1, 0),
        ],
        3,
        vec![TypeRef(1), TypeRef(2)],
        TypeRef(1),
        false,
    )
}
fn artifact(signature: Signature) -> Result<Interface, &'static str> {
    let owner = CrateIdentity::new("borrowed", "wire")?;
    let function = Function::new(
        ExternalDef::new(owner.clone(), vec!["f".into()], ExternalKind::Function)?,
        signature,
        Symbol::Rust,
    )?;
    Interface::new(owner, vec![], vec![function])
}
#[test]
fn typed_signatures_roundtrip_and_preserve_unused_binder_slots()
-> Result<(), Box<dyn std::error::Error>> {
    let original = artifact(signature()?)?;
    let encoded = crates::encode(&original)?;
    assert_eq!(encoded[12], 2);
    let decoded = crates::decode(&encoded)?;
    assert_eq!(decoded, original);
    assert_eq!(decoded.functions()[0].signature().bound_regions(), 3);
    assert_eq!(crates::encode(&decoded)?, encoded);
    for end in 0..encoded.len() {
        assert!(crates::decode(&encoded[..end]).is_err(), "prefix {end}");
    }
    Ok(())
}
#[test]
fn malformed_signature_graphs_fail_before_semantic_interning()
-> Result<(), Box<dyn std::error::Error>> {
    let scalar = TypeNode::Scalar(Scalar::U8);
    let cases = [
        (vec![], 0, vec![], TypeRef(0)),
        (vec![reference(0, 0)], 1, vec![TypeRef(0)], TypeRef(0)),
        (
            vec![scalar, reference(0, 2)],
            1,
            vec![TypeRef(1)],
            TypeRef(1),
        ),
        (vec![scalar, scalar], 0, vec![TypeRef(0)], TypeRef(1)),
        (
            vec![scalar, TypeNode::Scalar(Scalar::U16)],
            0,
            vec![TypeRef(0)],
            TypeRef(0),
        ),
        (
            vec![scalar, TypeNode::Scalar(Scalar::U16)],
            0,
            vec![TypeRef(1)],
            TypeRef(0),
        ),
        (vec![scalar], 0, vec![TypeRef(7)], TypeRef(0)),
        (
            vec![scalar, reference(1, 0)],
            1,
            vec![TypeRef(1)],
            TypeRef(1),
        ),
        (
            vec![scalar, reference(0, 0), reference(1, 0)],
            2,
            vec![TypeRef(1)],
            TypeRef(2),
        ),
    ];
    for (nodes, slots, parameters, result) in cases {
        assert!(Signature::with_types(nodes, slots, parameters, result, false).is_err());
    }
    assert!(
        Signature::with_types(
            vec![scalar; TypeGraph::MAX_NODES + 1],
            0,
            vec![],
            TypeRef(0),
            false
        )
        .is_err()
    );
    assert!(Signature::with_types(vec![scalar], 257, vec![], TypeRef(0), false).is_err());
    let mut nodes = vec![scalar];
    for i in 0..65u16 {
        nodes.push(TypeNode::Reference {
            region: Lifetime::Static,
            mutable: false,
            referent: TypeRef(i),
        });
    }
    assert!(
        Signature::with_types(nodes.clone(), 0, vec![TypeRef(65)], TypeRef(65), false).is_err()
    );
    nodes.pop();
    Signature::with_types(nodes, 0, vec![TypeRef(64)], TypeRef(64), false)?;
    Ok(())
}
#[test]
fn malformed_wire_nodes_binders_and_edges_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let original = artifact(signature()?)?;
    let encoded = crates::encode(&original)?;
    // Fixed schema-one header, this identity, zero dependencies, and one function f.
    let start = 13 + 8 + "borrowed".len() + "wire".len() + 4 + 4 + 4 + 3 + 1 + 2 + 2 + 4 + 1 + 1;
    for (offset, bytes) in [
        (start, vec![1, 1]),
        (start + 2, vec![0, 0]),
        (start + 2, vec![1, 16]),
        (start + 4, vec![255]),
        (start + 5, vec![255]),
        (start + 7, vec![255]),
        (start + 8, vec![3, 0]),
        (start + 10, vec![2]),
        (start + 11, vec![1, 0]),
        (start + 20, vec![1, 1]),
        (start + 26, vec![255, 0]),
        (12, vec![255]),
    ] {
        let mut altered = encoded.clone();
        altered[offset..offset + bytes.len()].copy_from_slice(&bytes);
        assert!(crates::decode(&altered).is_err(), "offset {offset}");
    }
    let mut scalar_only = encoded[..start].to_vec();
    scalar_only.extend_from_slice(&[0, 0, 1, 0, 0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        crates::decode(&scalar_only).err(),
        Some("borrowed metadata profile requires a typed signature")
    );
    Ok(())
}
