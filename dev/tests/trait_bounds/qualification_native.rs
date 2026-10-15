//! Qualified callable operands execute the explicitly selected trait implementation.
const EXECUTIONS: &[(&str, &str, bool)] = &[
    (
        "qualification-run-primitive",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let x=<u8 as A>::f(3);check(x==4);}",
        false,
    ),
    (
        "qualification-run-rigid-self",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}fn g<T:A>(x:T)->u8{<T as A>::f(x)}fn main(){let x=g(3u8);check(x==3);}",
        false,
    ),
    (
        "qualification-run-default-body",
        "trait A:Sized{fn f(self)->u8{9}}impl A for u8{}fn main(){let x=<u8 as A>::f(3);check(x==9);}",
        false,
    ),
    (
        "qualification-run-own-arguments",
        "trait A{fn f<T>(self,x:T)->T;}impl A for u8{fn f<U>(self,x:U)->U{x}}fn main(){let x=<u8 as A>::f::<u16>(3,7);check(x==7);}",
        false,
    ),
    (
        "qualification-run-trait-argument",
        "trait A<T>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=<u8 as A<u16>>::f(3,7);check(x==7);}",
        false,
    ),
    (
        "qualification-run-trait-hole",
        "trait A<T>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=<u8 as A<_>>::f(3,7u16);check(x==7);}",
        false,
    ),
    (
        "qualification-run-self-hole",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}fn main(){let x=<_ as A>::f(3u8);check(x==3);}",
        false,
    ),
    (
        "qualification-run-nominal-hole",
        "trait A{fn f(self)->u8;}struct W<T>(T);impl A for W<u8>{fn f(self)->u8{self.0}}fn main(){let x=<W<_>as A>::f(W(3u8));check(x==3);}",
        false,
    ),
    (
        "qualification-run-trait-default",
        "trait A<T=u16>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=<u8 as A>::f(3,7u16);check(x==7);}",
        false,
    ),
    (
        "qualification-run-trait-self-default",
        "trait A<T=Self>{fn f(self,x:T)->T;}impl A for u8{fn f(self,x:u8)->u8{x}}fn main(){let x=<u8 as A>::f(3,7);check(x==7);}",
        false,
    ),
    (
        "qualification-run-explicit-lifetime",
        "trait A<'a>{fn f(self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(self,x:&'a u8)->&'a u8{x}}fn g<'a>(x:&'a u8)->&'a u8{<u8 as A<'a>>::f(3,x)}fn main(){let x=7u8;let y=g(&x);check(*y==7);}",
        false,
    ),
    (
        "qualification-run-reference-self",
        "trait A{fn f(self)->u8;}impl<'a>A for &'a u8{fn f(self)->u8{*self}}fn g<'a>(x:&'a u8)->u8{<&'a u8 as A>::f(x)}fn main(){let x=7u8;let y=g(&x);check(y==7);}",
        false,
    ),
    (
        "qualification-run-array-self",
        "trait A{fn f(self)->u8;}impl<const N:usize>A for [u8;N]{fn f(self)->u8{self[0]}}fn main(){let x=<[u8;2]as A>::f([7,9]);check(x==7);}",
        false,
    ),
    (
        "qualification-run-disambiguation",
        "trait A{fn f(self)->u8;}trait B{fn f(self)->u8;}impl A for u8{fn f(self)->u8{1}}impl B for u8{fn f(self)->u8{2}}fn main(){let a=<u8 as A>::f(3);let b=<u8 as B>::f(3);check(a==1&&b==2);}",
        false,
    ),
    (
        "qualification-run-inherent-name",
        "struct S;impl S{fn f(self)->u8{1}}trait A{fn f(self)->u8;}impl A for S{fn f(self)->u8{2}}fn main(){let x=<S as A>::f(S);check(x==2);}",
        false,
    ),
    (
        "qualification-run-constant-value",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}const P:fn(u8)->u8=<u8 as A>::f;fn main(){let x=P(3);check(x==4);}",
        false,
    ),
    (
        "qualification-run-module-qualification",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}fn main(){let x=<u8 as m::A>::f(3);check(x==3);}",
        false,
    ),
    (
        "qualification-run-default-qualified-recursion",
        "trait A:Sized{fn f(self,n:u8)->u8{if n==0{7}else{<Self as A>::f(self,n-1)}}}impl A for u8{}fn main(){let x=<u8 as A>::f(3,2);check(x==7);}",
        false,
    ),
    (
        "qualification-run-const-arguments",
        "trait A<const N:usize>{fn f<const M:usize>(self,x:[u8;M])->[u8;M];}impl<const N:usize>A<N>for u8{fn f<const M:usize>(self,x:[u8;M])->[u8;M]{x}}fn main(){let x=<u8 as A<2>>::f::<3>(7,[1,2,3]);check(x[0]==1&&x[2]==3);}",
        false,
    ),
    (
        "qualification-run-dependent-self-default",
        "trait A<T,U=Self>{fn f(self,x:T,y:U)->U;}impl A<u16>for u8{fn f(self,x:u16,y:u8)->u8{y}}fn main(){let x=<u8 as A<u16>>::f(3,7,9);check(x==9);}",
        false,
    ),
    (
        "qualification-run-const-default",
        "trait A<const N:usize=2>{fn f(self,x:[u8;N])->[u8;N];}impl A for u8{fn f(self,x:[u8;2])->[u8;2]{x}}fn main(){let x=<u8 as A>::f(3,[7,9]);check(x[0]==7&&x[1]==9);}",
        false,
    ),
    (
        "qualification-run-const-hole",
        "trait A<const N:usize>{fn f(self,x:[u8;N])->[u8;N];}impl<const N:usize>A<N>for u8{fn f(self,x:[u8;N])->[u8;N]{x}}fn main(){let x=<u8 as A<_>>::f(3,[7,9]);check(x[0]==7&&x[1]==9);}",
        false,
    ),
    (
        "qualification-run-nested-argument-hole",
        "trait A<T>{fn f(self,x:T)->T;}impl A<(u8,u16)>for u8{fn f(self,x:(u8,u16))->(u8,u16){x}}fn main(){let x=<u8 as A<(u8,_)>>::f(3,(7,9u16));check(x.0==7&&x.1==9);}",
        false,
    ),
    (
        "qualification-run-alias-self-hole",
        "trait A{fn f(self)->u8;}struct W<T>(T);type X<T>=W<T>;impl A for W<u8>{fn f(self)->u8{self.0}}fn main(){let x=<X<_>as A>::f(W(7u8));check(x==7);}",
        false,
    ),
    (
        "qualification-run-imported-trait",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}use m::A as Alias;fn main(){let x=<u8 as Alias>::f(7);check(x==7);}",
        false,
    ),
    (
        "qualification-run-macro-qualification",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}macro_rules! call{($x:expr)=>{<u8 as A>::f($x)}}fn main(){let x=call!(7);check(x==7);}",
        false,
    ),
    (
        "qualification-run-own-early-lifetime",
        "trait A<'a>{fn f<'b:'b>(self,x:&'a u8,y:&'b u8)->&'b u8;}impl<'a>A<'a>for u8{fn f<'b:'b>(self,x:&'a u8,y:&'b u8)->&'b u8{y}}fn g<'a>(x:&'a u8)->&'a u8{<u8 as A<'a>>::f::<'a>(3,x,x)}fn main(){let n=7u8;let x=g(&n);check(*x==7);}",
        false,
    ),
    (
        "qualification-run-nested-callable-value",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn pass<T>(x:T)->T{x}fn main(){let p=pass(<u8 as A>::f);let x=p(3);check(x==4);}",
        false,
    ),
];
#[test]
fn fully_qualified_callables_lower() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in EXECUTIONS {
        super::cases(&[(name, &format!("{}{source}", super::native::CHECK), true)])?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn fully_qualified_callables_execute() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("fully-qualified-callables", EXECUTIONS)
}
