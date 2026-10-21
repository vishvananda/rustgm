//! Entry-free roots retain the ordinary template, ownership and lowering gates.
#[path = "helpers/resolve.rs"]
mod source;
use rustgm::{
    lower::{self, Roots},
    lowir::program::Program,
    resolve::model::DefId,
    sema::{foreign, model::Declarations},
    support::linkage::{Linkage, Naming},
    syntax::item::Safety,
};

fn definition(
    report: &rustgm::resolve::expansion::Report,
    facts: &Declarations,
    name: &str,
) -> Result<DefId, String> {
    facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == name
        })
        .map(|function| function.definition)
        .ok_or_else(|| format!("missing {name}"))
}
fn compile(
    text: &str,
    selected: &[&str],
    executable: bool,
    registered: &[&str],
) -> Result<Program, Box<dyn std::error::Error>> {
    let mut report = source::resolve(text, true)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let exports = selected
        .iter()
        .map(|name| definition(&report, &facts, name))
        .collect::<Result<Vec<_>, _>>()?;
    for name in registered {
        let id = definition(&report, &facts, name)?;
        let signature = facts
            .functions
            .iter()
            .find(|function| function.definition == id)
            .ok_or("missing registered signature")?
            .signature;
        facts
            .foreign
            .get_or_insert_with(Default::default)
            .insert_function(
                id,
                foreign::Function {
                    signature,
                    safety: Safety::Safe,
                    naming: Naming::Rust,
                    linkage: Linkage::Export,
                },
            )?;
    }
    let entry = if executable {
        Some(rustgm::sema::entry::select(&mut report, &facts)?)
    } else {
        None
    };
    Ok(lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        Roots {
            entry,
            exports: &exports,
        },
        prepared,
    )?)
}
fn bodies(program: &Program) -> usize {
    program
        .symbols
        .iter()
        .filter(|symbol| symbol.body.is_some())
        .count()
}

#[test]
fn multiple_library_roots_instantiate_checked_private_templates()
-> Result<(), Box<dyn std::error::Error>> {
    let text =
        "fn id<T:Copy>(n:T)->T{n}pub fn first(n:u32)->u32{id(n)}pub fn second(n:u8)->u8{id(n)}";
    let program = compile(text, &["first", "second", "first"], false, &[])?;
    assert_eq!(bodies(&program), 4);
    assert!(
        program
            .symbols
            .iter()
            .all(|symbol| !symbol.entry && symbol.name != "main")
    );
    Ok(())
}

#[test]
fn registry_exports_are_roots_with_and_without_an_executable()
-> Result<(), Box<dyn std::error::Error>> {
    let text = "fn first()->u32{7}fn second()->u32{9}fn main(){}";
    let library = compile(text, &["first"], false, &["second", "first"])?;
    assert_eq!(bodies(&library), 2);
    assert!(library.symbols.iter().all(|symbol| !symbol.entry));
    let executable = compile(text, &[], true, &["second", "first"])?;
    assert_eq!(bodies(&executable), 4);
    assert_eq!(
        executable
            .symbols
            .iter()
            .filter(|symbol| symbol.entry)
            .count(),
        1
    );
    let aligned = compile("static P:&u8=&7;pub fn f()->u8{*P}", &["f"], false, &[])?;
    assert!(!aligned.globals.is_empty());
    assert!(aligned.symbols.iter().all(|symbol| !symbol.entry));
    Ok(())
}

#[test]
fn source_foreign_entry_alias_is_rejected_even_with_unused_templates()
-> Result<(), Box<dyn std::error::Error>> {
    let text = "unsafe extern \"Rust\"{safe fn exported_main();}use crate::exported_main as main;fn unused<T:Copy>(n:T)->T{n}";
    let Err(error) = compile(text, &[], true, &[]) else {
        return Err("source foreign entry alias unexpectedly accepted".into());
    };
    assert!(
        error
            .to_string()
            .contains("main cannot be declared in a foreign block")
    );
    Ok(())
}

#[test]
fn empty_library_roots_emit_no_adapter_and_still_require_body_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    let empty = compile("pub fn unused()->u32{7}", &[], false, &[])?;
    assert!(empty.symbols.is_empty());
    assert!(
        compile(
            "pub fn good()->u32{7}fn bad<'a>(x:&'a u8)->&'static u8{x}",
            &["good"],
            false,
            &[]
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn executable_wrapper_matches_the_existing_entry_api() -> Result<(), Box<dyn std::error::Error>> {
    let text = "fn f(n:u32)->u32{n+1}fn main(){let _=f(7);}";
    let generalized = compile(text, &[], true, &[])?;
    let mut report = source::resolve(text, true)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let original = lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    let (mut left, mut right) = (Vec::new(), Vec::new());
    rustgm::lowir::writer::write(&generalized, &mut left)?;
    rustgm::lowir::writer::write(&original, &mut right)?;
    assert_eq!(left, right);
    Ok(())
}

#[test]
fn packed_address_fixups_require_an_executable_initialization_adapter()
-> Result<(), Box<dyn std::error::Error>> {
    let text =
        "#[repr(C,packed)]struct S{a:u8,p:&'static u8}static X:S=S{a:1,p:&7};pub fn f()->u8{*X.p}";
    let Err(error) = compile(text, &["f"], false, &[]) else {
        return Err("packed library unexpectedly omitted its initialization".into());
    };
    assert!(
        error
            .to_string()
            .contains("entry-free library requires packed address initialization support")
    );
    let executable = compile(&format!("{text}fn main(){{let _=f();}}"), &[], true, &[])?;
    assert_eq!(
        executable
            .symbols
            .iter()
            .filter(|symbol| symbol.entry)
            .count(),
        1
    );
    Ok(())
}
