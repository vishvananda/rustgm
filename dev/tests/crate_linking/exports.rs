//! Scalar metadata and generated private names share one checked library pipeline.
#[path = "../helpers/resolve.rs"]
mod source;
use rustgm::{
    lower::{self, Roots},
    support::crates::{self, CrateIdentity, Symbol},
};
use std::fmt::Write;
fn compile(
    text: &str,
    disambiguator: &str,
) -> Result<(rustgm::lowir::program::Program, crates::Interface), Box<dyn std::error::Error>> {
    let report = source::resolve(text, true)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let exports = rustgm::sema::crates::export(
        &report,
        &mut facts,
        CrateIdentity::new("library", disambiguator)?,
    )?;
    let program = lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        Roots {
            entry: None,
            exports: &exports.roots,
        },
        prepared,
    )?;
    Ok((program, exports.interface))
}
#[test]
fn library_symbols_scope_private_instances_closures_and_data()
-> Result<(), Box<dyn std::error::Error>> {
    let text = "fn id<T:Copy>(n:T)->T{n}static VALUE:u8=7;pub fn small(n:u8)->u8{id(n)+VALUE}pub mod nested{pub fn wide(n:u32)->u32{let f:fn(u32)->u32=|x|x+1;f(super::id(n))}}";
    let (first, interface) = compile(text, "first")?;
    let (second, other) = compile(text, "second")?;
    let prefix = interface.identity().symbol_prefix();
    let private: Vec<_> = first
        .symbols
        .iter()
        .filter(|symbol| symbol.body.is_some())
        .collect();
    assert!(
        private
            .iter()
            .all(|symbol| symbol.name.starts_with(&prefix))
    );
    assert!(
        private
            .iter()
            .any(|symbol| symbol.name.contains("rg.closure."))
    );
    assert_eq!(
        private
            .iter()
            .filter(|symbol| symbol.name.contains(".i"))
            .count(),
        2
    );
    assert!(!first.globals.is_empty());
    assert!(
        first
            .globals
            .iter()
            .all(|global| global.name.starts_with(&prefix))
    );
    for function in interface.functions() {
        assert!(
            first
                .symbols
                .iter()
                .any(|symbol| symbol.name == function.definition().rust_symbol()
                    && symbol.body.is_some())
        );
    }
    assert!(
        second
            .symbols
            .iter()
            .filter(|symbol| symbol.body.is_some())
            .all(|symbol| !symbol.name.starts_with(&prefix))
    );
    assert_ne!(crates::encode(&interface)?, crates::encode(&other)?);
    Ok(())
}
#[test]
fn export_order_and_exact_symbols_preserve_the_wire_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let (_, first) = compile("pub fn a(n:u8)->u8{n}pub fn b(n:u32)->u32{n}", "same")?;
    let (_, second) = compile("pub fn b(n:u32)->u32{n}pub fn a(n:u8)->u8{n}", "same")?;
    assert_eq!(crates::encode(&first)?, crates::encode(&second)?);
    let (program, exact) = compile(
        "#[unsafe(export_name=\"fixed.name\")]pub fn f(n:u32)->u32{n}",
        "exact",
    )?;
    assert_eq!(
        exact.functions()[0].symbol(),
        &Symbol::Exact("fixed.name".into())
    );
    assert!(
        program
            .symbols
            .iter()
            .any(|symbol| symbol.name == "fixed.name" && symbol.body.is_some())
    );
    Ok(())
}
#[test]
fn unsupported_public_contracts_are_diagnosed_and_unused_bodies_still_checked()
-> Result<(), Box<dyn std::error::Error>> {
    for text in [
        "pub fn id<T:Copy>(n:T)->T{n}",
        "pub const fn f(n:u32)->u32{n}",
        "pub extern \"C\" fn f(n:u32)->u32{n}",
        "#[target_feature(enable=\"sse2\")]pub unsafe fn f(n:u32)->u32{n}",
        "pub fn f(n:&u32)->u32{*n}",
        "pub struct S;",
        "pub mod empty{}",
        "mod hidden{pub fn f(n:u32)->u32{n}}pub use hidden::f;",
    ] {
        let report = source::resolve(text, true)?;
        let mut analysis = rustgm::pipeline::analyze_source(&report)?;
        let result = rustgm::sema::crates::export(
            &report,
            &mut analysis.facts,
            CrateIdentity::new("library", "boundary")?,
        );
        assert!(result.is_err(), "dropped public contract: {text}");
    }
    assert!(
        compile(
            "fn bad(){let x:u8;let _=x+1;}pub fn f(n:u32)->u32{n}",
            "invalid"
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn every_scalar_tag_exports_through_checked_signatures_and_roundtrips()
-> Result<(), Box<dyn std::error::Error>> {
    let names = [
        "bool", "char", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
        "i128", "isize", "f32", "f64",
    ];
    let mut text = String::from("pub fn unit(){}");
    for (index, name) in names.iter().enumerate() {
        write!(text, "pub fn scalar{index}(n:{name})->{name}{{n}}")?;
    }
    let (_, interface) = compile(&text, "scalars")?;
    let decoded = crates::decode(&crates::encode(&interface)?)?;
    assert_eq!(decoded, interface);
    assert_eq!(interface.functions().len(), 17);
    Ok(())
}
