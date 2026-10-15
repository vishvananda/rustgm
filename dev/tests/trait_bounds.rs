//! User predicates preserve complete arguments without inventing implementation evidence.
#[path = "trait_bounds/applicability.rs"]
mod applicability;
#[path = "trait_bounds/binders.rs"]
mod binders;
#[path = "trait_bounds/callables.rs"]
mod callables;
#[path = "trait_bounds/conformance.rs"]
mod conformance;
#[path = "trait_bounds/declarations.rs"]
mod declarations;
#[path = "trait_bounds/destructors.rs"]
mod destructors;
#[path = "trait_bounds/dispatch.rs"]
mod dispatch;
#[path = "trait_bounds/dispatch_extra.rs"]
mod dispatch_extra;
#[path = "trait_bounds/dispatch_native.rs"]
mod dispatch_native;
#[path = "trait_bounds/equality.rs"]
mod equality;
#[path = "trait_bounds/implementations.rs"]
mod implementations;
#[path = "trait_bounds/implicit_native.rs"]
mod implicit_native;
#[path = "trait_bounds/implicit_paths.rs"]
mod implicit_paths;
#[path = "trait_bounds/implicit_scope.rs"]
mod implicit_scope;
#[path = "trait_bounds/inference.rs"]
mod inference;
#[path = "harness/native_references.rs"]
mod native;
#[path = "trait_bounds/qualification.rs"]
mod qualification;
#[path = "trait_bounds/qualification_native.rs"]
mod qualification_native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "trait_bounds/selection.rs"]
mod selection;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::resolve::model::{DefId, DefKind};

fn prove(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(source)?;
    let main = report
        .resolution
        .definitions
        .iter()
        .position(|d| d.kind == DefKind::Function && report.package.names.get(d.name) == "main")
        .ok_or("missing main")?;
    rustgm::lower::instances::build(&report, &mut facts, DefId(u32::try_from(main)?))?;
    Ok(())
}

fn cases(rows: &[(&str, &str, bool)]) -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in rows {
        rustc::qualify(name, source, accepted)?;
        let result = prove(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
fn arguments_require_their_own_proofs() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "trait-argument-copy-missing",
            "trait A<U:Copy>{}fn f<T:A<U>,U>(x:T){}fn main(){}",
            false,
        ),
        (
            "trait-argument-copy-given",
            "trait A<U:Copy>{}fn f<T:A<U>,U:Copy>(x:T){}fn main(){}",
            true,
        ),
        (
            "trait-argument-sized-missing",
            "trait A<U>{}fn f<T:A<str>>(x:T){}fn main(){}",
            false,
        ),
        (
            "trait-argument-sized-relaxed",
            "trait A<U:?Sized>{}fn f<T:A<str>>(x:T){}fn main(){}",
            true,
        ),
        (
            "trait-argument-closed-noncopy",
            "trait A<U:Copy>{}struct N;fn f<T:A<N>>(x:T){}fn main(){}",
            false,
        ),
        (
            "trait-argument-where-missing",
            "trait A<U>where U:Copy{}fn f<T:A<U>,U>(x:T){}fn main(){}",
            false,
        ),
        (
            "trait-argument-user-missing",
            "trait B{}trait A<U:B>{}fn f<T:A<U>,U>(x:T){}fn main(){}",
            false,
        ),
        (
            "trait-argument-user-given",
            "trait B{}trait A<U:B>{}fn f<T:A<U>,U:B>(x:T){}fn main(){}",
            true,
        ),
        (
            "trait-argument-illformed",
            "trait A<U:?Sized>{}fn f<T:A<[str;0]>>(x:T){}fn main(){}",
            false,
        ),
        (
            "trait-argument-primitive-copy",
            "trait A<U:Copy>{}fn f<T:A<u8>>(x:T){}fn main(){}",
            true,
        ),
        (
            "trait-argument-forward-bound",
            "trait A<T:B<u8>>{}trait B<U>{}fn main(){}",
            true,
        ),
        (
            "trait-argument-default-self",
            "trait A<U=Self>{}fn f<T:A>(x:T){}fn main(){}",
            true,
        ),
        (
            "trait-argument-default-self-unsized",
            "trait A<U=Self>{}fn f<T:?Sized+A>(x:&T){}fn main(){}",
            false,
        ),
        (
            "trait-argument-default-self-relaxed",
            "trait A<U:?Sized=Self>{}fn f<T:?Sized+A>(x:&T){}fn main(){}",
            true,
        ),
        (
            "trait-argument-concrete-subject",
            "trait A<U>{}fn f<T>()where u8:A<T>{}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn supertraits_supply_only_their_declared_permissions() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "trait-super-copy",
            "trait A:Copy{}fn f<T:A>(x:T){let _a=x;let _b=x;}fn main(){}",
            true,
        ),
        (
            "trait-super-transitive",
            "trait A:B{}trait B:C{}trait C:Copy{}fn f<T:A>(x:T){let _a=x;let _b=x;}fn main(){}",
            true,
        ),
        (
            "trait-super-where",
            "trait A where Self:B{}trait B:Copy{}fn f<T:A>(x:T){let _a=x;let _b=x;}fn main(){}",
            true,
        ),
        (
            "trait-super-diamond",
            "trait A:B+C{}trait B:D{}trait C:D{}trait D:Copy{}fn f<T:A>(x:T){let _a=x;let _b=x;}fn main(){}",
            true,
        ),
        (
            "trait-super-clone-not-copy",
            "trait A:Clone{}fn f<T:A>(x:T){let _a=x;let _b=x;}fn main(){}",
            false,
        ),
        (
            "trait-shadow-copy",
            "trait Copy{}fn f<T:Copy>(x:T){let _a=x;let _b=x;}fn main(){}",
            false,
        ),
        (
            "trait-shadow-clone",
            "trait Clone{}trait A:Clone{}fn f<T:A>(x:T){let _a=x;let _b=x;}fn main(){}",
            false,
        ),
        (
            "trait-super-forward-arguments",
            "trait A:B<u8>{}trait B<U>{}fn g<T:B<u8>>(x:T){}fn f<T:A>(x:T){g(x)}fn main(){}",
            true,
        ),
        (
            "trait-super-forward-const",
            "trait A<const N:usize>:B<N>{}trait B<const M:usize>{}fn g<const M:usize,T:B<M>>(x:T){}fn f<const N:usize,T:A<N>>(x:T){g::<N,T>(x)}fn main(){}",
            true,
        ),
        (
            "trait-super-cycle",
            "trait A:B{}trait B:A{}fn main(){}",
            false,
        ),
        (
            "trait-super-argument-cycle",
            "trait A<T>:A<(T,)>{}fn main(){}",
            false,
        ),
        (
            "trait-super-default-body",
            "trait A:B{fn pair(self)->(Self,Self){(self,self)}}trait B:Copy{}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn unused_generic_bodies_prove_complete_call_predicates() -> Result<(), Box<dyn std::error::Error>>
{
    cases(&[
        (
            "trait-call-exact",
            "trait A{}fn g<T:A>(x:T){}fn f<T:A>(x:T){g(x)}fn main(){}",
            true,
        ),
        (
            "trait-call-missing",
            "trait A{}fn g<T:A>(x:T){}fn f<T>(x:T){g(x)}fn main(){}",
            false,
        ),
        (
            "trait-call-wrong-definition",
            "trait A{}trait B{}fn g<T:A>(x:T){}fn f<T:B>(x:T){g(x)}fn main(){}",
            false,
        ),
        (
            "trait-call-type-argument",
            "trait A<U>{}fn g<T:A<U>,U>(x:T,y:U){}fn f<T:A<u8>>(x:T){g(x,7u8)}fn main(){}",
            true,
        ),
        (
            "trait-call-wrong-type",
            "trait A<U>{}fn g<T:A<U>,U>(x:T,y:U){}fn f<T:A<u16>>(x:T){g(x,7u8)}fn main(){}",
            false,
        ),
        (
            "trait-call-const-argument",
            "trait A<const N:usize>{}fn g<const N:usize,T:A<N>>(x:T){}fn f<const N:usize,T:A<N>>(x:T){g::<N,T>(x)}fn main(){}",
            true,
        ),
        (
            "trait-call-wrong-const",
            "trait A<const N:usize>{}fn g<const N:usize,T:A<N>>(x:T){}fn f<T:A<2>>(x:T){g::<3,T>(x)}fn main(){}",
            false,
        ),
        (
            "trait-call-lifetime-type",
            "trait A<U>{}fn g<'a,T:A<&'a u8>>(x:T){}fn f<'a,T:A<&'a u8>>(x:T){g::<'a,T>(x)}fn main(){}",
            true,
        ),
        (
            "trait-call-lifetime-argument",
            "trait A<'a>{}fn g<'a,T:A<'a>>(x:T){}fn f<'a,T:A<'a>>(x:T){g::<'a,T>(x)}fn main(){}",
            true,
        ),
        (
            "trait-call-bound-value",
            "trait A{}fn g<T:A>(x:T){}fn f<T:A>(){let _p=g::<T>;}fn main(){}",
            true,
        ),
        (
            "trait-call-missing-value",
            "trait A{}fn g<T:A>(x:T){}fn f<T>(){let _p=g::<T>;}fn main(){}",
            false,
        ),
        (
            "trait-call-nominal",
            "trait A{}struct S<T:A>(T);fn g<T:A>(x:T)->S<T>{S(x)}fn main(){}",
            true,
        ),
        (
            "trait-call-nominal-missing",
            "trait A{}struct S<T:A>(T);fn g<T>(x:T)->S<T>{S(x)}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn declaration_lifetimes_and_closed_defaults_are_proved() -> Result<(), Box<dyn std::error::Error>>
{
    cases(&[
        (
            "trait-const-default-bound",
            "trait X<const N:usize>{}trait A<const N:usize=2>where u8:X<N>{}fn main(){}",
            false,
        ),
        (
            "trait-const-nondefault-bound",
            "trait X<const N:usize>{}trait A<const N:usize>where u8:X<N>{}fn main(){}",
            true,
        ),
        (
            "trait-nonself-composite-missing",
            "trait A where (Self,):Copy{}fn f<T:A>(x:T){let a=(x,);let _b=a;let _c=a;}fn main(){}",
            false,
        ),
        (
            "trait-nonself-composite-given",
            "trait A where (Self,):Copy{}fn f<T:A>(x:T)where (T,):Copy{let a=(x,);let _b=a;let _c=a;}fn main(){}",
            true,
        ),
        (
            "trait-recursive-parameter",
            "trait A<T:A<T>>{}fn main(){}",
            true,
        ),
        (
            "trait-recursive-argument",
            "trait A:B<Self>{}trait B<T:?Sized+A>{}fn main(){}",
            true,
        ),
        (
            "trait-super-outlives",
            "trait A:'static{}fn g<T:'static>(x:T){}fn f<T:A>(x:T){g(x)}fn main(){}",
            true,
        ),
        (
            "trait-argument-outlives-given",
            "trait A<'a,U:'a>{}fn f<'a,T:A<'a,U>,U:'a>(x:T){}fn main(){}",
            true,
        ),
        (
            "trait-argument-outlives-missing",
            "trait A<'a,U:'a>{}fn f<'a,T:A<'a,U>,U>(x:T){}fn main(){}",
            false,
        ),
        (
            "trait-bound-reference-missing",
            "trait A<U>{}fn f<'a,T:A<&'a U>,U>(x:T){}fn main(){}",
            false,
        ),
        (
            "trait-bound-reference-given",
            "trait A<U>{}fn f<'a,T:A<&'a U>,U:'a>(x:T){}fn main(){}",
            true,
        ),
        (
            "trait-bound-reference-signature",
            "trait A<U>{}fn f<'a,T:A<&'a U>,U>(x:T,y:&'a U){}fn main(){}",
            true,
        ),
        (
            "trait-bound-region-missing",
            "trait A<'a:'b,'b>{}fn f<'a,'b,T:A<'a,'b>>(x:T){}fn main(){}",
            false,
        ),
        (
            "trait-bound-region-given",
            "trait A<'a:'b,'b>{}fn f<'a:'b,'b,T:A<'a,'b>>(x:T){}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn diamond_supertraits_retain_each_assumption_once() -> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write as _;
    let mut source = String::from("trait Base:Copy{}");
    let mut previous = String::from("Base");
    for index in 0..24 {
        write!(
            source,
            "trait L{index}:{previous}{{}}trait R{index}:{previous}{{}}trait J{index}:L{index}+R{index}{{}}"
        )?;
        previous = format!("J{index}");
    }
    write!(
        source,
        "fn f<T:{previous}>(x:T){{let _a=x;let _b=x;}}fn main(){{}}"
    )?;
    rustc::qualify("trait-diamond-sharing", &source, true)?;
    let (report, facts) = semantic::analyze(&source)?;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "f"
        })
        .ok_or("missing bounded function")?;
    let environment = facts
        .generics
        .environments
        .get(&function.definition)
        .ok_or("missing bounded environment")?;
    assert_eq!(environment.trait_predicates().count(), 73);
    prove(&source)
}

#[test]
fn trait_headers_require_explicit_outer_lifetimes() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "trait-default-elided-reference",
            "trait A<U=&u8>{}fn main(){}",
            false,
        ),
        (
            "trait-default-placeholder-reference",
            "trait A<U=&'_ u8>{}fn main(){}",
            false,
        ),
        (
            "trait-default-named-reference",
            "trait A<'a,U=&'a u8>{}fn main(){}",
            true,
        ),
        (
            "trait-default-callable-elision",
            "trait A<U=fn(&u8)>{}fn main(){}",
            true,
        ),
        (
            "trait-super-elided-argument",
            "trait A<'a>{}trait B:A{}fn main(){}",
            false,
        ),
        (
            "trait-super-placeholder-argument",
            "trait A<'a>{}trait B:A<'_>{}fn main(){}",
            false,
        ),
        (
            "trait-super-explicit-argument",
            "trait A<'a>{}trait B<'b>:A<'b>{}fn main(){}",
            true,
        ),
    ])
}
