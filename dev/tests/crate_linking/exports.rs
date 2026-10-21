//! Scalar metadata and generated private names share one checked library pipeline.
#[path = "../helpers/resolve.rs"]
mod source;
use rustgm::{
    lower,
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
    Ok(lower::instances::build_library_prepared(
        &report,
        &mut facts,
        CrateIdentity::new("library", disambiguator)?,
        prepared,
    )?)
}

#[test]
fn escaping_closure_results_fail_proof_before_export_profile_selection()
-> Result<(), Box<dyn std::error::Error>> {
    let invalid = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa24/tests/spec/output-interfaces/escape-outer.rs"),
    )?;
    let Err(error) = compile(&invalid, "escaping") else {
        return Err("escaping closure result was exported".into());
    };
    assert!(
        error
            .to_string()
            .contains("required lifetime does not outlive")
    );
    let valid = "pub fn run()->impl Fn()->u8{||7}";
    let Err(error) = compile(valid, "opaque") else {
        return Err("opaque metadata boundary was silently removed".into());
    };
    assert!(
        error
            .to_string()
            .contains("outside the scalar metadata profile")
    );
    Ok(())
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
        "pub fn f<'a:'b,'b>(n:&'a u32)->&'b u32{n}",
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

#[test]
fn named_scalar_values_export_completed_bits_without_runtime_symbols()
-> Result<(), Box<dyn std::error::Error>> {
    use crates::Scalar;
    let text = "const PRIVATE:u8=3;const fn value()->u8{PRIVATE+4}pub const UNIT:()=();pub const BOOL:bool=true;pub const CHAR:char='🦀';pub const U8:u8=value();pub const U16:u16=65535;pub const U32:u32=4294967295;pub const U64:u64=18446744073709551615;pub const U128:u128=340282366920938463463374607431768211455;pub const USIZE:usize=7;pub const I8:i8=-1;pub const I16:i16=-1;pub const I32:i32=-1;pub const I64:i64=-1;pub const I128:i128=-170141183460469231731687303715884105728;pub const ISIZE:isize=-1;pub const F32:f32=-0.0;pub const F64:f64=-0.0;pub const ZERO:f32=0.0;pub const NAN:f64=0.0/0.0;";
    let (program, interface) = compile(text, "values")?;
    assert!(interface.functions().is_empty());
    assert_eq!(interface.constants().len(), 19);
    assert!(program.symbols.iter().all(|symbol| symbol.body.is_none()));
    let find = |name: &str| {
        interface
            .constants()
            .iter()
            .find(|value| value.definition().path() == [name])
            .map(crates::Constant::value)
            .ok_or("missing named value")
    };
    assert_eq!(find("U8")?.bits(), 7);
    assert_eq!(find("I8")?.bits(), 255);
    assert_eq!(find("I16")?.bits(), 65535);
    assert_eq!(find("I32")?.bits(), u32::MAX.into());
    assert_eq!(find("I64")?.bits(), u64::MAX.into());
    assert_eq!(find("ISIZE")?.bits(), u64::MAX.into());
    assert_eq!(find("I128")?.bits(), 1 << 127);
    assert_eq!(find("F32")?.bits(), 0x8000_0000);
    assert_eq!(find("F64")?.bits(), 0x8000_0000_0000_0000);
    assert_eq!(find("ZERO")?.bits(), 0);
    assert_eq!(find("UNIT")?.ty(), Scalar::Unit);
    let nan = find("NAN")?.bits();
    assert_eq!(nan & 0x7ff0_0000_0000_0000, 0x7ff0_0000_0000_0000);
    assert_ne!(nan & 0x000f_ffff_ffff_ffff, 0);
    let bytes = crates::encode(&interface)?;
    assert_eq!(crates::decode(&bytes)?, interface);
    let (_, reordered) = compile("pub const B:u8=2;pub const A:i8=-1;", "order")?;
    let (_, ordered) = compile("pub const A:i8=-1;pub const B:u8=2;", "order")?;
    assert_eq!(crates::encode(&reordered)?, crates::encode(&ordered)?);
    Ok(())
}

#[test]
fn non_scalar_constant_exports_are_explicit_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    for text in [
        "pub const C:&u8=&7;",
        "pub const C:[u8;2]=[1,2];",
        "fn f()->u8{7}pub const C:fn()->u8=f;",
    ] {
        let report = source::resolve(text, true)?;
        let mut analysis = rustgm::pipeline::analyze_source(&report)?;
        let result = rustgm::sema::crates::export(
            &report,
            &mut analysis.facts,
            CrateIdentity::new("library", "boundary")?,
        );
        assert!(
            result.is_err_and(|error| error
                .to_string()
                .contains("outside the scalar metadata profile")),
            "{text}"
        );
    }
    Ok(())
}
