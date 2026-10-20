//! Object annotations use the same declaration and lifetime pipeline as other types.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "reference",
        "trait T{fn f(&self)->u8;}fn check(_: &dyn T){}",
        true,
    ),
    (
        "mutable",
        "trait T{fn f(&mut self);}fn check(_: &mut dyn T){}",
        true,
    ),
    ("raw", "trait T{}fn check(_: *const dyn T){}", true),
    ("alias", "trait T{}type A=dyn T;fn check(_: &A){}", true),
    (
        "recursive-signature",
        "trait T{fn f(&self)->&dyn T;}fn check(_: &dyn T){}",
        true,
    ),
    (
        "generic-argument",
        "trait T<A>{fn f(&self,x:A);}fn check(_: &dyn T<u8>){}",
        true,
    ),
    (
        "generic-default",
        "trait T<A=u8>{fn f(&self,x:A);}fn check(_: &dyn T){}",
        true,
    ),
    (
        "self-default",
        "trait T<A:?Sized=Self>{}fn check(_: &dyn T){}",
        false,
    ),
    (
        "explicit-self-default",
        "trait T<A:?Sized=Self>{}fn check(_: &dyn T<u8>){}",
        true,
    ),
    (
        "associated",
        "trait T{type A;fn f(&self)->Self::A;}fn check(_: &dyn T<A=u8>){}",
        true,
    ),
    (
        "missing-associated",
        "trait T{type A;}fn check(_: &dyn T){}",
        false,
    ),
    (
        "associated-inherited",
        "trait P{type A;}trait T:P{}fn check(_: &dyn T<A=u8>){}",
        true,
    ),
    (
        "unknown-associated",
        "trait T{}fn check(_: &dyn T<A=u8>){}",
        false,
    ),
    (
        "duplicate-associated",
        "trait T{type A;}fn check(_: &dyn T<A=u8,A=u8>){}",
        false,
    ),
    (
        "sized-associated",
        "trait T{type A where Self:Sized;}fn check(_: &dyn T){}",
        true,
    ),
    (
        "sized-gat",
        "trait T{type A<'a> where Self:Sized;}fn check(_: &dyn T){}",
        true,
    ),
    (
        "generic-method",
        "trait T{fn f<A>(&self);}fn check(_: &dyn T){}",
        false,
    ),
    (
        "static-opt-out",
        "trait T{fn f<A>() where Self:Sized;}fn check(_: &dyn T){}",
        true,
    ),
    (
        "duplicate-principal",
        "trait T{}fn check(_: &(dyn T+T)){}",
        false,
    ),
    (
        "two-principals",
        "trait T{}trait U{}fn check(_: &(dyn T+U)){}",
        false,
    ),
    ("sized", "trait T:Sized{}fn check(_: &dyn T){}", false),
    ("copy", "fn check(_: &dyn Copy){}", false),
    (
        "named-lifetime",
        "trait T{}fn check<'a>(_: &'a (dyn T+'a)){}",
        true,
    ),
    (
        "placeholder-lifetime",
        "trait T{}fn check(_: &(dyn T+'_)){}",
        true,
    ),
    (
        "higher-ranked",
        "trait T<'a>{fn f(&self,x:&'a u8);}fn check(_: &dyn for<'a> T<'a>){}",
        true,
    ),
    (
        "lifetime-argument",
        "trait T<'a>{}fn check<'a>(_: &'a dyn T<'a>){}",
        true,
    ),
    (
        "captured-binding",
        "trait T{type A;}fn check<'a>(_: &'a dyn T<A=&'a u8>){}",
        true,
    ),
    (
        "trait-lifetime",
        "trait T<'a>:'a{}fn check<'a>(_: *const dyn T<'a>){}",
        true,
    ),
    (
        "multiple-trait-lifetimes",
        "trait T<'a,'b>:'a+'b{}fn check<'a,'b>(_: *const dyn T<'a,'b>){}",
        true,
    ),
    (
        "quantified-default",
        "trait T<'a>:'a{}fn f(_: *const dyn for<'a> T<'a>){}",
        true,
    ),
    (
        "trait-copy-arg",
        "struct N(u8);trait T<A:Copy>{}fn f(_: &dyn T<N>){}",
        true,
    ),
    (
        "trait-sized-arg",
        "trait T<A>{}fn f(_: &dyn T<str>){}",
        true,
    ),
    (
        "associated-copy",
        "struct N(u8);trait T{type A:Copy;}fn f(_: &dyn T<A=N>){}",
        true,
    ),
    (
        "multiple-alias-lifetimes",
        "trait T<'a,'b>:'a+'b{}type A<'a,'b>=dyn T<'a,'b>;",
        false,
    ),
    (
        "multiple-early-lifetimes",
        "trait T<'a,'b>:'a+'b{}fn f<'a:'a,'b:'b>(_: *const dyn T<'a,'b>){}",
        false,
    ),
    (
        "single-alias-lifetime",
        "trait T<'a>:'a{}type A<'a>=dyn T<'a>;",
        true,
    ),
    (
        "quantified-alias-default",
        "trait T<'a>:'a{}type A=dyn for<'a> T<'a>;",
        true,
    ),
    (
        "inherited-equality",
        "trait P{type A;}trait T:P<A=u8>{}fn f(_: &dyn T){}",
        true,
    ),
    (
        "inherited-explicit",
        "trait P{type A;}trait T:P<A=u8>{}fn f(_: &dyn T<A=u8>){}",
        true,
    ),
    (
        "inherited-conflict",
        "trait P{type A;}trait T:P<A=u8>{}fn f(_: &dyn T<A=u16>){}",
        true,
    ),
    (
        "inherited-diamond",
        "trait P{type A;}trait L:P<A=u8>{}trait R:P<A=u8>{}trait T:L+R{}fn f(_: &dyn T){}",
        true,
    ),
    (
        "inherited-quantified",
        "trait P<'a>{type A;}trait T:for<'a>P<'a,A=&'a u8>{}fn f(_: &dyn T){}",
        true,
    ),
    (
        "inherited-lifetime",
        "trait P<'a>{type A;}trait T<'a>:P<'a,A=&'a u8>{}fn f<'a>(_: &dyn T<'a>){}",
        true,
    ),
    (
        "quantified-explicit",
        "trait P<'a>{type A;}trait T:for<'a>P<'a>{}fn f(_: &dyn T<A=u8>){}",
        true,
    ),
    (
        "quantified-explicit-capture",
        "trait P<'a>{type A;}trait T:for<'a>P<'a>{}fn f<'b>(_: &dyn T<A=&'b u8>){}",
        true,
    ),
    (
        "quantified-two-binders",
        "trait P<'a,'b>{type A;}trait T<'a>:for<'b>P<'a,'b>{}fn f(_: &dyn for<'a>T<'a,A=&'a u8>){}",
        true,
    ),
    (
        "quantified-chain",
        "trait P<'a,'b>{type A;}trait Q<'a>:for<'b>P<'a,'b>{}trait T:for<'a>Q<'a>{}fn f(_: &dyn T<A=u8>){}",
        true,
    ),
    (
        "quantified-diamond",
        "trait P<'a>{type A;}trait L:for<'a>P<'a>{}trait R:for<'a>P<'a>{}trait T:L+R{}fn f(_: &dyn T<A=u8>){}",
        true,
    ),
    (
        "quantified-missing",
        "trait P<'a>{type A;}trait T:for<'a>P<'a>{}fn f(_: &dyn T){}",
        false,
    ),
    (
        "quantified-ambiguous",
        "trait P<'a>{type A;}trait Q<'a>{type A;}trait T:for<'a>P<'a>+for<'a>Q<'a>{}fn f(_: &dyn T<A=u8>){}",
        false,
    ),
    (
        "quantified-name",
        "trait P<'a>{type A;}trait T:for<'a>P<'a>{}fn f(_: &dyn T<B=u8>){}",
        false,
    ),
    (
        "unconstrained-object-binding",
        "trait T{type A;}fn f(_: &dyn for<'a>T<A=&'a u8>){}",
        false,
    ),
    (
        "unconstrained-inherited-binding",
        "trait P<'a>{type A;}trait T:for<'a>P<'a>{}fn f(_: &dyn for<'x>T<A=&'x u8>){}",
        false,
    ),
    (
        "unconstrained-parent-binding",
        "trait P<'a>{type A;}trait T<'a>:P<'static>{}fn f(_: &dyn for<'a>T<'a,A=&'a u8>){}",
        false,
    ),
];

#[test]
fn annotations_match_independent_source_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("object-{name}"), source, accepted)?;
        let result = semantic::analyze(source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn defaults_retain_the_binding_time_of_trait_lifetimes() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{Kind, RegionId};
    let cases = [
        ("late", "fn f<'a>(_: *const dyn T<'a>){}", true),
        ("early", "fn f<'a:'a>(_: *const dyn T<'a>){}", false),
        (
            "early-over-reference",
            "fn f<'a:'a>(_: &dyn T<'a>){}",
            false,
        ),
        (
            "late-alias-input",
            "type A<'a>=&'a u8;fn f<'a>(_:A<'a>)->*const dyn T<'a>{loop{}}",
            true,
        ),
        (
            "early-erased-alias",
            "type A<'a>=u8;fn f<'a>(_:A<'a>)->*const dyn T<'a>{loop{}}",
            false,
        ),
    ];
    for (name, fragment, static_) in cases {
        let source = format!("trait T<'a>:'a{{}}{fragment}");
        rustc::qualify(&format!("object-time-{name}"), &source, true)?;
        let (report, facts) = semantic::analyze(&source)?;
        let function = facts
            .functions
            .iter()
            .find(|function| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[function.definition.0 as usize].name)
                    == "f"
            })
            .ok_or("missing test function")?;
        let Some(Kind::Function {
            parameters, result, ..
        }) = facts.types.kind(function.signature)
        else {
            return Err("missing test signature".into());
        };
        let pointer = if result == rustgm::types::TypeId::UNIT {
            *facts
                .types
                .list(parameters)
                .and_then(|list| list.first())
                .ok_or("missing test parameter")?
        } else {
            result
        };
        let referent = match facts.types.kind(pointer) {
            Some(Kind::Reference { referent, .. }) => referent,
            Some(Kind::Pointer { pointee, .. }) => pointee,
            _ => return Err("missing object pointer".into()),
        };
        let Some(Kind::Dynamic { region, .. }) = facts.types.kind(referent) else {
            return Err("missing source object".into());
        };
        assert_eq!(region == RegionId::STATIC, static_, "{name}: {region:?}");
    }
    Ok(())
}
