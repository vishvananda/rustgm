use crate::qualify as rustc;
use crate::semantic;
use rustc::qualify;
use rustgm::{
    resolve::model::{DefKind, Target},
    syntax::item::ItemKind,
};

use crate::resolve as names;
use names::resolve;

const CASES: &[(&str, &str, bool)] = &[
    (
        "no-std-prelude",
        "#![no_std]impl Drop for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "no-std-core",
        "#![no_std]impl core::ops::Drop for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "no-std-explicit",
        "#![no_std]extern crate std;impl std::ops::Drop for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "disabled-absolute",
        "#![no_implicit_prelude]impl ::core::ops::Drop for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "disabled-explicit",
        "#![no_implicit_prelude]extern crate core;impl core::ops::Drop for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "disabled-nested",
        "#![no_implicit_prelude]extern crate core;mod m{impl crate::core::ops::Drop for crate::S{fn drop(&mut self){}}}",
        true,
    ),
    (
        "disabled-module",
        "#[no_implicit_prelude]mod m{impl ::core::ops::Drop for crate::S{fn drop(&mut self){}}}",
        true,
    ),
    (
        "forward-extern",
        "use ::c::ops::Drop as D;extern crate core as c;impl D for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "cfg-no-std",
        "#![cfg_attr(all(),no_std)]impl core::ops::Drop for S{fn drop(&mut self){}}",
        true,
    ),
    ("prelude", "impl Drop for S{fn drop(&mut self){}}", true),
    (
        "core",
        "impl core::ops::Drop for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "std",
        "impl std::ops::Drop for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "absolute",
        "impl ::core::ops::Drop for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "alias",
        "use core::ops::Drop as D;impl D for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "glob",
        "use core::ops::*;impl Drop for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "group",
        "use core::{ops::{Drop as D}};impl D for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "crate-alias",
        "use core as c;impl c::ops::Drop for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "reexport",
        "mod m{pub use std::ops::Drop as D;}use m::D;impl D for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "shadow-trait",
        "trait Drop{fn drop(&mut self);}impl Drop for S{fn drop(&mut self){}}",
        false,
    ),
    (
        "shadow-import",
        "mod m{pub trait T{fn drop(&mut self);}}impl Drop for S{fn drop(&mut self){}}use m::T as Drop;",
        false,
    ),
    (
        "shadow-crate",
        "mod core{pub mod ops{pub trait Drop{fn drop(&mut self);}}}impl core::ops::Drop for S{fn drop(&mut self){}}",
        false,
    ),
    (
        "absolute-shadow",
        "mod core{}impl ::core::ops::Drop for S{fn drop(&mut self){}}",
        true,
    ),
    (
        "macro",
        "macro_rules! d{()=>{use core::ops::Drop as D;impl D for S{fn drop(&mut self){}}}}d!();",
        true,
    ),
    (
        "macro-glob",
        "macro_rules! d{()=>{use core::ops::*;impl Drop for S{fn drop(&mut self){}}}}d!();",
        true,
    ),
];

#[test]
fn drop_identity_survives_imports_expansion_and_lexical_shadowing()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, builtin) in CASES {
        let source = format!("{source}struct S;");
        qualify(name, &source, true)?;
        for expanded in [false, true] {
            let report = resolve(&source, expanded).map_err(|e| format!("{name}: {e}"))?;
            let mut checked = 0;
            for reference in &report.resolution.references {
                let ast = &report.package.unit(reference.node.file).configured.ast;
                // A path type is its path node, which carries the reference.
                if !ast.items.iter().any(|item| {
                    matches!(item.kind,
                    ItemKind::Impl{trait_path:Some(path),..} if path == reference.node.node)
                }) {
                    continue;
                }
                let Target::Definition(id) = reference.target else {
                    return Err(format!("{name}: trait path has no definition").into());
                };
                assert_eq!(report.resolution.drop_trait == Some(id), builtin, "{name}");
                assert_eq!(
                    report.resolution.definitions[id.0 as usize].kind == DefKind::BuiltinTrait,
                    builtin,
                    "{name}"
                );
                checked += 1;
            }
            assert_eq!(checked, 1, "{name}");
        }
        if builtin {
            let (_, facts) = semantic::analyze(&source).map_err(|e| format!("{name}: {e}"))?;
            assert_eq!(facts.destructors.implementations().len(), 1, "{name}");
        }
    }
    Ok(())
}

#[test]
fn ordinary_programs_do_not_allocate_language_trait_metadata() -> Result<(), String> {
    for expanded in [false, true] {
        let report = resolve("struct S;fn f(s:S)->S{s}", expanded)?;
        assert!(report.resolution.drop_trait.is_none());
        assert!(report.resolution.extern_scope.is_none());
        assert!(
            !report
                .resolution
                .definitions
                .iter()
                .any(|d| d.kind == DefKind::BuiltinTrait)
        );
    }
    let (report, facts) = semantic::analyze("struct S;fn f(s:S)->S{s}")?;
    let program = rustgm::body::build(&report, &facts).map_err(|e| e.to_string())?;
    assert!(program.drops.is_none());
    Ok(())
}

#[test]
fn executable_typing_cannot_silently_erase_a_drop_implementation() -> Result<(), String> {
    assert!(semantic::analyze("struct S;impl Drop for S{}fn main(){}").is_err());
    for source in [
        "struct S;impl Drop for S{fn drop(self:&mut S){}}fn main(){let _s=S;}",
        "use core::ops::Drop as D;struct S;impl D for S{fn drop(self:&mut S){}}fn main(){let _s=S;}",
    ] {
        let (report, mut facts) = semantic::analyze(source)?;
        let program = rustgm::body::build(&report, &facts).map_err(|e| e.to_string())?;
        let error = rustgm::borrow::ownership::verify(
            &facts.types,
            &facts.aggregates,
            &facts.derives,
            &mut facts.copies,
            &program,
        )
        .err()
        .ok_or_else(|| format!("destructor was accepted without elaboration: {source}"))?;
        assert!(
            error.message.contains("require destructor elaboration"),
            "{}",
            error.message
        );
    }
    Ok(())
}

#[test]
fn drop_is_not_a_crate_root_item_or_value() -> Result<(), Box<dyn std::error::Error>> {
    for (index, source) in [
        "use crate::Drop;",
        "use ::Drop;",
        "use core::ops::Missing;",
        "mod core{}use core::ops::Drop;",
        "fn f(){Drop;}",
        "#![no_std]struct S;impl std::ops::Drop for S{fn drop(&mut self){}}",
        "#![no_std]struct S;impl ::std::ops::Drop for S{fn drop(&mut self){}}",
        "#![no_implicit_prelude]struct S;impl Drop for S{fn drop(&mut self){}}",
        "#![no_implicit_prelude]struct S;impl core::ops::Drop for S{fn drop(&mut self){}}",
        "#[no_implicit_prelude]mod m{struct S;impl Drop for S{fn drop(&mut self){}}}",
        "#[no_implicit_prelude]mod m{mod n{struct S;impl Drop for S{fn drop(&mut self){}}}}",
        "mod m{#![no_implicit_prelude]struct S;impl Drop for S{fn drop(&mut self){}}}",
        "#![no_implicit_prelude]extern crate core;mod m{struct S;impl core::ops::Drop for S{fn drop(&mut self){}}}",
    ].into_iter().enumerate() {
        qualify(&format!("reject-{index}"), source, false)?;
        for expanded in [false, true] {
            assert!(resolve(source, expanded).is_err(), "{source}");
        }
    }
    // A single builtin declaration serves all qualified/prelude aliases.
    let report = resolve(
        "use core::ops::Drop as A;use std::ops::Drop as B;struct S;impl Drop for S{fn drop(&mut self){}}",
        true,
    )?;
    assert_eq!(
        report
            .resolution
            .definitions
            .iter()
            .filter(|d| d.kind == DefKind::BuiltinTrait)
            .count(),
        1
    );
    Ok(())
}
