//! Nominal bound snapshots normalize before publication and retain declaration variance.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
const CASES: &[(&str, &str, bool, bool)] = &[
    (
        "explicit-static",
        "trait A{type Out:Copy;} struct S<T:A>where<T as A>::Out:'static{value:&'static<T as A>::Out}fn main(){}\n",
        true,
        false,
    ),
    (
        "explicit-region",
        "trait A{type Out:Copy;} struct S<'a,T:A>where<T as A>::Out:'a{value:&'a<T as A>::Out}fn main(){}\n",
        true,
        false,
    ),
    (
        "explicit-copy",
        "trait A{type Out;}struct S<T:A>where<T as A>::Out:Copy{value:<T as A>::Out}fn main(){}\n",
        true,
        false,
    ),
    (
        "explicit-user-trait",
        "trait A{type Out:Copy;} trait B{}struct S<T:A>where<T as A>::Out:B{value:<T as A>::Out}fn main(){}\n",
        true,
        false,
    ),
    (
        "explicit-enum-region",
        "trait A{type Out:Copy;} enum E<'a,T:A>where<T as A>::Out:'a{Some(&'a<T as A>::Out),None}fn main(){}\n",
        true,
        false,
    ),
    (
        "missing-parent-trait",
        "trait A{type Out:Copy;} struct S<T>where<T as A>::Out:'static{value:&'static<T as A>::Out}fn main(){}\n",
        false,
        false,
    ),
    (
        "unbounded-static",
        "trait A{type Out:Copy;} struct S<T:A>{value:&'static<T as A>::Out}fn main(){}\n",
        false,
        false,
    ),
    (
        "result-not-self",
        "trait A{type Out:Copy;} struct S<T:A>where<T as A>::Out:'static{value:&'static<T as A>::Out}fn need<T:'static>(){}fn f<T:A>(_:S<T>)where<T as A>::Out:'static{need::<T>();}fn main(){}\n",
        false,
        false,
    ),
    (
        "wrong-member",
        "trait A{type Out:Copy;type Other:Copy;}struct S<T:A>where<T as A>::Other:'static{value:&'static<T as A>::Out}fn main(){}\n",
        false,
        false,
    ),
    (
        "nested-explicit",
        "trait A{type Out:Copy;} struct S<'a,T:A>where<T as A>::Out:'a{value:&'a<T as A>::Out}struct W<'a,T:A>where<T as A>::Out:'a{inner:S<'a,T>}fn get<'a,T:A>(x:W<'a,T>)->&'a<T as A>::Out{x.inner.value}fn main(){}\n",
        true,
        false,
    ),
    (
        "explicit-equality",
        "trait A{type Out:Copy;} struct S<T:A<Out=u16>>{value:<T as A>::Out}fn main(){}\n",
        true,
        false,
    ),
    (
        "equality-record-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl A for u8{type Out=u16;}struct S<T:A<Out=u16>>{v:<T as A>::Out}fn main(){let s=S::<u8>{v:73};check(s.v==73);}\n",
        true,
        true,
    ),
    (
        "equality-enum-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl A for u8{type Out=u16;}enum E<T:A<Out=u16>>{V(<T as A>::Out),N}fn main(){let e=E::<u8>::V(73);match e{E::V(v)=>check(v==73),E::N=>check(false)}}\n",
        true,
        true,
    ),
    (
        "equality-alias-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl A for u8{type Out=u16;}type Out<T>=<T as A>::Out;struct S<T:A<Out=u16>>{v:Out<T>}fn main(){let s=S::<u8>{v:73};check(s.v==73);}\n",
        true,
        true,
    ),
    (
        "equality-callable-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl A for u8{type Out=u16;}struct S<T:A<Out=u16>>{v:fn(<T as A>::Out)}fn f(x:u16){check(x==73);}fn main(){let s=S::<u8>{v:f};(s.v)(73);}\n",
        true,
        true,
    ),
    (
        "equality-mixed-fields-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl A for u8{type Out=u16;}struct S<T:A<Out=u16>>{a:u8,b:u32,v:<T as A>::Out,c:u8}fn main(){let s=S::<u8>{a:1,b:2,v:73,c:3};check(s.a==1);check(s.b==2);check(s.v==73);check(s.c==3);}\n",
        true,
        true,
    ),
    (
        "normalized-projection-invariant",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl<'a>A for &'a u8{type Out=u16;}struct S<T:A<Out=u16>>{v:<T as A>::Out}fn shrink<'a>(x:S<&'static u8>,_:&'a u8)->S<&'a u8>{x}fn main(){}\n",
        false,
        false,
    ),
    (
        "ordinary-covariant",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct S<T>{v:T}fn shrink<'a>(x:S<&'static u8>,_:&'a u8)->S<&'a u8>{x}fn main(){}\n",
        true,
        false,
    ),
    (
        "where-only-unused",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl A for u8{type Out=u16;}struct S<T:A<Out=u16>>{v:u16}fn main(){}\n",
        false,
        false,
    ),
    (
        "unused-lifetime",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl A for u8{type Out=u16;}struct S<'a,T:A<Out=u16>>where<T as A>::Out:'a{v:<T as A>::Out}fn main(){}\n",
        false,
        false,
    ),
    (
        "default-static-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl A for u8{type Out=u16;}struct S<T:A=u8>where<T as A>::Out:'static{v:&'static<T as A>::Out}static X:u16=73;fn main(){let s:S=S{v:&X};check(*s.v==73);}\n",
        true,
        true,
    ),
    (
        "explicit-enum-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl A for u8{type Out=u16;}enum E<'a,T:A>where<T as A>::Out:'a{V(&'a<T as A>::Out),N}fn main(){let x=73;let e=E::<u8>::V(&x);match e{E::V(v)=>check(*v==73),E::N=>check(false)}}\n",
        true,
        true,
    ),
    (
        "trait-argument-before",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}struct S<T,U>where U:B<<T as A>::Out>,T:Copy{t:T,u:U}fn main(){}\n",
        true,
        false,
    ),
    (
        "trait-argument-after",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}struct S<T,U>where T:Copy,U:B<<T as A>::Out>{t:T,u:U}fn main(){}\n",
        true,
        false,
    ),
    (
        "trait-argument-missing-parent",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}struct S<T,U>where U:B<<T as A>::Out>{t:T,u:U}fn main(){}\n",
        false,
        false,
    ),
    (
        "trait-argument-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}impl B<u8>for u16{}struct S<T:Copy,U:B<<T as A>::Out>>{t:T,u:U}fn main(){let s=S{t:1u8,u:73u16};check(s.u==73);}\n",
        true,
        true,
    ),
    (
        "normalized-bound-unused",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}struct S<T:Copy>where u8:B<<T as A>::Out>{t:T}fn main(){}\n",
        true,
        false,
    ),
    (
        "normalized-bound-used",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}struct S<T:Copy>where u8:B<<T as A>::Out>{t:T}fn main(){let _=S{t:1u8};}\n",
        false,
        false,
    ),
    (
        "closed-written-bound",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}struct S<T:Copy>where u8:B<<u8 as A>::Out>{t:T}fn main(){}\n",
        false,
        false,
    ),
    (
        "alias-bound-subject",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}struct W<T>(T);type Alias<T>=W<T>;trait C{}struct S<T:Copy>where Alias<<T as A>::Out>:C{t:T}fn main(){}\n",
        true,
        false,
    ),
    (
        "mixed-bound-arguments",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<'a,U,const N:usize>{}struct S<'a,T:Copy,U:B<'a,<T as A>::Out,3>>{t:&'a T,u:U}fn main(){}\n",
        true,
        false,
    ),
    (
        "equality-short-self-static-run",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\ntrait A{type Out:Copy;}impl<'a>A for &'a u8{type Out=u16;}struct S<T:A<Out=u16>>{v:&'static<T as A>::Out}static X:u16=73;fn f<'a>(_:&'a u8)->S<&'a u8>{S{v:&X}}fn main(){let x=1;let s=f(&x);check(*s.v==73);}\n",
        true,
        true,
    ),
];
#[test]
fn aggregate_bounds_preserve_formation_selection_and_input_identity()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted, _) in CASES {
        rustc::qualify(&format!("aggregate-bound-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn aggregate_bounds_execute_with_independent_results() -> Result<(), Box<dyn std::error::Error>> {
    let mut cases = Vec::new();
    for &(name, source, _, execute) in CASES {
        if execute {
            let body = source
                .strip_prefix(native::CHECK)
                .ok_or("missing qualified native check prefix")?;
            cases.push((name, body, false));
        }
    }
    native::check_cases("aggregate-bounds", &cases)
}
