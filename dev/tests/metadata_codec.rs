//! Untrusted metadata is bounded, canonical and independent of local arena order.
#[path = "metadata_codec/borrowed.rs"]
mod borrowed;
#[path = "metadata_codec/calling_conventions.rs"]
mod calling_conventions;
#[path = "metadata_codec/extents.rs"]
mod extents;
#[path = "metadata_codec/nominals.rs"]
mod nominals;
#[path = "metadata_codec/public_names.rs"]
mod public_names;
#[path = "metadata_codec/scalar_values.rs"]
mod scalar_values;
#[path = "metadata_codec/stored_interfaces.rs"]
mod stored_interfaces;
use rustgm::support::crates::{
    self, CrateIdentity, ExternalDef, ExternalKind, Function, Interface, Scalar, Signature, Symbol,
};

fn function(
    owner: &CrateIdentity,
    path: &[&str],
    symbol: Symbol,
) -> Result<Function, &'static str> {
    Function::new(
        ExternalDef::new(
            owner.clone(),
            path.iter().map(|value| (*value).to_owned()).collect(),
            ExternalKind::Function,
        )?,
        Signature::new(vec![Scalar::U32, Scalar::Bool], Scalar::U64, false)?,
        symbol,
    )
}
fn interface() -> Result<Interface, &'static str> {
    let owner = CrateIdentity::new("library", "build_123")?;
    Interface::new(
        owner.clone(),
        vec![CrateIdentity::new("dependency", "a1")?],
        vec![
            function(&owner, &["math", "first"], Symbol::Rust)?,
            function(&owner, &["second"], Symbol::Exact("e\u{301}.$".into()))?,
        ],
    )
}
fn record_start(interface: &Interface) -> usize {
    let identity_bytes =
        |identity: &CrateIdentity| 8 + identity.name().len() + identity.disambiguator().len();
    13 + identity_bytes(interface.identity())
        + 4
        + interface
            .dependencies()
            .iter()
            .map(identity_bytes)
            .sum::<usize>()
        + 4
}

#[test]
fn identities_and_symbols_preserve_crate_path_and_namespace()
-> Result<(), Box<dyn std::error::Error>> {
    let first = CrateIdentity::new("same", "one")?;
    let second = CrateIdentity::new("same", "two")?;
    let left = function(&first, &["nested", "value"], Symbol::Rust)?;
    let right = function(&second, &["nested", "value"], Symbol::Rust)?;
    let flat = function(&first, &["nested_value"], Symbol::Rust)?;
    assert_ne!(
        left.definition().rust_symbol(),
        right.definition().rust_symbol()
    );
    assert_ne!(
        left.definition().rust_symbol(),
        flat.definition().rust_symbol()
    );
    assert!(
        left.definition()
            .rust_symbol()
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
    );
    assert!(CrateIdentity::new("bad-name", "one").is_err());
    assert!(CrateIdentity::new("name", "").is_err());
    assert!(CrateIdentity::new("name", "../id").is_err());
    assert!(
        ExternalDef::new(
            first.clone(),
            vec!["e\u{301}".into()],
            ExternalKind::Function
        )
        .is_err()
    );
    assert!(ExternalDef::new(first.clone(), vec!["é".into()], ExternalKind::Function).is_ok());
    assert!(ExternalDef::new(first.clone(), vec!["_".into()], ExternalKind::Function).is_err());
    let module = ExternalDef::new(first, vec!["nested".into()], ExternalKind::Module)?;
    assert_ne!(module.namespace(), left.definition().namespace());
    Ok(())
}

#[test]
fn canonical_roundtrip_is_independent_of_input_table_order()
-> Result<(), Box<dyn std::error::Error>> {
    let interface = interface()?;
    let mut functions = interface.functions().to_vec();
    functions.reverse();
    let mut dependencies = vec![CrateIdentity::new("z", "1")?, CrateIdentity::new("a", "2")?];
    let left = Interface::new(
        interface.identity().clone(),
        dependencies.clone(),
        functions.clone(),
    )?;
    dependencies.reverse();
    functions.reverse();
    let right = Interface::new(interface.identity().clone(), dependencies, functions)?;
    let bytes = crates::encode(&left)?;
    assert_eq!(bytes, crates::encode(&right)?);
    assert_eq!(crates::decode(&bytes)?, left);
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    assert_eq!(&bytes[..13], b"RGMETA\r\n\x01\0\0\0\0");
    let exact = crates::decode(&crates::encode(&interface)?)?;
    assert_eq!(
        exact.functions()[1].symbol(),
        &Symbol::Exact("e\u{301}.$".into())
    );
    Ok(())
}

#[test]
fn truncation_profiles_counts_and_body_permissions_fail_before_import()
-> Result<(), Box<dyn std::error::Error>> {
    let interface = interface()?;
    let bytes = crates::encode(&interface)?;
    for length in 0..bytes.len() {
        assert!(crates::decode(&bytes[..length]).is_err(), "prefix {length}");
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(crates::decode(&trailing).is_err());
    let mut unknown_scalar = bytes.clone();
    if let Some(tag) = unknown_scalar.last_mut() {
        *tag = 255;
    }
    assert_eq!(
        crates::decode(&unknown_scalar),
        Err("unknown metadata scalar type")
    );
    for offset in [0, 8, 10, 11, 12, 17] {
        let mut malformed = bytes.clone();
        malformed[offset] = 255;
        assert!(crates::decode(&malformed).is_err(), "header {offset}");
    }
    let first = record_start(&interface);
    for (offset, value) in [
        (0, 2),
        (0, 1),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 2),
        (7, 128),
        (8, 1),
        (9, 1),
        (10, 0),
    ] {
        let mut malformed = bytes.clone();
        malformed[first + offset] = value;
        assert!(
            crates::decode(&malformed).is_err(),
            "record {offset}={value}"
        );
    }
    let mut excessive = bytes.clone();
    excessive[first - 4..first].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(crates::decode(&excessive).is_err());
    Ok(())
}

#[test]
fn duplicate_foreign_or_unbounded_owned_tables_are_not_published()
-> Result<(), Box<dyn std::error::Error>> {
    let owner = CrateIdentity::new("library", "one")?;
    let other = CrateIdentity::new("library", "two")?;
    let value = function(&owner, &["f"], Symbol::Rust)?;
    assert!(Interface::new(owner.clone(), vec![], vec![value.clone(), value.clone()]).is_err());
    assert!(Interface::new(other.clone(), vec![], vec![value.clone()]).is_err());
    assert!(Interface::new(owner.clone(), vec![owner.clone()], vec![]).is_err());
    assert!(Interface::new(owner.clone(), vec![other.clone(), other], vec![]).is_err());
    assert!(Signature::new(vec![Scalar::Unit], Scalar::Unit, false).is_err());
    assert!(Signature::new(vec![Scalar::U8; 257], Scalar::Unit, false).is_err());
    for name in ["", "nul\0name"] {
        assert!(function(&owner, &["f"], Symbol::Exact(name.into())).is_err());
    }
    let one = Interface::new(owner, vec![], vec![value])?;
    let mut duplicate = crates::encode(&one)?;
    let start = record_start(&one);
    let record = duplicate[start..].to_vec();
    let mut unnamed = duplicate.clone();
    unnamed[start + 16] = b'_';
    assert_eq!(
        crates::decode(&unnamed),
        Err("invalid canonical metadata path component")
    );
    duplicate[start - 4..start].copy_from_slice(&2u32.to_le_bytes());
    duplicate.extend_from_slice(&record);
    assert!(crates::decode(&duplicate).is_err());
    Ok(())
}
#[path = "metadata_codec/array_storage.rs"]
mod array_storage;
