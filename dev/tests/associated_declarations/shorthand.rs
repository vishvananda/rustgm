//! Shorthand names retain complete trait inputs and use ordinary scoped projection proof.
use super::{native, object_outcome, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "trait-constant",
        "trait A{type Out;const C:Self::Out;}impl A for u8{type Out=usize;const C:usize=3;}fn main(){check(<u8 as A>::C==3);}",
        true,
    ),
    (
        "trait-method",
        "trait A{type Out;fn keep(x:Self::Out)->Self::Out{x}}impl A for u8{type Out=u16;}fn main(){check(<u8 as A>::keep(7)==7);}",
        true,
    ),
    (
        "impl-method",
        "trait A{type Out;fn value()-><Self as A>::Out;}impl A for u8{type Out=usize;fn value()->Self::Out{3}}fn main(){check(<u8 as A>::value()==3);}",
        true,
    ),
    (
        "generic-bound",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn keep<T:A>(x:T::Out)->T::Out{x}fn main(){check(keep::<u8>(7)==7);}",
        true,
    ),
    (
        "inherited-member",
        "trait B{type Out;}trait A:B{fn keep(x:Self::Out)->Self::Out{x}}impl B for u8{type Out=u16;}impl A for u8{}fn main(){check(<u8 as A>::keep(7)==7);}",
        true,
    ),
    (
        "mixed-trait-arguments",
        "trait A<T,const N:usize>{type Out;fn keep(x:Self::Out)->Self::Out{x}}impl<T,const N:usize>A<T,N>for u8{type Out=[T;N];}fn main(){let x=<u8 as A<u8,3>>::keep([7;3]);check(x[2]==7);}",
        true,
    ),
    (
        "captured-lifetime",
        "trait A<'a>{type Out;fn keep(x:Self::Out)->Self::Out{x}}impl<'a>A<'a>for u8{type Out=&'a u8;}fn main(){let x=7u8;check(*<u8 as A<'_>>::keep(&x)==7);}",
        true,
    ),
    (
        "inherited-arguments",
        "trait B<T>{type Out;}trait A:B<u16>{fn keep(x:Self::Out)->Self::Out{x}}impl B<u16>for u8{type Out=u16;}impl A for u8{}fn main(){check(<u8 as A>::keep(7)==7);}",
        true,
    ),
    (
        "own-supertrait-ambiguity",
        "trait B{type Out;}trait A:B{type Out;fn keep(x:Self::Out)->Self::Out{x}}fn main(){}",
        false,
    ),
    (
        "generic-ambiguity",
        "trait A{type Out;}trait B{type Out;}fn keep<T:A+B>(x:T::Out)->T::Out{x}fn main(){}",
        false,
    ),
    (
        "bare-trait-outside",
        "trait A{type Out;}type X=A::Out;fn main(){}",
        false,
    ),
    (
        "bare-trait-inside",
        "trait A{type Out;fn keep(x:A::Out)->A::Out{x}}fn main(){}",
        false,
    ),
    (
        "member-arguments",
        "trait A{type Out;fn keep(x:Self::Out<u8>);}fn main(){}",
        false,
    ),
    (
        "missing-bound",
        "trait A{type Out;}fn keep<T>(x:T::Out)->T::Out{x}fn main(){}",
        false,
    ),
    (
        "impl-supertrait",
        "trait B{type Out;}trait A:B{type Out;fn f()-><Self as A>::Out;}struct S;impl B for S{type Out=u8;}impl A for S{type Out=u16;fn f()->Self::Out{7}}fn main(){}",
        false,
    ),
    (
        "impl-other-bound",
        "trait B{type Out;}trait A{type Out;fn f()-><Self as A>::Out;}struct S;impl B for S{type Out=u8;}impl A for S where S:B{type Out=u16;fn f()->Self::Out{7}}fn main(){}",
        true,
    ),
    (
        "inherent-self-bound",
        "trait A{type Out;}struct S;impl A for S{type Out=u16;}impl S where S:A{fn f()->Self::Out{7}}fn main(){}",
        false,
    ),
    (
        "inherent-self-unbounded",
        "trait A{type Out;}struct S;impl A for S{type Out=u16;}impl S{fn f()->Self::Out{7}}fn main(){}",
        false,
    ),
    (
        "distinct-applications",
        "trait A<U>{type Out;}fn f<T:A<u8>+A<u16>>(x:T::Out)->T::Out{x}fn main(){}",
        false,
    ),
    (
        "distinct-lifetimes",
        "trait A<'a>{type Out;}fn f<'a,'b,T:A<'a>+A<'b>>(x:T::Out)->T::Out{x}fn main(){}",
        false,
    ),
    (
        "redundant-bound",
        "trait A{type Out;}fn f<T:A+A>(x:T::Out)->T::Out{x}fn main(){}",
        true,
    ),
    (
        "alias-use",
        "trait A{type Out;}impl A for u8{type Out=u16;}type X<T:A>=T::Out;fn main(){let _:X<u8>=7;}",
        true,
    ),
    (
        "alias-invalid-use",
        "trait A{type Out;}impl A for u8{type Out=u16;}type X<T:A>=T::Out;fn main(){let _:X<u16>=7;}",
        false,
    ),
    (
        "qualified-generic",
        "trait A{type Out;}fn f<T:A>(x:<T>::Out)-><T>::Out{x}fn main(){}",
        true,
    ),
    (
        "qualified-concrete",
        "trait A{type Out;}impl A for u8{type Out=u16;}type X=<u8>::Out;fn main(){}",
        false,
    ),
    (
        "where-subject",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn twice<T>(x:T::Out)->(T::Out,T::Out)where T:A,T::Out:Copy{(x,x)}fn main(){let x=twice::<u8>(7);check(x.0==7);check(x.1==7);}",
        true,
    ),
    (
        "where-subject-first",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn twice<T>(x:T::Out)->(T::Out,T::Out)where T::Out:Copy,T:A{(x,x)}fn main(){let x=twice::<u8>(7);check(x.1==7);}",
        true,
    ),
    (
        "bound-argument",
        "trait A{type Out;}trait B<U>{type Out;}impl A for u8{type Out=u16;}impl B<u16> for u32{type Out=u64;}fn keep<T,U>(x:U::Out)->U::Out where U:B<T::Out>,T:A{x}fn main(){check(keep::<u8,u32>(7)==7);}",
        true,
    ),
    (
        "qualified-self",
        "trait A{type Out;fn keep(x:<Self>::Out)-><Self>::Out{x}}impl A for u8{type Out=u16;}fn main(){check(<u8 as A>::keep(7)==7);}",
        true,
    ),
    (
        "renamed-import",
        "mod m{pub trait A{type Out;}impl A for u8{type Out=u16;}}use m::A as B;fn keep<T:B>(x:T::Out)->T::Out{x}fn main(){check(keep::<u8>(7)==7);}",
        true,
    ),
    (
        "alias-callable",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}type F<'a,T:A<'a>>=fn(T::Out)->T::Out;fn keep<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=7u8;let f:F<'_,u8>=keep;check(*f(&x)==7);}",
        true,
    ),
    (
        "generic-default",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct Wrap<T:A,U=T::Out>(T,U);fn main(){let x:Wrap<u8>=Wrap(0,7);check(x.1==7);}",
        false,
    ),
    (
        "drop-result",
        "struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=D<'a>;}fn consume<'a,T:A<'a>>(x:T::Out){}fn main(){let mut x=0u8;consume::<u8>(D(&mut x));check(x==1);}",
        true,
    ),
    (
        "duplicate-move",
        "trait A{type Out;}fn twice<T:A>(x:T::Out)->(T::Out,T::Out){(x,x)}fn main(){}",
        false,
    ),
    (
        "alias-unproved-bound",
        "trait A{type Out;}impl A for u8{type Out=u16;}type X<T:A>=T::Out;fn keep<T>(x:X<T>)->X<T>{x}fn main(){}",
        false,
    ),
    (
        "impl-parent-bound",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct S<T>(T);impl<T:A>S<T>{fn keep(x:T::Out)->T::Out{x}}fn main(){check(S::<u8>::keep(7)==7);}",
        true,
    ),
    (
        "method-extra-bound",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct S<T>(T);impl<T>S<T>{fn keep(x:T::Out)->T::Out where T:A{x}}fn main(){check(S::<u8>::keep(7)==7);}",
        true,
    ),
    (
        "impl-captured-bound",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}struct S<'a,T>(&'a T);impl<'a,T:A<'a>>S<'a,T>{fn keep(x:T::Out)->T::Out{x}}fn main(){let x=7u8;check(*S::<u8>::keep(&x)==7);}",
        true,
    ),
    (
        "diamond-supertrait",
        "trait A{type Out;}trait B:A{}trait C:A{}trait D:B+C{fn keep(x:Self::Out)->Self::Out{x}}impl A for u8{type Out=u16;}impl B for u8{}impl C for u8{}impl D for u8{}fn main(){check(<u8 as D>::keep(7)==7);}",
        true,
    ),
    (
        "trait-method-extra",
        "trait A{fn f(x:Self::Out)where Self:B{}}trait B{type Out;}fn main(){}",
        true,
    ),
    (
        "trait-method-conflict",
        "trait A{type Out;fn f(x:Self::Out)where Self:B{}}trait B{type Out;}fn main(){}",
        false,
    ),
    (
        "impl-method-extra",
        "trait A{fn f(x:<Self as B>::Out)where Self:B;}trait B{type Out;}struct S;impl B for S{type Out=u8;}impl A for S{fn f(x:Self::Out)where Self:B{}}fn main(){}",
        false,
    ),
    (
        "impl-method-conflict",
        "trait A{type Out;fn f(x:<Self as A>::Out)where Self:B;}trait B{type Out;}struct S;impl B for S{type Out=u8;}impl A for S{type Out=u16;fn f(x:Self::Out)where Self:B{}}fn main(){}",
        true,
    ),
    (
        "generic-default-qualified",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct Wrap<T:A,U=<T>::Out>(T,U);fn main(){let _:Wrap<u8>=Wrap(0,7);}",
        false,
    ),
    (
        "alias-default",
        "trait A{type Out;}type Pair<T:A,U=T::Out>=(T,U);fn main(){}",
        false,
    ),
    (
        "where-equality",
        "trait A{type Out;}trait B<U>{}fn f<T:A<Out=T::Out>>(){}fn main(){}",
        false,
    ),
    (
        "self-extra-native",
        "trait A{fn f(x:Self::Out)->Self::Out where Self:B{x}}trait B{type Out;}impl B for u8{type Out=u16;}impl A for u8{}fn main(){let _:u16=<u8 as A>::f(7);}",
        true,
    ),
];

#[test]
fn shorthand_associated_types_match_independent_outcomes() -> Result<(), Box<dyn std::error::Error>>
{
    let mut failures = Vec::new();
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("shorthand-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn shorthand_associated_types_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("shorthand-associated-types", &cases)
}
