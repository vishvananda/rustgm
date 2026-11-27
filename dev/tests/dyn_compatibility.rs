//! Dyn compatibility rejects object uses without rejecting ordinary static traits.
use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
    },
    sema::dyn_compat::{Error, Queries},
};
use std::fmt::Write;

const CASES: &[(&str, &str, &str, bool)] = &[
    (
        "gat-opt-out",
        "trait T {type A<'a> where Self:Sized;}",
        "T",
        true,
    ),
    (
        "associated-return",
        "trait T {type A;fn f(&self)->Self::A;}",
        "T<A=u8>",
        true,
    ),
    ("self-return", "trait T {fn f(&self)->Self;}", "T", false),
    (
        "other-projection",
        "trait Aux {type A;}trait T {fn f(&self)-><Self as Aux>::A where Self:Aux;}",
        "T",
        false,
    ),
    (
        "super-self",
        "trait Parent<A:?Sized>{}trait T:Parent<Self>{}",
        "T",
        false,
    ),
    (
        "sized-generic",
        "trait T {fn f<X>(&self) where Self:Sized;}",
        "T",
        true,
    ),
    ("by-value", "trait T {fn f(self);}", "T", true),
    (
        "double-reference",
        "trait T {fn f(self:&&Self);}",
        "T",
        false,
    ),
    (
        "extra-method-bound",
        "trait Aux {}trait T {fn f(&self) where Self:Aux;}",
        "T",
        false,
    ),
    ("generic-method", "trait T {fn f<X>(&self);}", "T", false),
    (
        "lifetime-method",
        "trait T {fn f<'a>(&self,x:&'a u8)->&'a u8;}",
        "T",
        true,
    ),
    (
        "unused-method-lifetime",
        "trait T {fn f<'a>(&self);}",
        "T",
        true,
    ),
    ("constant", "trait T {const X:u8;}", "T", false),
    (
        "sized-associated",
        "trait T {type A where Self:Sized;fn f(&self);}",
        "T",
        true,
    ),
    (
        "trait-says-sized",
        "trait T:Sized {fn f(&self);}",
        "T",
        false,
    ),
    (
        "redundant-self",
        "trait T {fn f(&self) where Self:T;}",
        "T",
        false,
    ),
    (
        "super-self-bound",
        "trait A {}trait T:A {fn f(&self) where Self:A;}",
        "T",
        false,
    ),
    (
        "projected-bound",
        "trait T {type A;fn f(&self) where Self::A:Copy;}",
        "T<A=u8>",
        true,
    ),
    (
        "unrelated-subject",
        "trait A<X:?Sized>{}trait T where ():A<Self> {}",
        "T",
        false,
    ),
    (
        "super-projected",
        "trait A<X:?Sized>{}trait B{type X;}trait T:B+A<Self::X>{}",
        "T<X=u8>",
        false,
    ),
    (
        "generic-const",
        "trait T {fn f<const N:usize>(&self);}",
        "T",
        false,
    ),
    ("static-method", "trait T {fn f();}", "T", false),
    (
        "static-sized",
        "trait T {fn f() where Self:Sized;}",
        "T",
        true,
    ),
    (
        "opaque-result",
        "trait T {fn f(&self)->impl Copy;}",
        "T",
        false,
    ),
    (
        "incompatible-super",
        "trait A{fn f<X>(&self);}trait T:A{}",
        "T",
        false,
    ),
    ("sized-inherited", "trait A:Sized{}trait T:A{}", "T", false),
    (
        "nested-self",
        "trait T {fn f(&self,x:fn(&Self));}",
        "T",
        false,
    ),
    (
        "associated-argument",
        "trait T{type A;fn f(&self,x:Self::A);}",
        "T<A=u8>",
        true,
    ),
];

/// The callable language traits as core declares them: each bounds its
/// argument by the tuple trait and `Fn` refines a conditionally const parent.
const CALLABLES: &str = "\
#![feature(unboxed_closures,no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang=\"pointee_sized\"]pub trait PointeeSized{}
#[lang=\"meta_sized\"]pub trait MetaSized:PointeeSized{}
#[lang=\"sized\"]pub trait Sized:MetaSized{}
#[lang=\"legacy_receiver\"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang=\"clone\"]pub trait Clone:Sized{#[lang=\"clone_fn\"]fn clone(&self)->Self;}
#[lang=\"copy\"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang=\"tuple_trait\"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[lang=\"fn_once\"]pub const trait FnOnce<Args:Tuple>{#[lang=\"fn_once_output\"]type Output;extern \"rust-call\" fn call_once(self,args:Args)->Self::Output;}
#[lang=\"fn_mut\"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern \"rust-call\" fn call_mut(&mut self,args:Args)->Self::Output;}
#[lang=\"fn\"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern \"rust-call\" fn call(&self,args:Args)->Self::Output;}
pub trait Plain{}
pub fn apply(call:&dyn Fn(u8)->u8)->u8{call(1)}
";

fn definition(report: &Report, name: &str) -> Result<DefId, &'static str> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.kind == DefKind::Trait && report.package.names.get(d.name) == name)
        .and_then(|(i, _)| u32::try_from(i).ok())
        .map(DefId)
        .ok_or("missing test trait")
}

#[test]
fn completed_contracts_match_independent_object_compatibility()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, object, accepted) in CASES {
        rustc::qualify(
            &format!("dyn-{name}"),
            &format!("{source}fn check(_: &dyn {object}){{}}"),
            accepted,
        )?;
        let (report, mut facts) = semantic::analyze(source).map_err(|e| format!("{name}: {e}"))?;
        let root = definition(&report, "T")?;
        let mut queries = Queries::default();
        let result = queries.demand(&report, &mut facts, root);
        match result {
            Ok(_) => assert!(accepted, "{name}: unexpected compatibility"),
            Err(error @ Error::Incompatible { .. }) => assert!(!accepted, "{name}: {error}"),
            Err(error) => return Err(format!("{name}: {error}").into()),
        }
    }
    Ok(())
}

#[test]
fn source_callable_traits_keep_only_their_written_supertrait_edges() -> Result<(), String> {
    let report = crate::resolve::source_provider(CALLABLES, "callable_objects")?;
    // Checking `apply` already demands the object contract of `Fn`.
    let mut facts = rustgm::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let mut queries = Queries::default();
    let mut edges = |name: &str| -> Result<usize, String> {
        let id = definition(&report, name)?;
        queries
            .demand(&report, &mut facts, id)
            .map(|contract| contract.supertraits.len())
            .map_err(|e| format!("{name}: {e}"))
    };
    // A bound on the argument is no edge, and a conditionally const parent is one.
    let implicit = edges("Plain")?;
    assert_eq!(edges("FnOnce")?, implicit);
    assert_eq!(edges("FnMut")?, implicit + 1);
    assert_eq!(edges("Fn")?, implicit + 1);
    Ok(())
}

#[test]
fn shared_supertraits_keep_direct_edges_and_reuse_completed_queries() -> Result<(), String> {
    for count in [8, 32, 128] {
        let mut source = "trait T0 {fn value(&self)->u8;}trait T1:T0{}".to_owned();
        for i in 2..count {
            write!(source, "trait T{i}:T{}+T{}{{}}", i - 1, i - 2).map_err(|e| e.to_string())?;
        }
        let (report, mut facts) = semantic::analyze(&source)?;
        let mut queries = Queries::default();
        let root = definition(&report, &format!("T{}", count - 1))?;
        queries
            .demand(&report, &mut facts, root)
            .map_err(|e| e.to_string())?;
        assert!(
            queries.steps <= 30 * count,
            "{} nodes: {} steps",
            count,
            queries.steps
        );
        let before = queries.steps;
        let mut edges = 0;
        let mut methods = 0;
        for i in 0..count {
            let id = definition(&report, &format!("T{i}"))?;
            let contract = queries
                .demand(&report, &mut facts, id)
                .map_err(|e| e.to_string())?;
            edges += contract.supertraits.len();
            methods += contract.methods.len();
        }
        assert_eq!(queries.steps - before, count);
        assert_eq!(methods, 1);
        assert_eq!(edges, 2 * count - 3);
    }
    Ok(())
}

const UNWIND_AUTO_ROLES: &str = r#"
#[lang = "sync"] pub unsafe auto trait Sync {}
#[rustc_diagnostic_item = "unwind_safe_trait"] pub auto trait OwnUnwind {}
#[rustc_diagnostic_item = "ref_unwind_safe_trait"] pub auto trait SharedUnwind {}
impl<T: SharedUnwind + ?Sized> OwnUnwind for &T {}
pub fn require_unwind<T: OwnUnwind>(_value: T) {}
"#;

#[test]
fn source_unwind_object_keeps_the_exact_reference_marker() -> Result<(), Box<dyn std::error::Error>>
{
    use rustgm::types::{Kind, auto_traits::AutoTrait};
    let prefix = CALLABLES.replacen(
        "rustc_attrs,const_trait_impl)",
        "rustc_attrs,const_trait_impl,auto_traits,negative_impls)",
        1,
    ) + UNWIND_AUTO_ROLES;
    let valid = prefix.clone()
        + "pub fn startup(main: &(dyn Fn(u8)->u8+Sync+SharedUnwind)) { require_unwind(main); }";
    let report = crate::resolve::source_provider(&valid, "unwind_object")?;
    let facts = rustgm::pipeline::analyze_source(&report)?.facts;
    let signature = facts
        .functions
        .iter()
        .find(|function| {
            let declared = &report.resolution.definitions[function.definition.0 as usize];
            report.package.names.get(declared.name) == "startup"
        })
        .ok_or("actual startup signature")?
        .signature;
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
        return Err("startup lost the actual signature".into());
    };
    let Some([pointer]) = facts.types.list(parameters) else {
        return Err("startup lost its object input".into());
    };
    let Some(Kind::Reference { referent, .. }) = facts.types.kind(*pointer) else {
        return Err("startup lost its shared object reference".into());
    };
    let Some(Kind::Dynamic { shape, .. }) = facts.types.kind(referent) else {
        return Err("startup lost its erased callable".into());
    };
    assert_eq!(
        facts.types.dynamic_auto_traits(shape),
        Some([AutoTrait::Sync, AutoTrait::RefUnwindSafe].as_slice())
    );

    for (tail, cause) in [
        (
            "pub fn startup(main: &(dyn Fn(u8)->u8+Sync)) { require_unwind(main); }",
            "trait obligation requires an unambiguous matching assumption or implementation",
        ),
        (
            "pub fn startup(main: &(dyn Fn(u8)->u8+Sync+OwnUnwind)) { require_unwind(main); }",
            "trait obligation requires an unambiguous matching assumption or implementation",
        ),
        (
            "impl !OwnUnwind for &mut u8 {} \
             pub fn startup(main: &mut u8) { require_unwind(main); }",
            "trait obligation requires an unambiguous matching assumption or implementation",
        ),
        (
            "impl<T: ?Sized> !OwnUnwind for &mut T {} \
             pub fn startup(main: &mut u8) { require_unwind(main); }",
            "trait obligation requires an unambiguous matching assumption or implementation",
        ),
        (
            "pub struct Cell(u8); impl !SharedUnwind for Cell {} \
             pub fn startup(main: &Cell) { require_unwind(main); }",
            "trait obligation requires an unambiguous matching assumption or implementation",
        ),
        (
            "mod another { pub auto trait SharedUnwind {} } \
             pub fn startup(_main: &(dyn Fn(u8)->u8+Sync+another::SharedUnwind)) {}",
            "object auto traits without a represented marker are not implemented",
        ),
    ] {
        let source = format!("{prefix}{tail}");
        let report = crate::resolve::source_provider(&source, "unwind_object_opponent")?;
        let error = rustgm::pipeline::analyze_source(&report)
            .err()
            .ok_or_else(|| format!("accepted an invalid unwind object premise: {tail}"))?;
        assert!(error.to_string().contains(cause), "{tail}: {error}");
    }
    Ok(())
}

#[test]
fn source_unwind_marker_preserves_declaration_and_wire_boundaries()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::lower::instances::{SourcePublication, build_source_library_profile};
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?
            .replacen("#![no_core]", "#![feature(auto_traits)]\n#![no_core]", 1);
    for (declaration, cause) in [
        (
            r#"#[lang = "unpin"]
            #[rustc_diagnostic_item = "ref_unwind_safe_trait"]
            pub auto trait Mixed {}"#,
            "unwind safety diagnostic identity requires a distinct auto trait",
        ),
        (
            r#"#[rustc_diagnostic_item = "ref_unwind_safe_trait"] pub trait NotAuto {}"#,
            "unwind safety diagnostic identity requires its actual safe module auto trait",
        ),
        (
            r#"#[lang = "ref_unwind_safe_trait"] pub auto trait Forged {}"#,
            "language item",
        ),
    ] {
        let source = format!("{prefix}\n{declaration}");
        let error = crate::resolve::source_provider(&source, "unwind_wrong_claim")
            .and_then(|report| rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string()))
            .err()
            .ok_or("accepted an invalid source marker claim")?;
        assert!(error.contains(cause), "{declaration}: {error}");
    }
    let source = prefix
        + r#"#[rustc_diagnostic_item = "ref_unwind_safe_trait"] pub auto trait SharedUnwind {}"#;
    let report = crate::resolve::source_provider(&source, "unwind_wire_gate")?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual Source owner")?
        .provider
        .clone();
    let marker = b"ref_unwind_safe_trait";
    let has_ref_unwind_wire_binding =
        |interface: &rustgm::support::crates::Interface| -> Result<bool, Box<dyn std::error::Error>> {
            let bytes = rustgm::support::crates::encode(interface)?;
            Ok(bytes.windows(marker.len()).any(|window| window == marker))
        };
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let (_, older) = build_source_library_profile(
        &report,
        &mut facts,
        identity.clone(),
        analysis.prepared,
        SourcePublication::ModuleBodies,
    )?;
    assert!(
        !has_ref_unwind_wire_binding(&older)?,
        "older profile should omit unwind-safety diagnostic binding"
    );
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let (_, current) = build_source_library_profile(
        &report,
        &mut facts,
        identity,
        analysis.prepared,
        SourcePublication::ModuleBodyOperations,
    )?;
    assert!(
        has_ref_unwind_wire_binding(&current)?,
        "current profile should emit unwind-safety diagnostic binding"
    );
    Ok(())
}
