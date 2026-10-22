//! Opaque identities remain private through typing, native storage, dispatch and cleanup.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "precise-repeat",
        "fn f<'a>(x:&'a u8)->impl Copy+use<'a,'a>{7u8}fn main(){}",
        false,
    ),
    (
        "precise-order",
        "fn f<'a,T:Copy>(x:T,y:&'a u8)->impl Copy+use<T,'a>{x}fn main(){}",
        false,
    ),
    (
        "precise-bound",
        "fn f<'a>(x:&'a u8)->impl Copy+'a+use<>{7u8}fn main(){}",
        false,
    ),
    (
        "precise-builtin",
        "fn f()->impl Copy+use<u8>{7u8}fn main(){}",
        false,
    ),
    (
        "precise-static",
        "fn f()->impl Copy+use<'static>{7u8}fn main(){}",
        false,
    ),
    (
        "precise-anonymous",
        "fn f(x:impl Copy)->impl Copy+use<>{x}fn main(){}",
        false,
    ),
    (
        "precise-missing-const",
        "fn f<const N:usize>()->impl Copy+use<>{7u8}fn main(){}",
        false,
    ),
    (
        "precise-missing-type",
        "fn f<T>(x:T)->impl Copy+use<>{7u8}fn main(){}",
        false,
    ),
    (
        "user-trait-generic",
        "trait Echo<T>{fn value(&self)->T;}impl Echo<u8> for u8{fn value(&self)->u8{*self}}fn f<T:Copy+Echo<T>>(x:T)->impl Echo<T>{x}fn main(){let _=f(7u8).value();}",
        true,
    ),
    (
        "unpublished-trait",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn f()->impl Copy{7u8}fn main(){let _=f().value();}",
        false,
    ),
    (
        "capture-const",
        "fn f<const N:usize>()->impl Copy{[7u8;N]}fn main(){let _=f::<3>();let _=f::<4>();}",
        true,
    ),
    (
        "capture-const-identity",
        "fn f<const N:usize>()->impl Copy{7u8}fn same<T>(_:T,_:T){}fn main(){same(f::<3>(),f::<4>());}",
        false,
    ),
    (
        "hidden-equality",
        "fn f()->impl Copy{7u8}fn main(){let a=f();let _=a==a;}",
        false,
    ),
    (
        "user-trait",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn f()->impl Value{7u8}fn main(){let _=f().value();}",
        true,
    ),
    (
        "user-trait-wrong",
        "trait Value{fn value(&self)->u8;}fn f()->impl Value{7u8}fn main(){}",
        false,
    ),
    (
        "copy",
        "fn f()->impl Copy{7u8}fn main(){let x=f();let _y=x;let _z=x;}",
        true,
    ),
    (
        "same-identity",
        "fn f()->impl Copy{7u8}fn same<T>(_:T,_:T){}fn main(){same(f(),f());}",
        true,
    ),
    (
        "distinct-identities",
        "fn f()->impl Copy{7u8}fn g()->impl Copy{7u8}fn same<T>(_:T,_:T){}fn main(){same(f(),g());}",
        false,
    ),
    (
        "hidden-not-public",
        "fn f()->impl Copy{7u8}fn main(){let _:u8=f();}",
        false,
    ),
    (
        "wrong-bound",
        "struct S;fn f()->impl Copy{S}fn main(){}",
        false,
    ),
    (
        "branch-mismatch",
        "fn f(x:bool)->impl Copy{if x{7u8}else{false}}fn main(){}",
        false,
    ),
    (
        "branch-consistent",
        "fn f(x:bool)->impl Copy{if x{7u8}else{8u8}}fn main(){let _=f(true);}",
        true,
    ),
    (
        "early-return",
        "fn f(x:bool)->impl Copy{if x{return 7u8;}8u8}fn main(){let _=f(true);}",
        true,
    ),
    (
        "early-return-mismatch",
        "fn f(x:bool)->impl Copy{if x{return 7u8;}false}fn main(){}",
        false,
    ),
    (
        "nested-results",
        "fn f()->(impl Copy,impl Copy){(7u8,false)}fn main(){let (a,b)=f();let _=(a,b,a,b);}",
        true,
    ),
    (
        "capture-type",
        "fn f<T:Copy>(x:T)->impl Copy{x}fn main(){let _=f(7u8);let _=f(false);}",
        true,
    ),
    (
        "capture-type-identity",
        "fn f<T>(_:T)->impl Copy{7u8}fn same<T>(_:T,_:T){}fn main(){same(f(7u8),f(false));}",
        false,
    ),
];

#[test]
fn identities_and_hidden_results_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(name, source, accepted)?;
        let result = semantic::analyze(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}

const CLOSED: &[(&str, &str)] = &[
    (
        "precise-empty",
        "fn make<'a>(x:&'a u8)->impl Copy+use<>{7u8}fn take(x:impl Copy){}fn main(){let x;{let v=7;x=make(&v);}take(x);}",
    ),
    (
        "precise-type",
        "fn make<'a,T:Copy>(x:T,y:&'a u8)->impl Copy+use<T>{x}fn take(x:impl Copy){}fn main(){let x;{let v=7;x=make(11u8,&v);}take(x);}",
    ),
    (
        "precise-const",
        "trait Len{fn len(&self)->usize;}impl<const N:usize>Len for [u8;N]{fn len(&self)->usize{N}}fn make<const N:usize>()->impl Len+use<N>{[7u8;N]}fn main(){check(make::<3>().len()==3);}",
    ),
    (
        "precise-elided",
        "fn make(x:&u8)->impl Copy+use<'_>{7u8}fn main(){let v=7;let _=make(&v);}",
    ),
    (
        "value",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn make()->impl Value{7u8}fn main(){check(make().value()==7);}",
    ),
    (
        "same-identity",
        "fn make()->impl Copy{7u8}fn same<T>(x:T,y:T){}fn main(){same(make(),make());}",
    ),
    (
        "branch-consistent",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn make(b:bool)->impl Value{if b{7u8}else{11u8}}fn main(){check(make(false).value()==11);}",
    ),
    (
        "capture-type",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn make<T:Value>(x:T)->impl Value{x}fn main(){check(make(7u8).value()==7);}",
    ),
    (
        "nested-results",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn make()->(impl Value,impl Copy){(7u8,true)}fn main(){let x=make();check(x.0.value()==7);}",
    ),
    (
        "inherent-result",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}struct S;impl S{fn make(&self)->impl Value{7u8}}fn main(){check(S.make().value()==7);}",
    ),
    (
        "trait-default-result",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}trait Make{fn make(&self)->impl Value{7u8}}struct S;impl Make for S{}fn main(){check(S.make().value()==7);}",
    ),
    (
        "const-capture",
        "trait Len{fn len(&self)->usize;}impl<const N:usize>Len for [u8;N]{fn len(&self)->usize{N}}fn make<const N:usize>()->impl Len{[7u8;N]}fn main(){check(make::<3>().len()==3);}",
    ),
    (
        "self-item",
        "fn f()->impl Copy{f}fn main(){let a=f();let _=(a,a);}",
    ),
    (
        "mutual-items",
        "fn f()->impl Copy{g}fn g()->impl Copy{f}fn main(){let _=(f(),g());}",
    ),
    (
        "opaque-chain",
        "fn first()->impl Copy{7u8}fn second()->impl Copy{first()}fn main(){let a=second();let _=(a,a);}",
    ),
];

#[test]
fn closed_results_lower_with_private_storage() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source) in CLOSED.iter().chain(DROPS) {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(&format!("closed-results-{name}"), &source, true)?;
        let (mut report, mut facts) = semantic::analyze(&source)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        let result = rustgm::lower::instances::build(&report, &mut facts, entry);
        assert!(result.is_ok(), "{name}: {:?}", result.err());
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn closed_results_execute_like_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CLOSED
        .iter()
        .map(|&(name, source)| (name, source, false))
        .collect::<Vec<_>>();
    let mut cases = cases;
    cases.extend(DROPS.iter().map(|&(name, source)| (name, source, true)));
    native::check_cases("opaque-results/native", &cases)
}

#[test]
fn unused_recursive_hidden_storage_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        ("recursive-tuple", "fn f()->impl Copy{(f(),)}fn main(){}"),
        (
            "recursive-generic",
            "fn f<T:Copy>(x:T)->impl Copy{(x,f(x))}fn main(){}",
        ),
    ] {
        rustc::qualify(name, source, false)?;
        let (mut report, mut facts) = semantic::analyze(source)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        let result = rustgm::lower::instances::build(&report, &mut facts, entry);
        assert!(
            result.is_err(),
            "{name}: infinite unused hidden representation accepted"
        );
    }
    Ok(())
}

// The destructor abort is observable: omitting opaque cleanup must fail these controls.
const DROPS: &[(&str, &str)] = &[
    (
        "drop-value",
        "trait V{}struct S;impl V for S{}impl Drop for S{fn drop(&mut self){check(false);}}fn make()->impl V{S}fn main(){let _x=make();}",
    ),
    (
        "drop-generic",
        "trait V{}struct S;impl V for S{}impl Drop for S{fn drop(&mut self){check(false);}}fn make()->impl V{S}fn consume<T>(_:T){}fn main(){consume(make());}",
    ),
    (
        "drop-field",
        "trait V{}struct S;impl V for S{}impl Drop for S{fn drop(&mut self){check(false);}}struct W<T>(T);fn make()->impl V{S}fn main(){let _x=W(make());}",
    ),
];
