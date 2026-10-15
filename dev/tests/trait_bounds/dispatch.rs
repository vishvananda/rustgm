//! Trait-qualified calls retain symbolic permissions and select actual executable members.
use super::cases;

#[test]
fn qualified_dispatch_0() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dispatch-trait-path",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let x=A::f(3u8);}",
            true,
        ),
        (
            "dispatch-bound-trait-path",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn g<T:A>(x:T)->u8{A::f(x)}fn main(){let x=g(3u8);}",
            true,
        ),
        (
            "dispatch-default-trait-path",
            "trait A{fn f(self)->u8 where Self:Sized{9}}impl A for u8{}fn main(){let x=A::f(3u8);}",
            true,
        ),
        (
            "dispatch-function-value",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let p=A::f;let x=p(3u8);}",
            true,
        ),
        (
            "dispatch-missing-bound",
            "trait A{fn f(self)->u8;}fn g<T>(x:T)->u8{A::f(x)}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn qualified_dispatch_1() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dispatch-missing-implementation",
            "trait A{fn f(self)->u8;}fn main(){let x=A::f(3u8);}",
            false,
        ),
        (
            "dispatch-bad-default",
            "trait A{fn f(self)->u8 where Self:Sized{false}}impl A for u8{}fn main(){}",
            false,
        ),
        (
            "dispatch-override-default",
            "trait A:Sized{fn f(self)->u8{9}}impl A for u8{fn f(self)->u8{self+1}}fn main(){let x=A::f(3u8);}",
            true,
        ),
        (
            "dispatch-default-required",
            "trait A:Sized{fn f(self)->u8{A::g(self)+1}fn g(self)->u8;}impl A for u8{fn g(self)->u8{self+2}}fn main(){let x=A::f(3u8);}",
            true,
        ),
        (
            "dispatch-default-recursive",
            "trait A:Sized{fn f(self,n:u8)->u8{if n==0{7}else{A::f(self,n-1)}}}impl A for u8{}fn main(){let x=A::f(3u8,3);}",
            true,
        ),
    ])
}

#[test]
fn qualified_dispatch_2() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dispatch-impl-generic",
            "trait A{fn f(self)->u8;}struct W<T>(T);impl<T:Copy>A for W<T>{fn f(self)->u8{let a=self.0;let b=self.0;7}}fn main(){let x=A::f(W(3u8));}",
            true,
        ),
        (
            "dispatch-method-generic",
            "trait A{fn f<T>(self,x:T)->T;}impl A for u8{fn f<U>(self,x:U)->U{x}}fn main(){let x=A::f(3u8,7u16);}",
            true,
        ),
        (
            "dispatch-method-generic-explicit",
            "trait A{fn f<T>(self,x:T)->T;}impl A for u8{fn f<U>(self,x:U)->U{x}}fn main(){let x=A::f::<u16>(3u8,7);}",
            true,
        ),
        (
            "dispatch-method-const",
            "trait A{fn f<const N:usize>(self,x:[u8;N])->[u8;N];}impl A for u8{fn f<const M:usize>(self,x:[u8;M])->[u8;M]{x}}fn main(){let x=A::f::<2>(3u8,[7,9]);}",
            true,
        ),
        (
            "dispatch-trait-argument-inferred",
            "trait A<T>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=A::f(3u8,7u16);}",
            true,
        ),
    ])
}

#[test]
fn qualified_dispatch_3() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dispatch-trait-argument-explicit",
            "trait A<T>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=A::<u16>::f(3u8,7);}",
            true,
        ),
        (
            "dispatch-trait-argument-wrong",
            "trait A<T>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=A::<u32>::f(3u8,7u16);}",
            false,
        ),
        (
            "dispatch-late-lifetimes",
            "trait A{fn f<'a>(&self,x:&'a u8)->&'a u8;}impl A for u8{fn f<'b>(&self,x:&'b u8)->&'b u8{x}}fn main(){let x=7u8;let y=A::f(&3u8,&x);}",
            true,
        ),
        (
            "dispatch-captured-lifetimes",
            "trait A<'a>{fn f(&self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(&self,x:&'a u8)->&'a u8{x}}fn main(){let x=7u8;let y=A::f(&3u8,&x);}",
            true,
        ),
        (
            "dispatch-early-method-lifetime",
            "trait A{fn f<'a:'a>(&self,x:&'a u8)->&'a u8;}impl A for u8{fn f<'b:'b>(&self,x:&'b u8)->&'b u8{x}}fn main(){let x=7u8;let y=A::f(&3u8,&x);}",
            true,
        ),
    ])
}

#[test]
fn qualified_dispatch_4() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dispatch-reference-self",
            "trait A{fn f(self)->u8;}impl<'a>A for &'a u8{fn f(self)->u8{*self}}fn g<'a>(x:&'a u8)->u8{A::f(x)}fn main(){let x=7u8;let y=g(&x);}",
            true,
        ),
        (
            "dispatch-borrowed-generic-self",
            "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl<'a>A for W<'a>{fn f(&self)->u8{*self.0}}fn g<T:A>(x:&T)->u8{A::f(x)}fn main(){let x=7u8;let y=g(&W(&x));}",
            true,
        ),
        (
            "dispatch-nested-function-value",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn pass<T>(x:T)->T{x}fn main(){let p=pass(A::f);let x=p(3u8);}",
            true,
        ),
        (
            "dispatch-array-function-value",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let p=[A::f;2];let x=p[1](3u8);}",
            true,
        ),
        (
            "dispatch-pointer-cast",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let p=A::f as fn(u8)->u8;let x=p(3u8);}",
            true,
        ),
    ])
}

#[test]
fn qualified_dispatch_5() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dispatch-constant-pointer",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}const P:fn(u8)->u8=A::f;fn main(){let x=P(3u8);}",
            true,
        ),
        (
            "dispatch-bad-method-own-bound",
            "trait A{fn f<T:Copy>(self,x:T);}impl A for u8{fn f<T:Copy>(self,x:T){}}struct N;fn main(){A::f(3u8,N);}",
            false,
        ),
    ])
}
