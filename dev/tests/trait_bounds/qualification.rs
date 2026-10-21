//! Fully qualified callable paths share Self inference and type-style argument rules.
use super::cases;

#[test]
fn qualified_self_requires_valid_trait_evidence() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "qualification-primitive",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let x=<u8 as A>::f(3);}",
            true,
        ),
        (
            "qualification-rigid-self",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}fn g<T:A>(x:T)->u8{<T as A>::f(x)}fn main(){let x=g(3u8);}",
            true,
        ),
        (
            "qualification-missing-bound",
            "trait A{fn f(self)->u8;}fn g<T>(x:T)->u8{<T as A>::f(x)}fn main(){}",
            false,
        ),
        (
            "qualification-missing-implementation",
            "trait A{fn f(self)->u8;}fn main(){let x=<u8 as A>::f(3);}",
            false,
        ),
        (
            "qualification-wrong-self",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}fn main(){let x=<u8 as A>::f(3u16);}",
            false,
        ),
        (
            "qualification-default-body",
            "trait A:Sized{fn f(self)->u8{9}}impl A for u8{}fn main(){let x=<u8 as A>::f(3);}",
            true,
        ),
    ])
}

#[test]
fn qualified_arguments_and_holes_share_inference() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "qualification-own-arguments",
            "trait A{fn f<T>(self,x:T)->T;}impl A for u8{fn f<U>(self,x:U)->U{x}}fn main(){let x=<u8 as A>::f::<u16>(3,7);}",
            true,
        ),
        (
            "qualification-trait-argument",
            "trait A<T>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=<u8 as A<u16>>::f(3,7);}",
            true,
        ),
        (
            "qualification-missing-trait-argument",
            "trait A<T>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=<u8 as A>::f(3,7u16);}",
            false,
        ),
        (
            "qualification-trait-hole",
            "trait A<T>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=<u8 as A<_>>::f(3,7u16);}",
            true,
        ),
        (
            "qualification-self-hole",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}fn main(){let x=<_ as A>::f(3u8);}",
            true,
        ),
        (
            "qualification-nominal-hole",
            "trait A{fn f(self)->u8;}struct W<T>(T);impl A for W<u8>{fn f(self)->u8{self.0}}fn main(){let x=<W<_>as A>::f(W(3u8));}",
            true,
        ),
    ])
}

#[test]
fn qualified_defaults_and_lifetimes_obey_annotations() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "qualification-trait-default",
            "trait A<T=u16>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=<u8 as A>::f(3,7u16);}",
            true,
        ),
        (
            "qualification-trait-default-conflict",
            "trait A<T=u16>{fn f(self,x:T)->T;}impl A<u32>for u8{fn f(self,x:u32)->u32{x}}fn main(){let x=<u8 as A>::f(3,7u32);}",
            false,
        ),
        (
            "qualification-trait-self-default",
            "trait A<T=Self>{fn f(self,x:T)->T;}impl A for u8{fn f(self,x:u8)->u8{x}}fn main(){let x=<u8 as A>::f(3,7);}",
            true,
        ),
        (
            "qualification-explicit-lifetime",
            "trait A<'a>{fn f(self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(self,x:&'a u8)->&'a u8{x}}fn g<'a>(x:&'a u8)->&'a u8{<u8 as A<'a>>::f(3,x)}fn main(){let x=7u8;let y=g(&x);}",
            true,
        ),
        (
            "qualification-static-conflict",
            "trait A<'a>{fn f(self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(self,x:&'a u8)->&'a u8{x}}fn main(){let x=7u8;let y=<u8 as A<'static>>::f(3,&x);}",
            false,
        ),
        (
            "qualification-reference-self",
            "trait A{fn f(self)->u8;}impl<'a>A for &'a u8{fn f(self)->u8{*self}}fn g<'a>(x:&'a u8)->u8{<&'a u8 as A>::f(x)}fn main(){let x=7u8;let y=g(&x);}",
            true,
        ),
    ])
}

#[test]
fn explicit_traits_disambiguate_direct_methods() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "qualification-array-self",
            "trait A{fn f(self)->u8;}impl<const N:usize>A for [u8;N]{fn f(self)->u8{self[0]}}fn main(){let x=<[u8;2]as A>::f([7,9]);}",
            true,
        ),
        (
            "qualification-disambiguation",
            "trait A{fn f(self)->u8;}trait B{fn f(self)->u8;}impl A for u8{fn f(self)->u8{1}}impl B for u8{fn f(self)->u8{2}}fn main(){let a=<u8 as A>::f(3);let b=<u8 as B>::f(3);}",
            true,
        ),
        (
            "qualification-inherent-name",
            "struct S;impl S{fn f(self)->u8{1}}trait A{fn f(self)->u8;}impl A for S{fn f(self)->u8{2}}fn main(){let x=<S as A>::f(S);}",
            true,
        ),
        (
            "qualification-constant-value",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}const P:fn(u8)->u8=<u8 as A>::f;fn main(){let x=P(3);}",
            true,
        ),
        (
            "qualification-module-qualification",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}fn main(){let x=<u8 as m::A>::f(3);}",
            true,
        ),
        (
            "qualification-supertrait-not-direct-member",
            "trait A{fn f(self)->u8;}trait B:A{}impl A for u8{fn f(self)->u8{self}}impl B for u8{}fn main(){let x=<u8 as B>::f(3);}",
            false,
        ),
    ])
}

#[test]
fn qualified_method_syntax_rejects_unavailable_members() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "qualification-wrong-trait-kind",
            "struct A;fn main(){let x=<u8 as A>::f(3);}",
            false,
        ),
        (
            "qualification-missing-member",
            "trait A{}impl A for u8{}fn main(){let x=<u8 as A>::f(3);}",
            false,
        ),
        (
            "qualification-default-qualified-recursion",
            "trait A:Sized{fn f(self,n:u8)->u8{if n==0{7}else{<Self as A>::f(self,n-1)}}}impl A for u8{}fn main(){let x=<u8 as A>::f(3,2);}",
            true,
        ),
        (
            "qualification-const-arguments",
            "trait A<const N:usize>{fn f<const M:usize>(self,x:[u8;M])->[u8;M];}impl<const N:usize>A<N>for u8{fn f<const M:usize>(self,x:[u8;M])->[u8;M]{x}}fn main(){let x=<u8 as A<2>>::f::<3>(7,[1,2,3]);}",
            true,
        ),
        (
            "qualification-extra-trait-arity",
            "trait A<T>{fn f(self,x:T);}impl A<u8>for u8{fn f(self,x:u8){}}fn main(){<u8 as A<u8,u16>>::f(3,7);}",
            false,
        ),
        (
            "qualification-missing-second-argument",
            "trait A<T,U>{fn f(self,x:T,y:U);}impl A<u8,u16>for u8{fn f(self,x:u8,y:u16){}}fn main(){<u8 as A<u8>>::f(3,7,9u16);}",
            false,
        ),
    ])
}

#[test]
fn type_prefixes_apply_defaults_and_nested_holes() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "qualification-dependent-self-default",
            "trait A<T,U=Self>{fn f(self,x:T,y:U)->U;}impl A<u16>for u8{fn f(self,x:u16,y:u8)->u8{y}}fn main(){let x=<u8 as A<u16>>::f(3,7,9);}",
            true,
        ),
        (
            "qualification-const-default",
            "trait A<const N:usize=2>{fn f(self,x:[u8;N])->[u8;N];}impl A for u8{fn f(self,x:[u8;2])->[u8;2]{x}}fn main(){let x=<u8 as A>::f(3,[7,9]);}",
            true,
        ),
        (
            "qualification-const-hole",
            "trait A<const N:usize>{fn f(self,x:[u8;N])->[u8;N];}impl<const N:usize>A<N>for u8{fn f(self,x:[u8;N])->[u8;N]{x}}fn main(){let x=<u8 as A<_>>::f(3,[7,9]);}",
            true,
        ),
        (
            "qualification-nested-argument-hole",
            "trait A<T>{fn f(self,x:T)->T;}impl A<(u8,u16)>for u8{fn f(self,x:(u8,u16))->(u8,u16){x}}fn main(){let x=<u8 as A<(u8,_)>>::f(3,(7,9u16));}",
            true,
        ),
        (
            "qualification-alias-self-hole",
            "trait A{fn f(self)->u8;}struct W<T>(T);type X<T>=W<T>;impl A for W<u8>{fn f(self)->u8{self.0}}fn main(){let x=<X<_>as A>::f(W(7u8));}",
            true,
        ),
        (
            "qualification-imported-trait",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}use m::A as Alias;fn main(){let x=<u8 as Alias>::f(7);}",
            true,
        ),
    ])
}

#[test]
fn qualified_paths_preserve_imports_macros_and_captures() -> Result<(), Box<dyn std::error::Error>>
{
    cases(&[
        (
            "qualification-macro-qualification",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}macro_rules! call{($x:expr)=>{<u8 as A>::f($x)}}fn main(){let x=call!(7);}",
            true,
        ),
        (
            "qualification-own-early-lifetime",
            "trait A<'a>{fn f<'b:'b>(self,x:&'a u8,y:&'b u8)->&'b u8;}impl<'a>A<'a>for u8{fn f<'b:'b>(self,x:&'a u8,y:&'b u8)->&'b u8{y}}fn g<'a>(x:&'a u8)->&'a u8{<u8 as A<'a>>::f::<'a>(3,x,x)}fn main(){let n=7u8;let x=g(&n);}",
            true,
        ),
        (
            "qualification-lifetime-arity",
            "trait A<'a>{fn f(self,x:&'a u8);}impl<'a>A<'a>for u8{fn f(self,x:&'a u8){}}fn main(){<u8 as A<'static,'static>>::f(3,&7);}",
            false,
        ),
        (
            "qualification-module-arguments",
            "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}fn main(){let x=<u8 as m::<u8>::A>::f(3);}",
            false,
        ),
        (
            "qualification-further-suffix",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}fn main(){let x=<u8 as A>::f::g(3);}",
            false,
        ),
        (
            "qualification-unsafe-required",
            "trait A{unsafe fn f(self)->u8;}impl A for u8{unsafe fn f(self)->u8{self}}fn main(){let x=<u8 as A>::f(7);}",
            false,
        ),
    ])
}

#[test]
fn qualification_preserves_context_and_implementation_requirements()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "qualification-nested-callable-value",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn pass<T>(x:T)->T{x}fn main(){let p=pass(<u8 as A>::f);let x=p(3);}",
            true,
        ),
        (
            "qualification-unknown-self",
            "trait A{fn f()->u8;}impl A for u8{fn f()->u8{7}}fn main(){let x=<_ as A>::f();}",
            false,
        ),
        (
            "qualification-static-receiver-short",
            "trait A{fn f(self)->u8;}impl A for &'static u8{fn f(self)->u8{*self}}fn main(){let n=7u8;let x=<&u8 as A>::f(&n);}",
            false,
        ),
        (
            "qualification-conditional-implementation",
            "trait B{}trait A{fn f(self)->u8;}struct W<T>(T);impl<T:B>A for W<T>{fn f(self)->u8{7}}fn main(){let x=<W<u8>as A>::f(W(3));}",
            false,
        ),
        (
            "qualification-default-parameter-bound",
            "trait A<T:Copy=Self>{fn f(self,x:T)->T;}struct N;impl A<u8>for N{fn f(self,x:u8)->u8{x}}fn main(){let x=<N as A>::f(N,7u8);}",
            false,
        ),
    ])
}

#[test]
fn unsafe_qualified_calls_require_lexical_permission() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "qualified-unsafe-block",
            "trait A{unsafe fn f(self)->u8;}impl A for u8{unsafe fn f(self)->u8{self}}fn main(){let x=unsafe{<u8 as A>::f(7)};}",
            true,
        ),
        (
            "qualified-unsafe-missing",
            "trait A{unsafe fn f(self)->u8;}impl A for u8{unsafe fn f(self)->u8{self}}fn main(){let x=<u8 as A>::f(7);}",
            false,
        ),
        (
            "qualified-unsafe-nested-item",
            "trait A{unsafe fn f(self)->u8;}impl A for u8{unsafe fn f(self)->u8{self}}fn main(){unsafe{fn nested(){let x=<u8 as A>::f(7);}nested();}}",
            false,
        ),
    ])
}
