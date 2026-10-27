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
