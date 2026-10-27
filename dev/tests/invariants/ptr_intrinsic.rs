//! Genuine source declarations reach real const and runtime pointer consumers.
use super::*;
use crate::{
    pipeline,
    resolve::model::DefKind,
    sema::values::{Reader, View},
};
use std::mem::size_of;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/pointer-metadata-intrinsic/{name}.rs"
    ))?)
}
fn report(name: &str) -> TestResult<Report> {
    Ok(crate::test_source::source_provider(
        &source(name)?,
        "pointer_metadata_actual",
    )?)
}
#[test]
fn source_unit_and_lengths_evaluate_without_reading_pointee_memory() -> TestResult {
    for (name, expected) in [
        ("thin-unit", None),
        ("foreign-thin", None),
        ("string-length", Some(6)),
        ("slice-length", Some(3)),
        ("tail-without-dereference", Some(3)),
    ] {
        let report = report(name)?;
        let analysis = pipeline::analyze_source(&report).map_err(|e| format!("{name}: {e}"))?;
        let binding = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find_map(|(i, d)| {
                (d.kind == DefKind::Const
                    && report.package.names.get(d.name)
                        == if expected.is_some() { "LENGTH" } else { "UNIT" })
                .then_some(DefId(u32::try_from(i).ok()?))
            })
            .ok_or("actual constant")?;
        let reader = Reader::new(&analysis.facts);
        let value = reader
            .definition(binding)
            .ok_or("mandatory source constant was not evaluated")?;
        if let Some(bits) = expected {
            assert!(
                matches!(reader.view(value)?,View::Scalar(crate::sema::model::Scalar::Integer(n)) if n == bits)
            );
        } else {
            assert!(matches!(
                reader.view(value)?,
                View::Product {
                    length: 0,
                    variant: None
                }
            ));
        }
    }
    Ok(())
}
#[test]
fn exact_source_calls_lower_thin_wide_and_unit_effects() -> TestResult {
    for name in [
        "thin-unit",
        "foreign-thin",
        "string-length",
        "slice-length",
        "tail-without-dereference",
        "unit-argument-once",
    ] {
        let report = report(name)?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )
        .map_err(|e| format!("{name}: {e}"))?;
        assert!(
            !program.functions.is_empty(),
            "{name}: no actual callable was lowered"
        );
    }
    Ok(())
}
#[test]
fn erased_unit_preserves_real_const_argument_faults() -> TestResult {
    let report = report("unit-argument-fault")?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("unit metadata erased a real argument read")?;
    assert!(
        error.to_string().contains("null") || error.to_string().contains("address"),
        "{error}"
    );
    Ok(())
}
#[test]
fn actual_header_role_gate_and_signature_opponents_stay_closed() -> TestResult {
    for (name, message) in [
        ("mutable-intrinsic-input", "*const P"),
        ("unrequested-intrinsics", "intrinsics"),
    ] {
        let result = report(name).and_then(|report| {
            pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(Into::into)
        });
        let error = result
            .err()
            .ok_or_else(|| format!("{name}: invalid source contract accepted"))?;
        assert!(error.to_string().contains(message), "{name}: {error}");
    }
    let text = source("thin-unit")?.replace("#[lang=\"pointee_trait\"]", "");
    let missing_report = crate::test_source::source_provider(&text, "missing_pointer_role")?;
    let error = pipeline::analyze_source(&missing_report)
        .err()
        .ok_or("same-spelling unclaimed trait gained Pointee authority")?;
    assert!(
        error.to_string().contains("Pointee") || error.to_string().contains("Metadata"),
        "{error}"
    );
    let dynamic = report("dynamic-result")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&dynamic)?;
    let definition = dynamic
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(index, item)| {
            (item.kind == DefKind::Function
                && dynamic.package.names.get(item.name) == "actual_dynamic")
                .then_some(DefId(u32::try_from(index).ok()?))
        })
        .ok_or("actual dynamic caller")?;
    let error = crate::lower::instances::build_roots_prepared(
        &dynamic,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[definition],
        },
        prepared,
    )
    .err()
    .ok_or("dynamic Metadata gained a fabricated standalone value")?;
    assert!(
        error.message.contains("metadata storage producer"),
        "{error}"
    );
    Ok(())
}
#[test]
fn pointer_contract_storage_keeps_hot_body_and_declaration_widths() -> TestResult {
    assert_eq!(size_of::<crate::types::Kind>(), 24);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::resolve::model::Definition>(), 36);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<crate::sema::model::Operation>(), 16);
    assert_eq!(size_of::<crate::body::ir::Body>(), 448);
    eprintln!(
        "ptr_metadata layouts intrinsic={} selected={} kind={} node={} operation={} body={}",
        size_of::<SourceIntrinsic>(),
        size_of::<PtrMetadataSelection>(),
        size_of::<Kind>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::sema::model::Operation>(),
        size_of::<crate::body::ir::Body>()
    );
    Ok(())
}

#[test]
fn ordinary_same_spelling_retains_its_real_function_body() -> TestResult {
    let report = report("ordinary-name")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(i, d)| {
            (d.kind == DefKind::Function && report.package.names.get(d.name) == "ptr_metadata")
                .then_some(DefId(u32::try_from(i).ok()?))
        })
        .ok_or("actual ordinary function")?;
    assert!(
        facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .intrinsic_kind(definition)?
            .is_none()
    );
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    assert!(!program.functions.is_empty());
    Ok(())
}
