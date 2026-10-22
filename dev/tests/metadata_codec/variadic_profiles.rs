//! Variadic envelopes compose with existing graphs without changing old bytes.
use rustgm::support::crates::{
    self, Abi, CrateIdentity, ExternalDef, ExternalKind, Function, Interface, Nominals,
    PublicBinding, PublicModule, PublicNames, Scalar, Signature, Static, Storage, StorageOrigin,
    Symbol, TypeGraph, TypeNode, TypeRef,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn artifact(variadic: bool, storage: bool) -> Result<Interface> {
    let owner = CrateIdentity::new("variadic", "wire")?;
    let graph = TypeGraph::new(vec![TypeNode::Scalar(Scalar::U32)])?;
    let function = Function::new(
        ExternalDef::new(owner.clone(), vec!["call".into()], ExternalKind::Function)?,
        Signature::with_shared_types(graph.clone(), 0, vec![TypeRef(0)], TypeRef(0), true)?
            .with_convention(Abi::C, variadic)?,
        Symbol::Exact("external_call".into()),
    )?;
    if !storage {
        return Ok(Interface::new(owner, vec![], vec![function])?);
    }
    let global = Static::new(
        ExternalDef::new(owner.clone(), vec!["value".into()], ExternalKind::Static)?,
        TypeRef(0),
        Symbol::Exact("external_value".into()),
        true,
        false,
        StorageOrigin::Foreign { unsafe_: true },
    )?;
    Ok(Interface::with_storage(
        owner,
        vec![],
        vec![function],
        vec![],
        Nominals::for_storage(graph, vec![])?,
        Storage::new(vec![global], vec![])?,
    )?)
}

#[test]
fn variadic_header_requires_its_envelope_and_preserves_fixed_bytes() -> Result {
    let fixed = artifact(false, false)?;
    let fixed_bytes = crates::encode(&fixed)?;
    let original = artifact(true, false)?;
    let bytes = crates::encode(&original)?;
    assert_eq!(&bytes[12..15], &[8, 4, 0]);
    assert_eq!(crates::decode(&bytes)?, original);
    let abi = super::record_start(&original) + 2 + 8;
    assert_eq!(&bytes[abi..abi + 2], &[1, 1]);
    let mut restored_fixed = bytes.clone();
    restored_fixed[abi + 1] = 0;
    assert!(crates::decode(&restored_fixed).is_err());
    restored_fixed.remove(12);
    assert_eq!(restored_fixed, fixed_bytes);
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    for (offset, value) in [
        (13, 8),
        (13, 0),
        (14, 2),
        (abi, 0),
        (abi, 2),
        (abi + 1, 2),
        (abi - 1, 2),
    ] {
        let mut bad = bytes.clone();
        bad[offset] = value;
        assert!(
            crates::decode(&bad).is_err(),
            "offset {offset}, value {value}"
        );
    }
    let mut unwrapped = bytes.clone();
    unwrapped.remove(12);
    assert!(crates::decode(&unwrapped).is_err());
    let mut trailing = bytes;
    trailing.push(0);
    assert!(crates::decode(&trailing).is_err());
    let signature = fixed.functions()[0].signature().clone();
    assert!(signature.clone().with_convention(Abi::Rust, true).is_err());
    let variable = signature.with_convention(Abi::C, true)?;
    assert!(variable.variadic());
    assert!(!variable.with_abi(Abi::Rust).variadic());
    Ok(())
}

#[test]
fn variadic_envelopes_compose_with_storage_and_public_aliases() -> Result {
    for storage in [false, true] {
        let original = artifact(true, storage)?;
        let binding = PublicBinding::new(
            "renamed".into(),
            original.functions()[0].definition().clone(),
        )?;
        let mut bindings = vec![binding];
        if let Some(storage) = original.storage() {
            bindings.push(PublicBinding::new(
                "value".into(),
                storage.statics()[0].definition().clone(),
            )?);
        }
        let named = original.with_public_names(PublicNames::new(vec![PublicModule::new(
            vec![],
            bindings,
        )?])?)?;
        let bytes = crates::encode(&named)?;
        assert_eq!(&bytes[12..15], &[8, 6, if storage { 5 } else { 4 }]);
        let decoded = crates::decode(&bytes)?;
        assert_eq!(decoded, named);
        assert_eq!(crates::encode(&decoded)?, bytes);
        if let Some(nominals) = decoded.nominals() {
            assert!(
                decoded.functions()[0]
                    .signature()
                    .types()
                    .shares_storage(nominals.types())
            );
        }
        let mut reordered = bytes.clone();
        reordered.swap(12, 13);
        assert!(crates::decode(&reordered).is_err());
        for end in 0..bytes.len() {
            assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
        }
    }
    Ok(())
}
