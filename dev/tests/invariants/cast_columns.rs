//! Primitive conversions keep their typed operand and explicit wire-version fence.
use super::*;
use crate::support::crates::{Scalar, generics::source::Profile};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn read(bytes: &[u8], profile: Profile) -> std::result::Result<s::Value, &'static str> {
    let mut reader = super::super::Reader::new(bytes);
    value_read(&mut Input {
        reader: &mut reader,
        bytes: 0,
        base: 0,
        profile,
    })
}

#[test]
fn scalar_cast_wire_retains_operand_and_rejects_older_profiles_and_truncation() -> Result {
    let value = s::Value {
        ty: g::TypeRef(11),
        kind: s::ValueKind::ScalarCast(3),
    };
    let mut writer = Writer::source(0);
    value_write(&mut writer, &value)?;
    let bytes = writer.finish();
    assert_eq!(bytes, [11, 0, 0, 0, 7, 3, 0, 0, 0]);
    assert_eq!(read(&bytes, Profile::Source2024ModuleConversions)?, value);
    for profile in [
        Profile::Source2024Functions,
        Profile::Source2024FunctionContracts,
        Profile::Source2024ModuleContracts,
        Profile::Source2024ModuleAvailability,
        Profile::Source2024ModuleFeaturePredicates,
        Profile::Source2024ModuleDefinitions,
        Profile::Source2024ModuleCompatibility,
        Profile::Source2024ModuleCallables,
        Profile::Source2024ModuleSafety,
        Profile::Source2024Provided,
        Profile::Source2024ProvidedContracts,
        Profile::Source2024ProvidedCurrent,
    ] {
        assert!(read(&bytes, profile).is_err(), "{profile:?}");
    }
    for end in 0..bytes.len() {
        assert!(read(&bytes[..end], Profile::Source2024ModuleConversions).is_err());
    }
    Ok(())
}

fn original() -> s::Original {
    s::Original {
        body: BodyRef(0),
        declaration: DeclarationRef(0),
        result: g::TypeRef(0),
        sites: 1,
        context: s::CallContext::Runtime,
        variables: Vec::new(),
        locals: Vec::new(),
        parameter_bindings: Vec::new(),
        projections: Vec::new(),
        values: vec![s::Value {
            ty: g::TypeRef(0),
            kind: s::ValueKind::Scalar([0; 16]),
        }],
        statements: Vec::new(),
        blocks: Vec::new(),
        scopes: Vec::new(),
        exits: Vec::new(),
        temporaries: Vec::new(),
        owned: None,
        arguments: Vec::new(),
        openings: Vec::new(),
        calls: Vec::new(),
    }
}

#[test]
fn scalar_cast_domains_rederive_kind_and_reject_nonprimitive_and_invalid_pairs() -> Result {
    let original = original();
    let value = s::Value {
        ty: g::TypeRef(1),
        kind: s::ValueKind::ScalarCast(0),
    };
    let scalars = [
        Scalar::Unit,
        Scalar::Bool,
        Scalar::Char,
        Scalar::U8,
        Scalar::U16,
        Scalar::U32,
        Scalar::U64,
        Scalar::U128,
        Scalar::Usize,
        Scalar::I8,
        Scalar::I16,
        Scalar::I32,
        Scalar::I64,
        Scalar::I128,
        Scalar::Isize,
        Scalar::F32,
        Scalar::F64,
    ];
    for from in scalars {
        for to in scalars {
            let types = [s::Type::Scalar(from), s::Type::Scalar(to)];
            let wire = value.scalar_cast_types(&original, &types);
            let ordinary = crate::sema::crates::scalar_cast(from, to);
            assert_eq!(wire.is_ok(), ordinary.is_some(), "{from:?} to {to:?}");
            if let Ok(pair) = wire {
                assert_eq!(pair, (from, to));
            }
        }
    }
    let types = [s::Type::Scalar(Scalar::F32), s::Type::Scalar(Scalar::U8)];
    assert_eq!(
        value.scalar_cast_types(&original, &types)?,
        (Scalar::F32, Scalar::U8)
    );
    assert!(value.scalar_cast_types(&original, &types[..1]).is_err());
    let missing = s::Value {
        kind: s::ValueKind::ScalarCast(1),
        ..value
    };
    assert!(missing.scalar_cast_types(&original, &types).is_err());
    let noncast = s::Value {
        kind: s::ValueKind::Constrain(0),
        ..value
    };
    assert!(noncast.scalar_cast_types(&original, &types).is_err());
    let pointer = s::Type::Reference {
        mutable: false,
        region: s::RegionRef(0),
        referent: g::TypeRef(0),
    };
    assert!(
        value
            .scalar_cast_types(&original, &[types[0], pointer])
            .is_err()
    );
    Ok(())
}
