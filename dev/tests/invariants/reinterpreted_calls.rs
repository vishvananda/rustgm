//! The original independently observed source is lowered without replacing its bodies.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn actual_source_transmute_preserves_owned_drop_and_prepares_native_roundtrips() -> Result {
    let source =
        crate::test_source::fixture("pa28/tests/spec/source-transmute/positive-owned-native.rs")?;
    let report = crate::test_source::source_provider(&source, "source_transmute")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
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
        "float_bits",
        "signed_bits",
        "const_bits",
        "tagged_numeric",
        "pointer_roundtrip",
        "function_roundtrip",
        "moved",
        "same_name",
        "identity_value",
    ] {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|s| s.name == name && s.body.is_some())
                .count(),
            1,
            "{name}"
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/native/source-transmute");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    assert_eq!(size_of::<crate::sema::compiler_items::Catalog>(), 1528);
    assert_eq!(size_of::<crate::resolve::model::Resolution>(), 2032);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    eprintln!(
        "source transmute Catalog={} Resolution={} Package={} Body={} Selection={}",
        size_of::<crate::sema::compiler_items::Catalog>(),
        size_of::<crate::resolve::model::Resolution>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<ir::Body>(),
        size_of::<crate::sema::compiler_items::TransmuteSelection>()
    );
    Ok(())
}

#[path = "formatter_calls.rs"]
mod erased_formatters;
