//! Self constructors preserve the enclosing impl application and source access.
use crate::{native_references as native, qualify as rustc, semantic};

const ACCEPTED: &[(&str, &str, bool)] = &[
    (
        "self-constructor-nested-concrete",
        "struct S(u8);impl S{fn make()->Self{fn inner()->S{Self(7)}inner()}}fn main(){check(S::make().0==7);}",
        false,
    ),
    (
        "self-constructor-tuple",
        "struct S(u8);impl S{fn make(x:u8)->Self{Self(x)}}fn main(){check(S::make(7).0==7);}",
        false,
    ),
    (
        "self-constructor-unit",
        "struct S;impl S{fn make()->Self{Self}}fn main(){let _=S::make();}",
        false,
    ),
    (
        "self-constructor-generic",
        "struct S<T>(T);impl<T>S<T>{fn make(x:T)->Self{Self(x)}}fn main(){check(S::make(7u8).0==7);check(S::make(517u16).0==517);}",
        false,
    ),
    (
        "self-constructor-const",
        "struct S<const N:usize>([u8;N]);impl<const N:usize>S<N>{fn make(x:[u8;N])->Self{Self(x)}}fn main(){check(S::make([3,7]).0[1]==7);}",
        false,
    ),
    (
        "self-constructor-lifetime",
        "struct S<'a>(&'a u8);impl<'a>S<'a>{fn make(x:&'a u8)->Self{Self(x)}}fn main(){let n=7;check(*S::make(&n).0==7);}",
        false,
    ),
    (
        "self-constructor-record-reordered",
        "struct S<A,B>{a:A,b:B}impl<A,B>S<B,A>{fn make(a:B,b:A)->Self{Self{a,b}}fn take(self)->B{let Self{a,b:_}=self;a}}fn main(){check(S::make(517u16,7u8).take()==517);check(S::make(7u8,517u16).take()==7);}",
        false,
    ),
    (
        "self-constructor-record-const",
        "struct S<const N:usize>{x:[u8;N]}impl<const N:usize>S<N>{fn make(x:[u8;N])->Self{Self{x}}}fn main(){check(S::make([3,7]).x[1]==7);}",
        false,
    ),
    (
        "self-constructor-record-lifetime",
        "struct S<'a,T>{x:&'a T}impl<'a,T>S<'a,T>{fn make(x:&'a T)->Self{Self{x}}}fn main(){let n=7u8;check(*S::make(&n).x==7);}",
        false,
    ),
    (
        "self-constructor-pointer",
        "struct S<T>(T);impl<T>S<T>{fn factory()->fn(T)->Self{Self}}fn main(){check(S::<u16>::factory()(517).0==517);}",
        false,
    ),
    (
        "self-constructor-trait-impl",
        "struct S(u8);trait Make{fn make(x:u8)->Self;}impl Make for S{fn make(x:u8)->Self{Self(x)}}fn main(){check(S::make(7).0==7);}",
        false,
    ),
    (
        "self-constructor-alias-impl",
        "struct S(u8);type A=S;impl A{fn make(x:u8)->Self{Self(x)}}fn main(){check(A::make(7).0==7);}",
        false,
    ),
    (
        "self-constructor-pattern",
        "struct S(u8);impl S{fn take(self)->u8{let Self(x)=self;x}}fn main(){check(S(7).take()==7);}",
        false,
    ),
    (
        "self-constructor-macro",
        "macro_rules! construct{($x:expr)=>{Self($x)}}struct S(u8);impl S{fn make(x:u8)->Self{construct!(x)}}fn main(){check(S::make(7).0==7);}",
        false,
    ),
];

const REJECTED: &[(&str, &str)] = &[
    (
        "self-constructor-unused-generic-capture",
        "struct S<T>(T);impl<T>S<T>{fn unused(){fn inner(){let _=Self;}}}fn main(){}",
    ),
    (
        "self-constructor-record-call",
        "struct S{x:u8}impl S{fn bad()->Self{Self(7)}}fn main(){}",
    ),
    (
        "self-constructor-enum-call",
        "enum E{V(u8)}impl E{fn bad()->Self{Self(7)}}fn main(){}",
    ),
    (
        "self-constructor-trait-default",
        "trait Make:Sized{fn bad()->Self{Self(7)}}fn main(){}",
    ),
    (
        "self-constructor-private-fields",
        "mod m{pub struct S(u8);}impl m::S{fn bad()->Self{Self(7)}}fn main(){}",
    ),
    (
        "self-constructor-wrong-type",
        "struct S<T>(T);impl<T>S<T>{fn bad()->Self{Self(7u8)}}fn main(){}",
    ),
    (
        "self-constructor-extra-arguments",
        "struct S<T>(T);impl<T>S<T>{fn bad(x:T)->Self{Self::<T>(x)}}fn main(){}",
    ),
    (
        "self-constructor-unit-call",
        "struct S;impl S{fn bad()->Self{Self()}}fn main(){}",
    ),
    (
        "self-constructor-ordinary-alias",
        "struct S(u8);type A=S;fn main(){let _=A(7);}",
    ),
    (
        "self-constructor-escaped-borrow",
        "struct S<'a>(&'a u8);impl<'a>S<'a>{fn bad()->Self{let n=7;Self(&n)}}fn main(){}",
    ),
    (
        "self-constructor-generic-barrier",
        "struct S<T>(T);impl<T>S<T>{fn bad(x:T)->Self{fn inner()->S<u8>{Self(7)}inner()}}fn main(){}",
    ),
    (
        "self-constructor-record-wrong-type",
        "struct S<T>{x:T}impl<T>S<T>{fn bad()->Self{Self{x:7u8}}}fn main(){}",
    ),
    (
        "self-constructor-record-generic-barrier",
        "struct S<T>{x:T}impl<T>S<T>{fn bad()->Self{fn inner()->S<u8>{Self{x:7}}inner()}}fn main(){}",
    ),
    (
        "self-constructor-record-private-fields",
        "mod m{pub struct S<T>{x:T}}impl<T>m::S<T>{fn bad(x:T)->Self{Self{x}}}fn main(){}",
    ),
    (
        "self-constructor-record-trait-default",
        "trait Make:Sized{fn bad()->Self{Self{x:7}}}fn main(){}",
    ),
    (
        "self-constructor-record-extra-arguments",
        "struct S<T>{x:T}impl<T>S<T>{fn bad(x:T)->Self{Self::<T>{x}}}fn main(){}",
    ),
    (
        "self-constructor-record-escaped-borrow",
        "struct S<'a,T>{x:&'a T}impl<'a>S<'a,u8>{fn bad()->Self{let n=7;Self{x:&n}}}fn main(){}",
    ),
];

fn compile(source: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let program = rustgm::lower::instances::build(&report, &mut facts, entry)?;
    let mut bytes = Vec::new();
    rustgm::lowir::writer::write(&program, &mut bytes)?;
    Ok(bytes)
}

#[test]
fn self_values_select_exact_impl_constructor_arguments() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, _) in ACCEPTED {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(name, &source, true)?;
        assert!(
            !compile(&source)
                .map_err(|e| format!("{name}: {e}"))?
                .is_empty()
        );
    }
    Ok(())
}

#[test]
fn self_values_preserve_shape_privacy_and_template_restrictions()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source) in REJECTED {
        rustc::qualify(name, source, false)?;
        assert!(compile(source).is_err(), "{name}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn self_constructor_values_execute_against_rustc() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases_with("self-constructors", ACCEPTED, |source, _| compile(source))
}
