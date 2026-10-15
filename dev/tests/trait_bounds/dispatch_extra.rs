//! Owner arguments, defaults and callable lifetimes remain distinct through dispatch.
use super::cases;

#[test]
fn dispatch_owner_arguments_0() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dispatch-default-trait-argument",
            "trait A<T=u8>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=A::f(3u8,7u16);}",
            true,
        ),
        (
            "dispatch-explicit-trailing-default",
            "trait A<T,U=T>{fn f(self,x:T,y:U)->U;}impl A<u16>for u8{fn f(self,x:u16,y:u16)->u16{y}}fn main(){let x=A::<u16>::f(3u8,7,9);}",
            true,
        ),
        (
            "dispatch-explicit-default-self",
            "trait A<T,U=Self>{fn f(self,x:T,y:U)->U;}impl A<u16>for u8{fn f(self,x:u16,y:u8)->u8{y}}fn main(){let x=A::<u16>::f(3u8,7,9);}",
            true,
        ),
        (
            "dispatch-explicit-default-conflict",
            "trait A<T,U=T>{fn f(self,x:T,y:U)->U;}impl A<u16,u32>for u8{fn f(self,x:u16,y:u32)->u32{y}}fn main(){let x=A::<u16>::f(3u8,7,9u32);}",
            false,
        ),
        (
            "dispatch-trait-const",
            "trait A<const N:usize>{fn f(self,x:[u8;N])->u8;}impl A<2>for u8{fn f(self,x:[u8;2])->u8{x[1]}}fn main(){let x=A::<2>::f(3u8,[7,9]);}",
            true,
        ),
    ])
}

#[test]
fn dispatch_owner_arguments_1() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dispatch-trait-const-wrong",
            "trait A<const N:usize>{fn f(self,x:[u8;N])->u8;}impl A<2>for u8{fn f(self,x:[u8;2])->u8{x[1]}}fn main(){let x=A::<3>::f(3u8,[7,9]);}",
            false,
        ),
        (
            "dispatch-trait-const-default",
            "trait A<T,const N:usize=2>{fn f(self,x:T,y:[u8;N])->u8;}impl A<u16>for u8{fn f(self,x:u16,y:[u8;2])->u8{y[1]}}fn main(){let x=A::<u16>::f(3u8,7,[8,9]);}",
            true,
        ),
        (
            "dispatch-trait-lifetime-explicit",
            "trait A<'a>{fn f(self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(self,x:&'a u8)->&'a u8{x}}fn g<'b>(x:&'b u8)->&'b u8{A::<'b>::f(3u8,x)}fn main(){let x=7u8;let y=g(&x);}",
            true,
        ),
        (
            "dispatch-trait-lifetime-static-rejects-local",
            "trait A<'a>{fn f(self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(self,x:&'a u8)->&'a u8{x}}fn main(){let x=7u8;let y=A::<'static>::f(3u8,&x);}",
            false,
        ),
        (
            "dispatch-reordered-impl-arguments",
            "trait A<T>{fn f(self,x:T)->T;}struct W<T,U>(T,U);impl<V:Copy,U:Copy>A<U>for W<U,V>{fn f(self,x:U)->U{x}}fn main(){let x=A::f(W(3u8,4u16),7u8);}",
            true,
        ),
    ])
}

#[test]
fn dispatch_owner_arguments_2() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dispatch-nested-reference-function-value",
            "trait A{fn f(self)->u8;}impl<'a>A for &'a u8{fn f(self)->u8{*self}}fn pass<T>(x:T)->T{x}fn g<T:A>(x:T)->u8{let p=pass(A::f);p(x)}fn main(){let v=7u8;let x=g(&v);}",
            true,
        ),
        (
            "dispatch-late-more-general-override",
            "trait A{fn f<'a>(self,x:&'a u8,y:&'a u8)->&'a u8;}impl A for u8{fn f<'b,'c>(self,x:&'b u8,y:&'c u8)->&'b u8{x}}fn main(){let a=7u8;let b=9u8;let p=A::f as for<'a>fn(u8,&'a u8,&'a u8)->&'a u8;let x=p(3,&a,&b);}",
            true,
        ),
        (
            "dispatch-default-calls-overridden-default",
            "trait A:Sized{fn f(self)->u8{A::g(self)+1}fn g(self)->u8{3}}impl A for u8{fn g(self)->u8{self+2}}fn main(){let x=A::f(7u8);}",
            true,
        ),
        (
            "dispatch-nested-default-function-value",
            "trait A:Sized{fn f(self)->u8{let p=A::g;p(self)+1}fn g(self)->u8{3}}impl A for u8{fn g(self)->u8{self+2}}fn main(){let x=A::f(7u8);}",
            true,
        ),
        (
            "dispatch-static-pointer",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}static P:fn(u8)->u8=A::f;fn main(){let x=P(3u8);}",
            true,
        ),
    ])
}

#[test]
fn dispatch_owner_arguments_3() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dispatch-nested-constant-pointer",
            "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}const P:(fn(u8)->u8,[fn(u8)->u8;2])=(A::f,[A::f;2]);fn main(){let x=(P.1)[1](3u8);}",
            true,
        ),
        (
            "dispatch-constant-generic-impl",
            "trait A{fn f(self)->u8;}struct W<T>(T);impl<T:Copy>A for W<T>{fn f(self)->u8{7}}const P:fn(W<u8>)->u8=A::f;fn main(){let x=P(W(3u8));}",
            true,
        ),
        (
            "dispatch-constant-default",
            "trait A:Sized{fn f(self)->u8{7}}impl A for u8{}const P:fn(u8)->u8=A::f;fn main(){let x=P(3u8);}",
            true,
        ),
        (
            "dispatch-constant-borrowed-method",
            "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self+1}}const P:fn(&u8)->u8=A::f;fn main(){let x=P(&3u8);}",
            true,
        ),
        (
            "dispatch-unknown-self",
            "trait A{fn f()->u8;}impl A for u8{fn f()->u8{7}}fn main(){let x=A::f();}",
            false,
        ),
    ])
}

#[test]
fn dispatch_owner_arguments_4() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "dispatch-self-result-inference",
            "trait A{fn f()->Self;}impl A for u8{fn f()->Self{7}}fn main(){let x:u8=A::f();}",
            true,
        ),
        (
            "dispatch-unrelated-trait-parameter",
            "trait A<T>{fn f(self)->u8;}impl<T>A<T>for u8{fn f(self)->u8{7}}fn main(){let x=A::f(3u8);}",
            false,
        ),
        (
            "dispatch-conditional-impl-missing",
            "trait A{fn f(self)->u8;}struct W<T>(T);impl<T:Copy>A for W<T>{fn f(self)->u8{7}}struct N;fn main(){let x=A::f(W(N));}",
            false,
        ),
    ])
}
