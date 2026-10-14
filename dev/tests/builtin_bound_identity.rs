#[path = "helpers/resolve.rs"]
mod names;
#[path = "helpers/qualify.rs"]
mod rustc;
use rustgm::resolve::model::{BuiltinTrait, DefKind, Target};

const CASES: &[(&str, &str, bool)] = &[
    ("prelude", "fn f<T:Copy+Clone+Sized>(){}", true),
    (
        "qualified",
        "fn f<T:core::marker::Copy+std::clone::Clone+std::marker::Sized>(){}",
        true,
    ),
    (
        "forward-alias",
        "use c::marker::Copy as C;use core as c;fn f<T:C>(){}",
        true,
    ),
    (
        "forward-module",
        "use m::Copy as C;use core::marker as m;fn f<T:C>(){}",
        true,
    ),
    (
        "reexport",
        "mod m{pub use core::marker::Copy as C;}use m::*;fn f<T:C>(){}",
        true,
    ),
    (
        "glob",
        "use core::marker::*;use core::clone::*;fn f<T:Copy+Clone+Sized>(){}",
        true,
    ),
    (
        "module-self",
        "use core::marker::{self as m};fn f<T:m::Copy>(){}",
        true,
    ),
    ("no-std", "#![no_std]fn f<T:Copy+Clone+Sized>(){}", true),
    (
        "explicit-std",
        "#![no_std]extern crate std;fn f<T:std::marker::Copy>(){}",
        true,
    ),
    (
        "explicit-core",
        "#![no_implicit_prelude]extern crate core;fn f<T:core::marker::Copy>(){}",
        true,
    ),
    (
        "absolute-shadow",
        "mod core{}fn f<T: ::core::marker::Copy>(){}",
        true,
    ),
    ("shadow", "trait Copy{}fn f<T:Copy>(){}", false),
    (
        "shadow-import",
        "mod m{pub trait C{}}use m::C as Copy;fn f<T:Copy>(){}",
        false,
    ),
    (
        "shadow-core",
        "mod core{pub mod marker{pub trait Copy{}}}fn f<T:core::marker::Copy>(){}",
        false,
    ),
    (
        "macro-import",
        "macro_rules! m{()=>{use core::marker::Copy as C;fn f<T:C>(){}}}m!();",
        true,
    ),
    (
        "macro-shadow",
        "macro_rules! m{()=>{trait Copy{}fn f<T:Copy>(){}}}m!();",
        false,
    ),
];

#[test]
fn builtin_bounds_retain_definition_identity_through_resolution_and_expansion()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source, builtin) in CASES {
        rustc::qualify(&format!("bound-identity-{name}"), source, true)?;
        for expanded in [false, true] {
            let report = names::resolve(source, expanded).map_err(|e| format!("{name}: {e}"))?;
            let mut checked = 0;
            for reference in &report.resolution.references {
                let Target::Definition(id) = reference.target else {
                    continue;
                };
                let definition = &report.resolution.definitions[id.0 as usize];
                if !matches!(definition.kind, DefKind::Trait | DefKind::BuiltinTrait) {
                    continue;
                }
                assert_eq!(
                    report.resolution.builtin_traits.identify(id).is_some(),
                    *builtin,
                    "{name}"
                );
                assert_eq!(definition.kind == DefKind::BuiltinTrait, *builtin, "{name}");
                checked += 1;
            }
            assert!(checked != 0, "{name}");
        }
    }
    Ok(())
}

#[test]
fn external_traits_are_unique_and_loaded_by_module_demand() -> Result<(), String> {
    let report = names::resolve("fn f<T:Clone>(){}", true)?;
    assert!(
        report
            .resolution
            .builtin_traits
            .get(BuiltinTrait::Clone)
            .is_some()
    );
    assert!(
        report
            .resolution
            .builtin_traits
            .get(BuiltinTrait::Copy)
            .is_none()
    );
    assert!(
        report
            .resolution
            .builtin_traits
            .get(BuiltinTrait::Sized)
            .is_none()
    );
    let report = names::resolve(
        "use core::marker::Copy as C;use std::marker::Copy as D;fn f<T:Copy+C+D+Clone+Sized>(){}",
        true,
    )?;
    for trait_ in [BuiltinTrait::Copy, BuiltinTrait::Clone, BuiltinTrait::Sized] {
        assert_eq!(
            report
                .resolution
                .definitions
                .iter()
                .filter(|d| d.kind == DefKind::BuiltinTrait
                    && report.package.names.get(d.name) == trait_.name())
                .count(),
            1
        );
    }
    Ok(())
}

#[test]
fn trait_metadata_respects_namespace_and_prelude_policy() -> Result<(), Box<dyn std::error::Error>>
{
    for (index, source) in [
        "use crate::Copy;",
        "use ::Copy;",
        "fn f(){Copy;}",
        "use core::marker::Missing;",
        "use core::clone::Copy;",
        "#![no_std]use std::marker::Copy;",
        "#![no_implicit_prelude]fn f<T:Copy>(){}",
        "#[no_implicit_prelude]mod m{fn f<T:Clone>(){}}",
        "mod core{}use core::marker::Copy;",
    ]
    .iter()
    .enumerate()
    {
        rustc::qualify(&format!("bound-name-reject-{index}"), source, false)?;
        for expanded in [false, true] {
            assert!(names::resolve(source, expanded).is_err(), "{source}");
        }
    }
    Ok(())
}
