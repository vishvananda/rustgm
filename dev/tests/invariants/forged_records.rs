//! Actual exported rows reject changed nominal owners, fields and original uses.
use crate::{
    lower::instances::SourcePublication,
    module_packets,
    support::crates::{
        self,
        generics::{self as g, source as s},
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
type Mutation = fn(&mut s::SourceRows);
fn validate(interface: &crates::Interface, rows: s::SourceRows) -> Result {
    let descriptor = interface.source_descriptor().ok_or("source descriptor")?;
    let (_, attributes, modules) = module_packets::raw(interface)?;
    g::Payload::from_source_module_nominals(
        interface.identity(),
        rows,
        attributes,
        modules,
        descriptor.availability()?.clone(),
        0,
        &mut g::Work::default(),
    )?;
    Ok(())
}
#[test]
fn nominal_rows_reject_foreign_stale_and_retyped_original_contracts() -> Result {
    let source = crate::test_source::fixture("pa29/tests/spec/source-nominals/provider.rs")?;
    let interface = module_packets::exported_profile(
        &source,
        SourcePublication::ModuleNominals,
        "nominal_forgery",
    )?;
    let (rows, _, _) = module_packets::raw(&interface)?;
    validate(&interface, rows.clone())?;
    let mutations: [(&str, Mutation); 8] = [
        ("cross-owner field", |rows| {
            rows.fields[0].owner = rows.nominals[1].declaration;
        }),
        ("stale ordinal", |rows| rows.fields[0].ordinal = 1),
        ("invalid source site", |rows| {
            rows.fields[0].site = rows.declarations[rows.nominals[0].declaration.0 as usize].sites;
        }),
        ("wrong template type", |rows| {
            rows.fields[0].ty = rows.nominals[0].self_type;
        }),
        ("foreign access owner", |rows| {
            rows.fields[0].visibility = s::ModuleVisibility::Within(rows.functions[0].declaration);
        }),
        ("omitted formal argument", |rows| {
            let s::Type::Nominal { arguments, .. } =
                &mut rows.types[rows.nominals[0].self_type.0 as usize]
            else {
                return;
            };
            arguments.end = arguments.start;
        }),
        ("incomplete constructor", |rows| {
            for body in &mut rows.originals {
                for value in &mut body.values {
                    if let s::ValueKind::Aggregate { fields } = &mut value.kind {
                        fields.end = fields.start;
                        return;
                    }
                }
            }
        }),
        ("cross-owner projection", |rows| {
            for body in &mut rows.originals {
                for projection in &mut body.projections {
                    if let s::ProjectionKind::Field { owner, .. } = &mut projection.kind {
                        *owner = rows.nominals[1].declaration;
                        return;
                    }
                }
            }
        }),
    ];
    for (label, mutate) in mutations {
        let mut changed = rows.clone();
        mutate(&mut changed);
        assert!(validate(&interface, changed).is_err(), "{label}");
    }
    let descriptor = interface.source_descriptor().ok_or("source descriptor")?;
    let (rows, attributes, modules) = module_packets::raw(&interface)?;
    assert!(
        g::Payload::from_source_module_conversions(
            interface.identity(),
            rows,
            attributes,
            modules,
            descriptor.availability()?.clone(),
            0,
            &mut g::Work::default(),
        )
        .is_err()
    );
    Ok(())
}
