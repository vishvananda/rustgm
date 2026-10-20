//! Call bounds retain ordinary trait proof, distinct identities and scoped Output equalities.
#[path = "helpers/resolve.rs"]
mod names;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    resolve::model::{BuiltinTrait, DefKind, Target},
    support::call_traits::CallTrait,
};

const CASES: &[(&str, &str, bool)] = &[
    ("plain", "fn f<F:Fn(u8)->u16>(){}", true),
    ("default-output", "fn f<F:FnMut()>(){}", true),
    (
        "explicit-binder",
        "fn f<F:for<'a>Fn(&'a u8)->&'a u8>(){}",
        true,
    ),
    ("elided", "fn f<F:Fn(&u8)->&u8>(){}", true),
    ("outer-lifetime", "fn f<'a,F:Fn()->&'a u8>(){}", true),
    (
        "unconstrained-output",
        "fn f<F:for<'a>Fn()->&'a u8>(){}",
        false,
    ),
    (
        "subject-only-output",
        "fn f<F>()where for<'a> &'a F:Fn()->&'a u8{}",
        true,
    ),
    ("angle", "fn f<F:Fn<(u8,),Output=u16>>(){}", false),
    ("empty-angle", "fn f<F:Fn<>>(){}", false),
    ("user-parentheses", "trait Fn{} fn f<F:Fn()>(){}", false),
    (
        "call-hierarchy",
        "fn once<F:FnOnce(u8)->u16>(){} fn shared<F:Fn(u8)->u16>(){once::<F>();}",
        true,
    ),
    (
        "wrong-hierarchy",
        "fn shared<F:Fn(u8)->u16>(){} fn once<F:FnOnce(u8)->u16>(){shared::<F>();}",
        false,
    ),
    (
        "output-mismatch",
        "fn need<F:Fn(u8)->u8>(){} fn other<F:Fn(u8)->u16>(){need::<F>();}",
        false,
    ),
    (
        "inherited-output",
        "fn f<F:Fn(u8)->u16>(x:F::Output)->u16{x}",
        true,
    ),
    ("ambiguous-elision", "fn f<F:Fn(&u8,&u8)->&u8>(){}", false),
    ("unsized-output", "fn f<F:Fn()->str>(){}", true),
    ("unsized-input", "fn f<F:Fn(str)>(){}", false),
    (
        "wrong-input",
        "fn need<F:Fn(u8)>(){} fn other<F:Fn(u16)>(){need::<F>();}",
        false,
    ),
    (
        "mutable-hierarchy",
        "fn once<F:FnOnce(u8)->u16>(){} fn mutable<F:FnMut(u8)->u16>(){once::<F>();}",
        true,
    ),
    (
        "shared-mutable",
        "fn mutable<F:FnMut(u8)->u16>(){} fn shared<F:Fn(u8)->u16>(){mutable::<F>();}",
        true,
    ),
    (
        "default-output-mismatch",
        "fn need<F:Fn()>(){}fn other<F:Fn()->u8>(){need::<F>();}",
        false,
    ),
    (
        "shadow-implementation",
        "trait Fn{}impl Fn for u8{}fn need<F:Fn>(){}fn f(){need::<u8>();}",
        true,
    ),
    (
        "caller-binder",
        "fn need<F:for<'a>Fn(&'a u8)->&'a u8>(){} fn f<F:Fn(&u8)->&u8>(){need::<F>();}",
        true,
    ),
    (
        "bound-lifetime-escape",
        "fn need<F:for<'a>Fn(&'a u8)->&'a u8>(){}fn f<F:Fn(&'static u8)->&'static u8>(){need::<F>();}",
        false,
    ),
    (
        "same-input-lifetime",
        "fn f<F:for<'a>Fn(&'a u8,&'a u8)->&u8>(){}",
        false,
    ),
    (
        "separate-elision-contexts",
        "fn f<F>()where for<'a>F:Fn(&'a u8)->&u8+Fn(&'a u16)->&u16{}",
        true,
    ),
];

#[test]
fn callable_contracts_use_scoped_trait_assumptions() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for (name, source, accepted) in CASES {
        rustc::qualify(&format!("call-contract-{name}"), source, *accepted)?;
        let result = checked(source);
        if result.is_ok() != *accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn checked(source: &str) -> Result<(), String> {
    let source = format!("{source}\nfn main(){{}}");
    let (mut report, mut facts) = semantic::analyze(&source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[test]
fn callable_identity_survives_aliases_globs_and_shadowing() -> Result<(), Box<dyn std::error::Error>>
{
    for (name, source, builtin) in [
        ("prelude", "fn f<F:Fn()+FnMut()+FnOnce()>(){}", true),
        (
            "qualified",
            "fn f<F:core::ops::Fn(u8)+std::ops::FnOnce(u8)>(){}",
            true,
        ),
        (
            "forward",
            "use o::Fn as C;use core::ops as o;fn f<F:C()>(){}",
            true,
        ),
        (
            "reexport",
            "mod m{pub use core::ops::*;}use m::Fn as C;fn f<F:C()>(){}",
            true,
        ),
        ("glob", "use core::ops::*;fn f<F:FnMut()>(){}", true),
        ("no-std", "#![no_std]fn f<F:FnOnce()>(){}", true),
        (
            "explicit-core",
            "#![no_implicit_prelude]extern crate core;fn f<F:core::ops::Fn()>(){}",
            true,
        ),
        ("shadow", "trait Fn{}fn f<F:Fn>(){}", false),
        (
            "shadow-module",
            "mod core{pub mod ops{pub trait Fn{}}}fn f<F:core::ops::Fn>(){}",
            false,
        ),
        ("absolute", "mod core{}fn f<F: ::core::ops::Fn()>(){}", true),
    ] {
        rustc::qualify(&format!("call-identity-{name}"), source, true)?;
        for expanded in [false, true] {
            let report = names::resolve(source, expanded).map_err(|e| format!("{name}: {e}"))?;
            let mut checked = 0;
            for reference in &report.resolution.references {
                let Target::Definition(id) = reference.target else {
                    continue;
                };
                if !matches!(
                    report.resolution.definitions[id.0 as usize].kind,
                    DefKind::Trait | DefKind::BuiltinTrait
                ) {
                    continue;
                }
                assert_eq!(
                    matches!(
                        report.resolution.builtin_traits.identify(id),
                        Some(BuiltinTrait::Callable(_))
                    ),
                    builtin,
                    "{name}"
                );
                checked += 1;
            }
            assert!(checked > 0, "{name}");
        }
    }
    let report = names::resolve("fn f<F:Fn()>(){}", true)?;
    let metadata = &report.resolution.builtin_traits;
    assert!(
        metadata
            .callable(CallTrait::Fn)
            .is_some_and(|m| m.output.is_none())
    );
    assert!(
        metadata
            .callable(CallTrait::FnMut)
            .is_some_and(|m| m.output.is_none())
    );
    assert!(
        metadata
            .callable(CallTrait::FnOnce)
            .is_some_and(|m| m.output.is_some())
    );
    Ok(())
}

#[test]
fn callable_metadata_does_not_extend_user_namespaces() -> Result<(), Box<dyn std::error::Error>> {
    for (index, source) in [
        "#![no_implicit_prelude]fn f<F:Fn()>(){}",
        "#![no_std]use std::ops::Fn;",
        "use core::marker::Fn;",
        "use crate::Fn;",
        "mod core{}use core::ops::Fn;",
        "fn f(){Fn;}",
        "use core::ops::FnOnce::Args;",
        "use core::ops::FnOnce::Output;",
    ]
    .iter()
    .enumerate()
    {
        rustc::qualify(&format!("call-names-reject-{index}"), source, false)?;
        assert!(names::resolve(source, true).is_err(), "{source}");
    }
    let report = names::resolve("use core::ops::*;", true)?;
    assert!(
        report
            .resolution
            .builtin_traits
            .callable(CallTrait::Fn)
            .is_none()
    );
    Ok(())
}

#[test]
fn callable_parentheses_survive_expansion_without_a_written_output() -> Result<(), String> {
    let report = names::resolve(
        "macro_rules! m{()=>{fn f<F:Fn(),G:Fn<()>>() {}}}m!();",
        true,
    )?;
    let mut parentheses = false;
    let mut angles = false;
    for unit in &report.package.units {
        for node in &unit.configured.ast.nodes {
            if let rustgm::syntax::ast::Kind::Applied {
                parenthesized,
                output: None,
                ..
            } = node.kind
            {
                parentheses |= parenthesized;
                angles |= !parenthesized;
            }
        }
    }
    assert!(parentheses && angles);
    Ok(())
}
