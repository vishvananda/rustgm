//! Overflowing products preserve target bits and report faults without panicking.
use crate::resolve as source;
use rustgm::{lower, pipeline};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa28/tests/spec/multiply-overflow";
fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let text = source::fixture(&format!("{PACKET}/{name}.rs"))?;
    let report = source::source_provider(&text, "integer_products")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    Ok(lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn actual_multiplication_keeps_ctfe_all_integer_widths_copy_and_ordinary_name() -> Result {
    for name in ["direct", "signed", "widths", "ordinary", "native"] {
        program(name).map_err(|error| format!("{name}: {error}"))?;
    }
    let error = program("noncopy")
        .err()
        .ok_or("non-Copy multiplication accepted")?;
    assert!(error.to_string().contains("trait obligation"), "{error}");
    Ok(())
}
#[test]
#[ignore = "requires the locked object backend and independent C observer"]
fn actual_product_bits_and_flags_match_all156_independent_pairs() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/multiply-overflow/native");
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "products",
        &program("native")?,
        &source::fixture(&format!("{PACKET}/observer.c"))?,
        &source::fixture(&format!("{PACKET}/expected.stdout"))?,
    )?;
    Ok(())
}
