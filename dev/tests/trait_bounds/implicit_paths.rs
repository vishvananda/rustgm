//! Implicit trait lookup retains typed prefixes, scoped candidates and ordinary callable proof.
use super::cases;

#[test]
fn primitive_nominal_and_rigid_prefixes_preserve_self() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "primitive-type-path",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let x=u8::f(3);}",
            true,
        ),
        (
            "primitive-angle-path",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let x=<u8>::f(3);}",
            true,
        ),
        (
            "nominal-type-path",
            "trait A{fn f(self)->u8;}struct S;impl A for S{fn f(self)->u8{7}}fn main(){let x=S::f(S);}",
            true,
        ),
        (
            "generic-type-path",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn g<T:A>(x:T)->u8{T::f(x)}fn main(){let x=g(3u8);}",
            true,
        ),
        (
            "generic-angle-path",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn g<T:A>(x:T)->u8{<T>::f(x)}fn main(){let x=g(3u8);}",
            true,
        ),
        (
            "self-default",
            "trait A:Sized{fn f(self)->u8;fn g(self)->u8{Self::f(self)+1}}impl A for u8{fn f(self)->u8{self}}fn main(){let x=u8::g(3);}",
            true,
        ),
        (
            "self-cannot-infer-another-type",
            "trait A:Sized{fn f(self)->u8;fn g(self)->u8{Self::f(3u16)}}impl A for u8{fn f(self)->u8{1}}impl A for u16{fn f(self)->u8{2}}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn supertraits_and_function_values_require_unique_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "type-supertrait",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}trait B:A{}impl B for u8{}fn g<T:B>(x:T)->u8{T::f(x)}fn main(){let x=g(3u8);}",
            true,
        ),
        (
            "type-no-bound",
            "trait A{fn f(self)->u8;}fn g<T>(x:T)->u8{T::f(x)}fn main(){}",
            false,
        ),
        (
            "type-function-value",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let p=u8::f;let x=p(3);}",
            true,
        ),
        (
            "type-ambiguity",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}trait B{fn f(self)->u8;}impl B for u8{fn f(self)->u8{self}}fn main(){let x=u8::f(3);}",
            false,
        ),
        (
            "prefix-argument-cannot-disambiguate",
            "trait A{fn f(self,x:u8);}trait B{fn f(self,x:u16);}impl A for u8{fn f(self,x:u8){}}impl B for u8{fn f(self,x:u16){}}fn main(){u8::f(3,4u8);}",
            false,
        ),
        (
            "prefix-result-cannot-disambiguate",
            "trait A{fn f(self)->u8;}trait B{fn f(self)->u16;}impl A for u8{fn f(self)->u8{1}}impl B for u8{fn f(self)->u16{2}}fn main(){let x:u8=u8::f(3);}",
            false,
        ),
    ])
}

#[test]
fn inherent_applicability_precedes_trait_fallback() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "prefix-inherent-bound-fallback",
            "trait A{fn f(self)->u8;}struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->u8{1}}impl<T>A for S<T>{fn f(self)->u8{2}}struct N;fn main(){let x=S::<N>::f(S(N));}",
            true,
        ),
        (
            "prefix-rigid-inherent-bound-fallback",
            "trait A{fn f(self)->u8;}struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->u8{1}}impl<T>A for S<T>{fn f(self)->u8{2}}fn g<T>(x:S<T>)->u8{S::<T>::f(x)}struct N;fn main(){let x=g(S(N));}",
            true,
        ),
        (
            "prefix-inferred-inherent-bound",
            "trait A{fn f(self)->u8;}struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->u8{1}}impl<T>A for S<T>{fn f(self)->u8{2}}struct N;fn main(){let x=S::f(S(N));}",
            false,
        ),
        (
            "prefix-method-bound-no-fallback",
            "trait A{fn f<U>(self,x:U);}struct S;impl S{fn f<U:Copy>(self,x:U){}}impl A for S{fn f<U>(self,x:U){}}struct N;fn main(){S::f(S,N);}",
            false,
        ),
        (
            "prefix-explicit-trait-bypasses-inherent-bound",
            "trait A{fn f<U>(self,x:U);}struct S;impl S{fn f<U:Copy>(self,x:U){}}impl A for S{fn f<U>(self,x:U){}}struct N;fn main(){A::f(S,N);}",
            true,
        ),
    ])
}

#[test]
fn prefix_segments_have_their_own_argument_rules() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "prefix-module-arguments",
            "mod m{pub trait A{fn f(self)->u8;}pub struct S;impl A for S{fn f(self)->u8{7}}}use m::A;fn main(){let x=m::<u8>::S::f(m::S);}",
            false,
        ),
        (
            "prefix-parameter-arguments",
            "trait A{fn f(self)->u8;}fn g<T:A>(x:T)->u8{T::<u8>::f(x)}fn main(){}",
            false,
        ),
        (
            "prefix-self-arguments",
            "trait A:Sized{fn f(self)->u8;fn g(self)->u8{Self::<u8>::f(self)}}impl A for u8{fn f(self)->u8{7}}fn main(){}",
            false,
        ),
        (
            "prefix-default-trait-argument-inferred",
            "trait A<T=u8>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=u8::f(3,7u16);}",
            true,
        ),
        (
            "prefix-trait-captured-lifetime",
            "trait A<'a>{fn f(self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(self,x:&'a u8)->&'a u8{x}}fn g<'a>(x:&'a u8)->&'a u8{u8::f(3,x)}fn main(){let n=7u8;let x=g(&n);}",
            true,
        ),
    ])
}

#[test]
fn implicit_trait_arguments_infer_through_the_callable() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "prefix-generic-trait-argument",
            "trait A<T>{fn f(self,x:T)->T;}fn g<X:A<Y>,Y>(x:X,y:Y)->Y{X::f(x,y)}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=g(3u8,7u16);}",
            true,
        ),
        (
            "prefix-trait-argument-uninferred",
            "trait A<T>{fn f(self)->u8;}impl A<u8>for u8{fn f(self)->u8{7}}impl A<u16>for u8{fn f(self)->u8{9}}fn main(){let x=u8::f(3);}",
            false,
        ),
        (
            "prefix-method-const",
            "trait A{fn f<const N:usize>(self,x:[u8;N])->[u8;N];}impl A for u8{fn f<const N:usize>(self,x:[u8;N])->[u8;N]{x}}fn main(){let x=u8::f::<2>(3,[7,9]);}",
            true,
        ),
        (
            "prefix-constant-function-value",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}const P:fn(u8)->u8=u8::f;fn main(){let x=P(3);}",
            true,
        ),
    ])
}

#[test]
fn compound_and_alias_prefixes_share_annotation_inference() -> Result<(), Box<dyn std::error::Error>>
{
    cases(&[
        (
            "prefix-reference-type",
            "trait A{fn f(self)->u8;}impl<'a>A for &'a u8{fn f(self)->u8{*self}}fn main(){let n=7u8;let x=<&u8>::f(&n);}",
            true,
        ),
        (
            "prefix-array-type",
            "trait A{fn f(self)->u8;}impl<const N:usize>A for [u8;N]{fn f(self)->u8{self[0]}}fn main(){let x=<[u8;2]>::f([7,9]);}",
            true,
        ),
        (
            "prefix-alias-hole",
            "trait A{fn f(self)->u8;}struct W<T>(T);type X<T>=W<T>;impl A for W<u8>{fn f(self)->u8{self.0}}fn main(){let x=X::<_>::f(W(7u8));}",
            true,
        ),
    ])
}

#[test]
fn enum_variants_keep_priority_over_trait_methods() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "implicit-alias-variant-before-method",
            "trait A{fn F()->u8;}enum E{F}impl A for E{fn F()->u8{7}}type X=E;fn main(){let _:E=X::F;}",
            true,
        ),
        (
            "implicit-unit-variant-is-not-a-callable",
            "trait A{fn F()->u8;}enum E{F}impl A for E{fn F()->u8{7}}type X=E;fn main(){let x=X::F();}",
            false,
        ),
    ])
}
