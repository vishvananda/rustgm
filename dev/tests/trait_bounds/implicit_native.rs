//! Executable controls distinguish the chosen body, including bound and import precedence.
const EXECUTIONS: &[(&str, &str, bool)] = &[
    (
        "implicit-run-primitive-type-path",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let x=u8::f(3);check(x==4);}",
        false,
    ),
    (
        "implicit-run-primitive-angle-path",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let x=<u8>::f(3);check(x==4);}",
        false,
    ),
    (
        "implicit-run-nominal-type-path",
        "trait A{fn f(self)->u8;}struct S;impl A for S{fn f(self)->u8{7}}fn main(){let x=S::f(S);check(x==7);}",
        false,
    ),
    (
        "implicit-run-generic-type-path",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn g<T:A>(x:T)->u8{T::f(x)}fn main(){let x=g(3u8);check(x==4);}",
        false,
    ),
    (
        "implicit-run-generic-angle-path",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn g<T:A>(x:T)->u8{<T>::f(x)}fn main(){let x=g(3u8);check(x==4);}",
        false,
    ),
    (
        "implicit-run-self-default",
        "trait A:Sized{fn f(self)->u8;fn g(self)->u8{Self::f(self)+1}}impl A for u8{fn f(self)->u8{self}}fn main(){let x=u8::g(3);check(x==4);}",
        false,
    ),
    (
        "implicit-run-type-supertrait",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}trait B:A{}impl B for u8{}fn g<T:B>(x:T)->u8{T::f(x)}fn main(){let x=g(3u8);check(x==4);}",
        false,
    ),
    (
        "implicit-run-type-function-value",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let p=u8::f;let x=p(3);check(x==4);}",
        false,
    ),
    (
        "implicit-run-prefix-inherent-bound-fallback",
        "trait A{fn f(self)->u8;}struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->u8{1}}impl<T>A for S<T>{fn f(self)->u8{2}}struct N;fn main(){let x=S::<N>::f(S(N));check(x==2);}",
        false,
    ),
    (
        "implicit-run-prefix-rigid-inherent-bound-fallback",
        "trait A{fn f(self)->u8;}struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->u8{1}}impl<T>A for S<T>{fn f(self)->u8{2}}fn g<T>(x:S<T>)->u8{S::<T>::f(x)}struct N;fn main(){let x=g(S(N));check(x==2);}",
        false,
    ),
    (
        "implicit-run-prefix-default-trait-argument-inferred",
        "trait A<T=u8>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=u8::f(3,7u16);check(x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-trait-captured-lifetime",
        "trait A<'a>{fn f(self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(self,x:&'a u8)->&'a u8{x}}fn g<'a>(x:&'a u8)->&'a u8{u8::f(3,x)}fn main(){let n=7u8;let x=g(&n);check(*x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-generic-trait-argument",
        "trait A<T>{fn f(self,x:T)->T;}fn g<X:A<Y>,Y>(x:X,y:Y)->Y{X::f(x,y)}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=g(3u8,7u16);check(x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-method-const",
        "trait A{fn f<const N:usize>(self,x:[u8;N])->[u8;N];}impl A for u8{fn f<const N:usize>(self,x:[u8;N])->[u8;N]{x}}fn main(){let x=u8::f::<2>(3,[7,9]);check(x[0]==7&&x[1]==9);}",
        false,
    ),
    (
        "implicit-run-prefix-constant-function-value",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}const P:fn(u8)->u8=u8::f;fn main(){let x=P(3);check(x==4);}",
        false,
    ),
    (
        "implicit-run-prefix-reference-type",
        "trait A{fn f(self)->u8;}impl<'a>A for &'a u8{fn f(self)->u8{*self}}fn main(){let n=7u8;let x=<&u8>::f(&n);check(x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-array-type",
        "trait A{fn f(self)->u8;}impl<const N:usize>A for [u8;N]{fn f(self)->u8{self[0]}}fn main(){let x=<[u8;2]>::f([7,9]);check(x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-alias-hole",
        "trait A{fn f(self)->u8;}struct W<T>(T);type X<T>=W<T>;impl A for W<u8>{fn f(self)->u8{self.0}}fn main(){let x=X::<_>::f(W(7u8));check(x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-bound-priority",
        "trait A{fn f(self)->u8;}trait B{fn f(self)->u8;}impl<T>B for T{fn f(self)->u8{9}}impl A for u8{fn f(self)->u8{7}}fn g<T:A>(x:T)->u8{T::f(x)}fn main(){let x=g(3u8);check(x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-bound-trait-outside-scope",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}fn g<T:m::A>(x:T)->u8{T::f(x)}fn main(){let x=g(3u8);check(x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-shadowed-trait-name",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}fn main(){struct A;let x=u8::f(3);check(x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-macro-call-site-trait",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}macro_rules! call{($x:expr)=>{<u8>::f($x)}}fn main(){use m::A;let x=call!(3u8);check(x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-macro-use-visible-after-expansion",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}macro_rules! enable{()=>{use m::A as _;}}fn main(){enable!();let x=u8::f(3);check(x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-macro-local-trait-visible-after-expansion",
        "macro_rules! enable{()=>{trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}}fn main(){enable!();let x=u8::f(3);check(x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-ambiguous-glob-same-method",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}mod n{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{9}}}use m::*;use n::*;fn main(){let x=u8::f(3);check(x==7);}",
        false,
    ),
    (
        "implicit-run-prefix-reversed-glob-imports-g",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{1}}}mod n{pub trait A{fn g(self)->u8;}impl A for u8{fn g(self)->u8{2}}}use n::*;use m::*;fn main(){let x=u8::g(3);check(x==2);}",
        false,
    ),
    (
        "implicit-run-prefix-reversed-glob-declarations-f",
        "mod n{pub trait A{fn g(self)->u8;}impl A for u8{fn g(self)->u8{2}}}mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{1}}}use m::*;use n::*;fn main(){let x=u8::f(3);check(x==1);}",
        false,
    ),
];
#[test]
fn implicit_callables_lower() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in EXECUTIONS {
        super::cases(&[(name, &format!("{}{source}", super::native::CHECK), true)])?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn implicit_callables_execute() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("implicit-callables", EXECUTIONS)
}
