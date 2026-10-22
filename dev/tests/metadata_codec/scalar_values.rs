//! Profile one transports validated values, never allocation or execution permissions.
use super::{
    CrateIdentity, ExternalDef, ExternalKind, Function, Interface, Scalar, Signature, Symbol,
    crates,
};
use crates::{Constant, ScalarValue};

fn constant(
    owner: &CrateIdentity,
    name: &str,
    ty: Scalar,
    bits: u128,
) -> Result<Constant, &'static str> {
    Constant::new(
        ExternalDef::new(owner.clone(), vec![name.into()], ExternalKind::Constant)?,
        ScalarValue::new(ty, bits)?,
    )
}
#[test]
fn scalar_values_preserve_widths_sign_bits_zero_and_nan_payloads()
-> Result<(), Box<dyn std::error::Error>> {
    let owner = CrateIdentity::new("values", "v1")?;
    let cases = [
        (Scalar::Unit, 0),
        (Scalar::Bool, 1),
        (Scalar::Char, 0x10_ffff),
        (Scalar::U8, 255),
        (Scalar::I8, 255),
        (Scalar::U16, 65535),
        (Scalar::I16, 65535),
        (Scalar::U32, u32::MAX.into()),
        (Scalar::I32, u32::MAX.into()),
        (Scalar::U64, u64::MAX.into()),
        (Scalar::I64, u64::MAX.into()),
        (Scalar::Usize, u64::MAX.into()),
        (Scalar::Isize, u64::MAX.into()),
        (Scalar::U128, u128::MAX),
        (Scalar::I128, 1 << 127),
        (Scalar::F32, 0),
        (Scalar::F32, 0x8000_0000),
        (Scalar::F32, 0x7fc0_1234),
        (Scalar::F64, 0),
        (Scalar::F64, 0x8000_0000_0000_0000),
        (Scalar::F64, 0xfff8_0000_0000_1234),
    ];
    let values = cases
        .into_iter()
        .enumerate()
        .map(|(index, (ty, bits))| constant(&owner, &format!("V{index:02}"), ty, bits))
        .collect::<Result<Vec<_>, _>>()?;
    let first = Interface::with_constants(owner.clone(), vec![], vec![], values.clone())?;
    super::public_names::roundtrip(&first)?;
    let second =
        Interface::with_constants(owner, vec![], vec![], values.into_iter().rev().collect())?;
    let bytes = crates::encode(&first)?;
    assert_eq!(&bytes[..13], b"RGMETA\r\n\x01\0\0\0\x01");
    assert_eq!(bytes, crates::encode(&second)?);
    assert_eq!(crates::decode(&bytes)?, first);
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for (value, (ty, bits)) in first.constants().iter().zip(cases) {
        assert_eq!(value.value(), ScalarValue::new(ty, bits)?);
    }
    Ok(())
}
#[test]
fn invalid_typed_payloads_and_cross_kind_names_are_rejected()
-> Result<(), Box<dyn std::error::Error>> {
    for (ty, bits) in [
        (Scalar::Unit, 1),
        (Scalar::Bool, 2),
        (Scalar::Char, 0xd800),
        (Scalar::Char, 0x11_0000),
        (Scalar::Char, 1 << 32),
        (Scalar::I8, 256),
        (Scalar::U16, 1 << 16),
        (Scalar::I32, 1 << 32),
        (Scalar::F32, 1 << 32),
        (Scalar::F64, 1 << 64),
    ] {
        assert!(ScalarValue::new(ty, bits).is_err(), "{ty:?}: {bits}");
    }
    let owner = CrateIdentity::new("values", "v1")?;
    let value = constant(&owner, "F", Scalar::U8, 7)?;
    let function = Function::new(
        ExternalDef::new(owner.clone(), vec!["F".into()], ExternalKind::Function)?,
        Signature::new(vec![], Scalar::U8, false)?,
        Symbol::Rust,
    )?;
    assert!(
        Interface::with_constants(
            owner.clone(),
            vec![],
            vec![function.clone()],
            vec![value.clone()]
        )
        .is_err()
    );
    assert!(
        Interface::with_constants(
            owner.clone(),
            vec![],
            vec![],
            vec![value.clone(), value.clone()]
        )
        .is_err()
    );
    assert!(
        Interface::with_constants(
            CrateIdentity::new("foreign", "v1")?,
            vec![],
            vec![],
            vec![value.clone()]
        )
        .is_err()
    );
    assert_eq!(
        Interface::with_constants(owner, vec![], vec![function], vec![value; 100_000]).err(),
        Some("metadata table exceeds profile budget")
    );
    Ok(())
}
#[test]
fn malformed_value_wire_is_rejected_before_remapping() -> Result<(), Box<dyn std::error::Error>> {
    let owner = CrateIdentity::new("values", "v1")?;
    let interface = Interface::with_constants(
        owner.clone(),
        vec![],
        vec![],
        vec![constant(&owner, "A", Scalar::U8, 7)?],
    )?;
    let bytes = crates::encode(&interface)?;
    let record = bytes.len() - 30;
    for length in 0..bytes.len() {
        assert!(crates::decode(&bytes[..length]).is_err(), "prefix {length}");
    }
    for (offset, value) in [
        (12, 2),
        (record, 1),
        (record + 4, 0),
        (record + 5, 1),
        (record + 6, 255),
    ] {
        let mut malformed = bytes.clone();
        malformed[offset] = value;
        assert!(crates::decode(&malformed).is_err(), "offset {offset}");
    }
    for (ty, bits) in [
        (Scalar::Bool, 2u128),
        (Scalar::Char, 0xd800),
        (Scalar::I8, 256),
        (Scalar::F32, 1 << 32),
    ] {
        let mut malformed = bytes.clone();
        malformed[record + 6] = ty as u8;
        let last = malformed.len() - 16;
        malformed[last..].copy_from_slice(&bits.to_le_bytes());
        assert!(crates::decode(&malformed).is_err(), "{ty:?}");
    }
    let mut empty = bytes[..record].to_vec();
    empty[record - 4..].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        crates::decode(&empty).err(),
        Some("empty scalar value profile is not canonical")
    );
    let mut duplicate = bytes.clone();
    duplicate[record - 4..record].copy_from_slice(&2u32.to_le_bytes());
    duplicate.extend_from_slice(&bytes[record..]);
    assert!(crates::decode(&duplicate).is_err());
    let mut huge = bytes;
    huge[record - 4..record].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(crates::decode(&huge).is_err());
    Ok(())
}
#[test]
fn function_only_profile_keeps_its_original_exact_encoding()
-> Result<(), Box<dyn std::error::Error>> {
    let owner = CrateIdentity::new("x", "v")?;
    let function = Function::new(
        ExternalDef::new(owner.clone(), vec!["f".into()], ExternalKind::Function)?,
        Signature::new(vec![], Scalar::U8, false)?,
        Symbol::Rust,
    )?;
    let interface = Interface::new(owner, vec![], vec![function])?;
    let expected=b"RGMETA\r\n\x01\0\0\0\0\x01\0\0\0x\x01\0\0\0v\0\0\0\0\x01\0\0\0\0\0\0\0\x01\x01\x01\0\0\0\x01\0\x01\0\0\0f\0\0\0\x03";
    assert_eq!(crates::encode(&interface)?, expected);
    Ok(())
}

#[test]
fn named_scalar_profile_keeps_its_original_exact_encoding() -> Result<(), Box<dyn std::error::Error>>
{
    let owner = CrateIdentity::new("x", "v")?;
    let value = constant(&owner, "C", Scalar::U8, 7)?;
    let interface = Interface::with_constants(owner, vec![], vec![], vec![value])?;
    let expected=b"RGMETA\r\n\x01\0\0\0\x01\x01\0\0\0x\x01\0\0\0v\0\0\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x01\x02\x03\x01\0\x01\0\0\0C\x07\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    assert_eq!(crates::encode(&interface)?, expected);
    assert_eq!(crates::decode(expected)?, interface);
    Ok(())
}
