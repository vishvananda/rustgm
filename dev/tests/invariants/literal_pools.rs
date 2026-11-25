//! Real fresh immutable views reject scalar-pool mutations before original publication.
use super::super::{ImportedOriginals, rebuilt_operations};
use crate::{body::ir::Rvalue, sema::model::Scalar, support::crates};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;
#[test]
fn fresh_scalar_storage_rejects_changed_missing_extra_and_foreign_slots() -> Result {
    let (report, analysis, _) = crate::source_packets::client_analysis(
        crate::encoded_literals::provider()?,
        &crate::encoded_literals::fixture("consumer")?,
    )?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.node.is_none() && report.package.names.get(d.name) == "byte")
        .map(|(i, _)| u32::try_from(i).map(crate::resolve::model::DefId))
        .transpose()?
        .ok_or("actual imported byte")?;
    let mut steps = 0;
    let view = crate::sema::crates::source_original(&analysis.facts, definition, &mut steps)?
        .ok_or("actual retained source view")?;
    let body = rebuilt_operations::build(&view, &mut ImportedOriginals::default())?;
    rebuilt_operations::validate(&analysis.facts, &view, &body, &mut steps)?;
    assert_eq!(body.scalars, [Scalar::Integer(7)]);
    let edits: [fn(&mut crate::body::ir::Body); 4] = [
        |b| b.scalars[0] = Scalar::Integer(8),
        |b| b.scalars.clear(),
        |b| b.scalars.push(Scalar::Integer(7)),
        |b| {
            for value in &mut b.values {
                if let Rvalue::Scalar(id) = &mut value.kind {
                    id.0 = 1;
                }
            }
        },
    ];
    for edit in edits {
        let mut changed = body.clone();
        edit(&mut changed);
        assert!(
            rebuilt_operations::validate(&analysis.facts, &view, &changed, &mut steps).is_err()
        );
    }
    let mut limited = ImportedOriginals {
        steps: crates::MAX_COMPONENTS,
        ..ImportedOriginals::default()
    };
    assert!(rebuilt_operations::build(&view, &mut limited).is_err());
    assert!(limited.rows.is_empty() && limited.steps > crates::MAX_COMPONENTS);
    Ok(())
}

#[test]
fn fresh_wide_scalar_storage_keeps_exact_bits_and_rejects_kind_changes() -> Result {
    use crate::literal::value::FloatBits;
    let artifact = crate::encoded_literals::wide_provider()?;
    let fresh = crates::decode(&crates::encode(&artifact)?)?;
    let (report, analysis, _) = crate::source_packets::client_analysis(
        fresh,
        "#![feature(no_core,f16,f128)]#![no_core]extern crate base;\
         pub fn half()->f16{base::half_low()}pub fn quad()->f128{base::quad_low()}",
    )?;
    for (name, expected, opponent) in [
        ("half_low", FloatBits::F16(0x3c01), FloatBits::F32(0x3c01)),
        (
            "quad_low",
            FloatBits::from_quad(0x3fff_0000_0000_0000_0000_0000_0000_0001),
            FloatBits::from_quad(0x3fff_0000_0000_0000_0000_0000_0000_0000),
        ),
    ] {
        let definition = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find(|(_, d)| d.node.is_none() && report.package.names.get(d.name) == name)
            .map(|(i, _)| u32::try_from(i).map(crate::resolve::model::DefId))
            .transpose()?
            .ok_or("actual imported wide function")?;
        let mut steps = 0;
        let view = crate::sema::crates::source_original(&analysis.facts, definition, &mut steps)?
            .ok_or("actual wide source view")?;
        let mut body = rebuilt_operations::build(&view, &mut ImportedOriginals::default())?;
        rebuilt_operations::validate(&analysis.facts, &view, &body, &mut steps)?;
        assert_eq!(body.scalars, [Scalar::Float(expected)]);
        body.scalars[0] = Scalar::Float(opponent);
        assert!(rebuilt_operations::validate(&analysis.facts, &view, &body, &mut steps).is_err());
    }
    let (mut rows, attributes, modules) = crate::module_packets::raw(&artifact)?;
    let (original, value) = rows
        .originals
        .iter()
        .enumerate()
        .find_map(|(original, body)| {
            body.values.iter().enumerate().find_map(|(value, row)| {
                (rows.types.get(row.ty.0 as usize) == Some(&crates::generics::source::Type::F16)
                    && matches!(row.kind, crates::generics::source::ValueKind::Scalar(_)))
                .then_some((original, value))
            })
        })
        .ok_or("actual f16 scalar row")?;
    rows.originals[original].values[value].kind =
        crates::generics::source::ValueKind::Scalar((1_u128 << 16).to_le_bytes());
    assert_eq!(
        crates::generics::Payload::from_source_module_body_operations(
            artifact.identity(),
            rows,
            attributes,
            modules,
            artifact
                .source_descriptor()
                .ok_or("actual wide descriptor")?
                .availability()?
                .clone(),
            0,
            &mut crates::generics::Work::default(),
        )
        .err(),
        Some("non-canonical source f16 scalar value")
    );
    Ok(())
}
