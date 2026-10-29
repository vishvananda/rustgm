//! Primitive literals pass through real provider bytes and fresh source body proof.
use crate::{
    lower::instances::SourcePublication,
    module_packets as test, source_packets,
    support::crates::{
        self, Scalar, ScalarValue,
        generics::{self as g, source as s},
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const _: [(); 20] = [(); size_of::<s::ValueKind>()];
const _: [(); 24] = [(); size_of::<s::Value>()];
const _: [(); 24] = [(); size_of::<crate::body::ir::Value>()];
pub(crate) fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/scalar-body-values/{name}.rs"
    ))?)
}
pub(crate) fn provider() -> Result<crates::Interface> {
    test::exported_profile(
        &fixture("provider")?,
        SourcePublication::ModuleFeaturePredicates,
        "scalar_body_values",
    )
}
#[test]
fn primitive_literals_reconstruct_fresh_originals_and_emit_all_native_roots() -> Result {
    let artifact = provider()?;
    let bytes = crates::encode(&artifact)?;
    assert_eq!(bytes.get(13..15), Some([4, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    let (
        report,
        crate::pipeline::Analysis {
            mut facts,
            prepared,
        },
        _,
    ) = source_packets::client_analysis(crates::decode(&bytes)?, &fixture("consumer")?)?;
    let exports = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .filter(|(_, d)| {
            d.kind == crate::resolve::model::DefKind::Function
                && d.node.is_some()
                && [
                    "byte", "wide", "signed", "flag", "letter", "single", "double",
                ]
                .contains(&report.package.names.get(d.name))
        })
        .map(|(index, _)| u32::try_from(index).map(crate::resolve::model::DefId))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(exports.len(), 7);
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &exports,
        },
        prepared,
    )?;
    let mut lowir = Vec::new();
    crate::lowir::writer::write(&program, &mut lowir)?;
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/scalar-body-values-v1/candidate-v1");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("consumer.lowir"), lowir)?;
    std::fs::write(out.join("provider.rgcrate"), bytes)?;
    for profile in [
        SourcePublication::ModuleAvailability,
        SourcePublication::ModuleContracts,
        SourcePublication::LegacyFunctions,
    ] {
        assert!(test::exported_profile(&fixture("provider")?, profile, "older_scalars").is_err());
    }
    Ok(())
}
#[test]
fn primitive_bit_carrier_checks_type_width_unicode_and_float_payloads() -> Result {
    for (ty, bits) in [
        (Scalar::U128, u128::MAX),
        (Scalar::I8, 255),
        (Scalar::Bool, 1),
        (Scalar::Char, 0x0010_ffff),
        (Scalar::F32, 0x8000_0000),
        (Scalar::F32, 0x7fc0_1234),
        (Scalar::F64, 0x8000_0000_0000_0000),
        (Scalar::F64, 0x7ff8_1234_5678_9abc),
    ] {
        let expected = ScalarValue::new(ty, bits)?;
        let row = s::Value {
            ty: g::TypeRef(0),
            kind: s::ValueKind::Scalar(bits.to_le_bytes()),
        };
        assert_eq!(row.scalar_value(&[s::Type::Scalar(ty)])?, expected);
        assert_eq!(
            crate::sema::crates::export_scalar_value(
                ty,
                crate::sema::crates::import_scalar_value(expected)?
            )?,
            expected
        );
    }
    for (ty, bits) in [
        (Scalar::Bool, 2_u128),
        (Scalar::Char, 0xd800),
        (Scalar::Char, 0x0011_0000),
        (Scalar::U8, 256),
        (Scalar::I8, 256),
        (Scalar::F32, 1_u128 << 32),
        (Scalar::F64, 1_u128 << 64),
        (Scalar::Unit, 0),
    ] {
        let row = s::Value {
            ty: g::TypeRef(0),
            kind: s::ValueKind::Scalar(bits.to_le_bytes()),
        };
        assert!(row.scalar_value(&[s::Type::Scalar(ty)]).is_err());
    }
    let row = s::Value {
        ty: g::TypeRef(0),
        kind: s::ValueKind::Scalar([0; 16]),
    };
    assert!(row.scalar_value(&[]).is_err());
    assert!(
        row.scalar_value(&[s::Type::Parameter(s::SlotRef(0))])
            .is_err()
    );
    assert!(
        crate::sema::crates::export_scalar_value(
            Scalar::Bool,
            crate::sema::model::Scalar::Integer(1)
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn malformed_scalar_columns_and_failed_payment_publish_no_payload() -> Result {
    let artifact = provider()?;
    let (original, attributes, modules) = test::raw(&artifact)?;
    let availability = artifact
        .source_descriptor()
        .ok_or("actual descriptor")?
        .availability()?
        .clone();
    let scalar = original
        .originals
        .iter()
        .enumerate()
        .find_map(|(body, row)| {
            row.values.iter().enumerate().find_map(|(value, row)| {
                (matches!(row.kind, s::ValueKind::Scalar(_))
                    && original.types.get(row.ty.0 as usize)
                        == Some(&s::Type::Scalar(Scalar::Bool)))
                .then_some((body, value))
            })
        })
        .ok_or("actual boolean body value")?;
    for ty in [None, Some(g::TypeRef(u32::MAX))] {
        let mut rows = original.clone();
        let row = &mut rows.originals[scalar.0].values[scalar.1];
        if let Some(ty) = ty {
            row.ty = ty;
        } else {
            row.kind = s::ValueKind::Scalar(2_u128.to_le_bytes());
        }
        let mut work = g::Work::default();
        assert!(
            g::Payload::from_source_module_feature_predicates(
                artifact.identity(),
                rows,
                attributes.clone(),
                modules.clone(),
                availability.clone(),
                0,
                &mut work
            )
            .is_err()
        );
        assert!(work.steps > 0);
    }
    let mut work = g::Work::default();
    assert!(
        g::Payload::from_source_module_feature_predicates(
            artifact.identity(),
            original,
            attributes,
            modules,
            availability,
            crates::MAX_BYTES,
            &mut work
        )
        .is_err()
    );
    assert!(work.steps > 0);
    Ok(())
}
