//! Every selected override/default must execute, including stored callable values.
const EXECUTIONS: &[(&str, &str, bool)] = &[
    (
        "dispatch-run-override-default",
        "trait A:Sized{fn f(self)->u8{9}}impl A for u8{fn f(self)->u8{self+1}}fn main(){let x=A::f(3u8);check(x==4);}",
        false,
    ),
    (
        "dispatch-run-default-required",
        "trait A:Sized{fn f(self)->u8{A::g(self)+1}fn g(self)->u8;}impl A for u8{fn g(self)->u8{self+2}}fn main(){let x=A::f(3u8);check(x==6);}",
        false,
    ),
    (
        "dispatch-run-default-recursive",
        "trait A:Sized{fn f(self,n:u8)->u8{if n==0{7}else{A::f(self,n-1)}}}impl A for u8{}fn main(){let x=A::f(3u8,3);check(x==7);}",
        false,
    ),
    (
        "dispatch-run-impl-generic",
        "trait A{fn f(self)->u8;}struct W<T>(T);impl<T:Copy>A for W<T>{fn f(self)->u8{let a=self.0;let b=self.0;7}}fn main(){let x=A::f(W(3u8));check(x==7);}",
        false,
    ),
    (
        "dispatch-run-method-generic",
        "trait A{fn f<T>(self,x:T)->T;}impl A for u8{fn f<U>(self,x:U)->U{x}}fn main(){let x=A::f(3u8,7u16);check(x==7);}",
        false,
    ),
    (
        "dispatch-run-method-const",
        "trait A{fn f<const N:usize>(self,x:[u8;N])->[u8;N];}impl A for u8{fn f<const M:usize>(self,x:[u8;M])->[u8;M]{x}}fn main(){let x=A::f::<2>(3u8,[7,9]);check(x[0]==7&&x[1]==9);}",
        false,
    ),
    (
        "dispatch-run-trait-argument-explicit",
        "trait A<T>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=A::<u16>::f(3u8,7);check(x==7);}",
        false,
    ),
    (
        "dispatch-run-late-lifetimes",
        "trait A{fn f<'a>(&self,x:&'a u8)->&'a u8;}impl A for u8{fn f<'b>(&self,x:&'b u8)->&'b u8{x}}fn main(){let x=7u8;let y=A::f(&3u8,&x);check(*y==7);}",
        false,
    ),
    (
        "dispatch-run-captured-lifetimes",
        "trait A<'a>{fn f(&self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(&self,x:&'a u8)->&'a u8{x}}fn main(){let x=7u8;let y=A::f(&3u8,&x);check(*y==7);}",
        false,
    ),
    (
        "dispatch-run-early-method-lifetime",
        "trait A{fn f<'a:'a>(&self,x:&'a u8)->&'a u8;}impl A for u8{fn f<'b:'b>(&self,x:&'b u8)->&'b u8{x}}fn main(){let x=7u8;let y=A::f(&3u8,&x);check(*y==7);}",
        false,
    ),
    (
        "dispatch-run-reference-self",
        "trait A{fn f(self)->u8;}impl<'a>A for &'a u8{fn f(self)->u8{*self}}fn g<'a>(x:&'a u8)->u8{A::f(x)}fn main(){let x=7u8;let y=g(&x);check(y==7);}",
        false,
    ),
    (
        "dispatch-run-borrowed-generic-self",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl<'a>A for W<'a>{fn f(&self)->u8{*self.0}}fn g<T:A>(x:&T)->u8{A::f(x)}fn main(){let x=7u8;let y=g(&W(&x));check(y==7);}",
        false,
    ),
    (
        "dispatch-run-nested-function-value",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn pass<T>(x:T)->T{x}fn main(){let p=pass(A::f);let x=p(3u8);check(x==4);}",
        false,
    ),
    (
        "dispatch-run-array-function-value",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let p=[A::f;2];let x=p[1](3u8);check(x==4);}",
        false,
    ),
    (
        "dispatch-run-pointer-cast",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let p=A::f as fn(u8)->u8;let x=p(3u8);check(x==4);}",
        false,
    ),
    (
        "dispatch-run-constant-pointer",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}const P:fn(u8)->u8=A::f;fn main(){let x=P(3u8);check(x==4);}",
        false,
    ),
];

const MORE: &[(&str, &str, bool)] = &[
    (
        "dispatch-run-default-trait-argument",
        "trait A<T=u8>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=A::f(3u8,7u16);check(x==7);}",
        false,
    ),
    (
        "dispatch-run-explicit-trailing-default",
        "trait A<T,U=T>{fn f(self,x:T,y:U)->U;}impl A<u16>for u8{fn f(self,x:u16,y:u16)->u16{y}}fn main(){let x=A::<u16>::f(3u8,7,9);check(x==9);}",
        false,
    ),
    (
        "dispatch-run-explicit-default-self",
        "trait A<T,U=Self>{fn f(self,x:T,y:U)->U;}impl A<u16>for u8{fn f(self,x:u16,y:u8)->u8{y}}fn main(){let x=A::<u16>::f(3u8,7,9);check(x==9);}",
        false,
    ),
    (
        "dispatch-run-trait-const",
        "trait A<const N:usize>{fn f(self,x:[u8;N])->u8;}impl A<2>for u8{fn f(self,x:[u8;2])->u8{x[1]}}fn main(){let x=A::<2>::f(3u8,[7,9]);check(x==9);}",
        false,
    ),
    (
        "dispatch-run-trait-const-default",
        "trait A<T,const N:usize=2>{fn f(self,x:T,y:[u8;N])->u8;}impl A<u16>for u8{fn f(self,x:u16,y:[u8;2])->u8{y[1]}}fn main(){let x=A::<u16>::f(3u8,7,[8,9]);check(x==9);}",
        false,
    ),
    (
        "dispatch-run-trait-lifetime-explicit",
        "trait A<'a>{fn f(self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(self,x:&'a u8)->&'a u8{x}}fn g<'b>(x:&'b u8)->&'b u8{A::<'b>::f(3u8,x)}fn main(){let x=7u8;let y=g(&x);check(*y==7);}",
        false,
    ),
    (
        "dispatch-run-reordered-impl-arguments",
        "trait A<T>{fn f(self,x:T)->T;}struct W<T,U>(T,U);impl<V:Copy,U:Copy>A<U>for W<U,V>{fn f(self,x:U)->U{x}}fn main(){let x=A::f(W(3u8,4u16),7u8);check(x==7);}",
        false,
    ),
    (
        "dispatch-run-nested-reference-function-value",
        "trait A{fn f(self)->u8;}impl<'a>A for &'a u8{fn f(self)->u8{*self}}fn pass<T>(x:T)->T{x}fn g<T:A>(x:T)->u8{let p=pass(A::f);p(x)}fn main(){let v=7u8;let x=g(&v);check(x==7);}",
        false,
    ),
    (
        "dispatch-run-late-more-general-override",
        "trait A{fn f<'a>(self,x:&'a u8,y:&'a u8)->&'a u8;}impl A for u8{fn f<'b,'c>(self,x:&'b u8,y:&'c u8)->&'b u8{x}}fn main(){let a=7u8;let b=9u8;let p=A::f as for<'a>fn(u8,&'a u8,&'a u8)->&'a u8;let x=p(3,&a,&b);check(*x==7);}",
        false,
    ),
    (
        "dispatch-run-default-calls-overridden-default",
        "trait A:Sized{fn f(self)->u8{A::g(self)+1}fn g(self)->u8{3}}impl A for u8{fn g(self)->u8{self+2}}fn main(){let x=A::f(7u8);check(x==10);}",
        false,
    ),
    (
        "dispatch-run-nested-default-function-value",
        "trait A:Sized{fn f(self)->u8{let p=A::g;p(self)+1}fn g(self)->u8{3}}impl A for u8{fn g(self)->u8{self+2}}fn main(){let x=A::f(7u8);check(x==10);}",
        false,
    ),
    (
        "dispatch-run-static-pointer",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}static P:fn(u8)->u8=A::f;fn main(){let x=P(3u8);check(x==4);}",
        false,
    ),
    (
        "dispatch-run-nested-constant-pointer",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}const P:(fn(u8)->u8,[fn(u8)->u8;2])=(A::f,[A::f;2]);fn main(){let x=(P.1)[1](3u8);check(x==4);}",
        false,
    ),
    (
        "dispatch-run-constant-generic-impl",
        "trait A{fn f(self)->u8;}struct W<T>(T);impl<T:Copy>A for W<T>{fn f(self)->u8{7}}const P:fn(W<u8>)->u8=A::f;fn main(){let x=P(W(3u8));check(x==7);}",
        false,
    ),
    (
        "dispatch-run-constant-default",
        "trait A:Sized{fn f(self)->u8{7}}impl A for u8{}const P:fn(u8)->u8=A::f;fn main(){let x=P(3u8);check(x==7);}",
        false,
    ),
    (
        "dispatch-run-constant-borrowed-method",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self+1}}const P:fn(&u8)->u8=A::f;fn main(){let x=P(&3u8);check(x==4);}",
        false,
    ),
    (
        "dispatch-run-self-result-inference",
        "trait A{fn f()->Self;}impl A for u8{fn f()->Self{7}}fn main(){let x:u8=A::f();check(x==7);}",
        false,
    ),
];

#[test]
fn selected_trait_callables_lower() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in EXECUTIONS.iter().chain(MORE) {
        super::cases(&[(name, &format!("{}{source}", super::native::CHECK), true)])?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn selected_trait_callables_execute() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("trait-qualified-dispatch", EXECUTIONS)?;
    super::native::check_cases("trait-dispatch-owner-arguments", MORE)
}
