//! Real source calls retain generic layout demand and independent field operands.
use crate::{pipeline, test_source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn input() -> Result<String> {
    Ok(test_source::fixture(
        "pa31/tests/spec/source-offset-of/provider/reached-provider.rs",
    )?)
}
#[test]
fn real_offset_calls_keep_generic_tagged_niche_and_same_instance_fields() -> Result {
    println!(
        "offset roles={} Reservations={} Catalog={} Body={} Operation={}",
        crate::support::bindings::LangItem::ALL.len(),
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<super::super::Catalog>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::sema::model::Operation>()
    );
    let source = input()?;
    let report = test_source::source_provider(&source, "offset_reached")?;
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
    )?;
    for name in [
        "option_u8",
        "option_u32",
        "option_u64",
        "option_ref",
        "field_first",
        "field_second",
        "field_third",
    ] {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|s| s.name == name && s.body.is_some())
                .count(),
            1
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/offset-reached-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn offset_declarations_require_actual_role_formal_const_and_u32_contract() -> Result {
    let source = input()?;
    for (before, after) in [
        ("#[lang = \"offset_of\"]", ""),
        ("pub const fn offset_of", "pub fn offset_of"),
        ("variant: u32", "variant: u64"),
        ("field: u32", "field: usize"),
        ("T: PointeeSized", "T: Sized"),
        (
            "(variant: u32, field: u32) -> usize",
            "(variant: u32, field: u32) -> u32",
        ),
    ] {
        assert!(source.contains(before));
        let changed = source.replace(before, after);
        let result = test_source::source_provider(&changed, "offset_invalid").and_then(|report| {
            pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        let error = result
            .err()
            .ok_or_else(|| format!("invalid offset contract accepted: {before}"))?;
        assert!(
            !error.contains("unsupported source language item"),
            "{error}"
        );
        println!("offset contract {before}: {error}");
    }
    Ok(())
}

#[test]
fn offset_headers_require_current_source_and_completed_zero_regions() -> Result {
    let source = input()?;
    let mut report = test_source::source_provider(&source, "offset_header_image")?;
    let pipeline::Analysis { mut facts, .. } = pipeline::analyze_source(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let definition = catalog
        .language(crate::support::bindings::LangItem::OffsetOf)
        .ok_or("language declaration")?
        .definition;
    catalog.require_offset_report(&report, &facts, definition)?;
    let foreign = test_source::source_provider(&source, "offset_header_image")?;
    assert!(
        catalog
            .require_offset_report(&foreign, &facts, definition)
            .is_err()
    );
    let function = facts.functions[*facts.function_indices.get(&definition).ok_or("function")?];
    let regions = facts
        .lifetimes
        .functions
        .remove(&definition)
        .ok_or("zero-region completion")?;
    assert!(super::validate(&facts, function).is_err());
    facts.lifetimes.functions.insert(definition, regions);
    super::validate(&facts, function)?;
    report.package.units[0].source = crate::support::source::SourceFile::new(source.into_bytes())?;
    assert!(
        catalog
            .require_offset_report(&report, &facts, definition)
            .is_err()
    );
    Ok(())
}
