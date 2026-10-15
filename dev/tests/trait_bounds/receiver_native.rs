//! Native checks distinguish receiver search order and verify mutation of the source place.
const EXECUTIONS: &[(&str, &str, bool)] = &[
    (
        "receiver-default-recursion-run",
        "trait A:Sized{fn f(self,n:u8)->u8{if n==0{7}else{self.f(n-1)}}}impl A for u8{}fn main(){let x=3u8.f(2);check(x==7);}",
        false,
    ),
    (
        "receiver-returned-shared-borrow-run",
        "trait A{fn f(&self)->&u8;}struct S(u8);impl A for S{fn f(&self)->&u8{&self.0}}fn g<T:A>(x:&T)->&u8{x.f()}fn main(){let s=S(7);let x=g(&s);check(*x==7);}",
        false,
    ),
    (
        "receiver-returned-exclusive-borrow-run",
        "trait A{fn f(&mut self)->&mut u8;}struct S(u8);impl A for S{fn f(&mut self)->&mut u8{&mut self.0}}fn main(){let mut s=S(7);let r=s.f();*r=9;check(s.0==9);}",
        false,
    ),
    (
        "receiver-two-phase-argument-run",
        "trait A{fn get(&self)->u8;fn set(&mut self,x:u8);}impl A for u8{fn get(&self)->u8{*self}fn set(&mut self,x:u8){*self=x;}}fn main(){let mut n=7u8;n.set(n.get()+1);check(n==8);}",
        false,
    ),
    (
        "receiver-trait-capture-run",
        "trait A<'a>{fn f(self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(self,x:&'a u8)->&'a u8{x}}fn g<'a>(x:&'a u8)->&'a u8{3u8.f(x)}fn main(){let n=7u8;let r=g(&n);check(*r==7);}",
        false,
    ),
    (
        "receiver-method-const-run",
        "trait A{fn f<const N:usize>(self,x:[u8;N])->[u8;N];}impl A for u8{fn f<const N:usize>(self,x:[u8;N])->[u8;N]{x}}fn main(){let x=3u8.f::<2>([7,9]);check(x[0]==7&&x[1]==9);}",
        false,
    ),
    (
        "receiver-trait-type-inference-run",
        "trait A<T=u8>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=3u8.f(7u16);check(x==7);}",
        false,
    ),
    (
        "receiver-former-pa18-boundary",
        super::receiver_lookup::PA18_BOUNDARY,
        false,
    ),
    (
        "receiver-run-dot-primitive",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn main(){let x=3u8.f();check(x==4);}",
        false,
    ),
    (
        "receiver-run-dot-rigid",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn g<T:A>(x:T)->u8{x.f()}fn main(){let x=g(3u8);check(x==4);}",
        false,
    ),
    (
        "receiver-run-dot-supertrait",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}trait B:A{}impl B for u8{}fn g<T:B>(x:T)->u8{x.f()}fn main(){let x=g(3u8);check(x==4);}",
        false,
    ),
    (
        "receiver-run-dot-borrow",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn main(){let x=7u8.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-dot-reference-self",
        "trait A{fn f(self)->u8;}impl<'a>A for &'a u8{fn f(self)->u8{*self}}fn main(){let n=7u8;let x=(&n).f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-dot-reference-self-borrowed",
        "trait A{fn f(&self)->u8;}impl<'a>A for &'a u8{fn f(&self)->u8{**self}}fn main(){let n=7u8;let x=(&n).f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-dot-slice-unsize",
        "trait A{fn f(&self)->u8;}impl A for [u8]{fn f(&self)->u8{self[0]}}fn main(){let a=[7u8,9];let x=a.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-dot-anonymous-import",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}use m::A as _;fn main(){let x=7u8.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-dot-aliased-import",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}use m::A as Alias;fn main(){let x=7u8.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-dot-duplicate-trait-aliases",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}use m::A as X;use m::A as Y;fn main(){let x=7u8.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-dot-inherent-precedes-trait",
        "struct S;impl S{fn f(&self)->u8{1}}trait A{fn f(&self)->u8;}impl A for S{fn f(&self)->u8{2}}fn main(){let x=S.f();check(x==1);}",
        false,
    ),
    (
        "receiver-run-dot-shared-trait-precedes-mutable-inherent",
        "struct S;impl S{fn f(&mut self)->u8{1}}trait A{fn f(&self)->u8;}impl A for S{fn f(&self)->u8{2}}fn main(){let mut s=S;let x=s.f();check(x==2);}",
        false,
    ),
    (
        "receiver-run-dot-generic-method",
        "trait A{fn f<T>(self,x:T)->T;}impl A for u8{fn f<T>(self,x:T)->T{x}}fn main(){let x=3u8.f::<u16>(7);check(x==7);}",
        false,
    ),
    (
        "receiver-run-shadowed-trait-name",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}fn main(){struct A;let x=3u8.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-bound-trait-outside-scope",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}fn g<T:m::A>(x:T)->u8{x.f()}fn main(){let x=g(3u8);check(x==7);}",
        false,
    ),
    (
        "receiver-run-macro-call-site-trait",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}macro_rules! call{($x:expr)=>{$x.f()}}fn main(){use m::A;let x=call!(3u8);check(x==7);}",
        false,
    ),
    (
        "receiver-run-macro-use-visible-after-expansion",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}macro_rules! enable{()=>{use m::A as _;}}fn main(){enable!();let x=3u8.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-macro-local-trait-visible-after-expansion",
        "macro_rules! enable{()=>{trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}}fn main(){enable!();let x=3u8.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-ambiguous-glob-same-method",
        "mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}mod n{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{9}}}use m::*;use n::*;fn main(){let x=3u8.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-bound-priority",
        "trait A{fn f(self)->u8;}trait B{fn f(self)->u8;}impl<T>B for T{fn f(self)->u8{9}}impl A for u8{fn f(self)->u8{7}}fn g<T:A>(x:T)->u8{x.f()}fn main(){let x=g(3u8);check(x==7);}",
        false,
    ),
    (
        "receiver-run-reference-self-reborrows-mutable-reference",
        "trait A{fn f(self)->u8;}impl A for &mut u8{fn f(self)->u8{*self}}fn main(){let mut n=7u8;let r=&mut n;let x=r.f();*r=9;check(x==7&&*r==9);}",
        false,
    ),
    (
        "receiver-run-borrowed-receiver-reborrows-mutable-reference",
        "trait A{fn f(&mut self)->u8;}impl A for u8{fn f(&mut self)->u8{*self}}fn main(){let mut n=7u8;let r=&mut n;let x=r.f();*r=9;check(x==7&&*r==9);}",
        false,
    ),
    (
        "receiver-run-reference-self-borrowed-reborrows-outer-reference",
        "trait A{fn f(&mut self)->u8;}impl A for &mut u8{fn f(&mut self)->u8{**self}}fn main(){let mut n=7u8;let mut r=&mut n;let rr=&mut r;let x=rr.f();**rr=9;check(x==7&&**rr==9);}",
        false,
    ),
    (
        "receiver-run-hidden-bound-autoref",
        "mod m{pub trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}}fn g<T:m::A>(x:T)->u8{x.f()}fn main(){let x=g(7u8);check(x==7);}",
        false,
    ),
    (
        "receiver-run-hidden-bound-reference-receiver",
        "mod m{pub trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}}fn g<T:m::A>(x:&T)->u8{x.f()}fn main(){let x=g(&7u8);check(x==7);}",
        false,
    ),
    (
        "receiver-run-bound-shared-priority",
        "trait A{fn f(&self)->u8;}trait B{fn f(self)->u8;}impl<T>B for &T{fn f(self)->u8{9}}impl A for u8{fn f(&self)->u8{7}}fn g<T:A>(x:T)->u8{x.f()}fn main(){let x=g(7u8);check(x==7);}",
        false,
    ),
    (
        "receiver-run-double-reference-receiver",
        "trait A{fn f(self:&&Self)->u8;}impl A for u8{fn f(self:&&Self)->u8{**self}}fn main(){let n=7u8;let x=(&n).f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-numeric-receiver-inferred-from-single-impl",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}fn main(){let x=7.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-numeric-explicit-receiver",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}fn main(){let x=7u8.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-array-unsize-borrow",
        "trait A{fn f(&self)->u8;}impl A for [u8]{fn f(&self)->u8{self[0]}}fn main(){let x=[7u8,9].f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-array-unsize-mutable",
        "trait A{fn f(&mut self)->u8;}impl A for [u8]{fn f(&mut self)->u8{self[0]=9;self[0]}}fn main(){let mut a=[7u8,3];let x=a.f();check(x==9&&a[0]==9&&a[1]==3);}",
        false,
    ),
    (
        "receiver-run-slice-reference-self",
        "trait A{fn f(self)->u8;}impl A for &[u8]{fn f(self)->u8{self[0]}}fn main(){let x=[7u8,9].f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-numeric-default-selects-i32",
        "trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}impl A for i32{fn f(self)->u8{9}}fn main(){let x=7.f();check(x==9);}",
        false,
    ),
    (
        "receiver-run-float-unique-implementation",
        "trait A{fn f(self)->u8;}impl A for f32{fn f(self)->u8{7}}fn main(){let x=7.0.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-explicit-self-reference-value",
        "trait A{fn f(self)->u8;}impl A for &u8{fn f(self)->u8{*self}}fn main(){let n=7u8;let x=n.f();check(x==7);}",
        false,
    ),
    (
        "receiver-run-trait-inherent-alias-fallback",
        "trait A{fn f(self)->u8;}struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->u8{7}}impl<T>A for S<T>{fn f(self)->u8{9}}struct N;fn main(){let x=S(N).f();check(x==9);}",
        false,
    ),
];
#[test]
fn receiver_calls_lower() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in EXECUTIONS {
        super::cases(&[(name, &format!("{}{source}", super::native::CHECK), true)])?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn receiver_calls_execute() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("trait-receiver-calls", EXECUTIONS)
}
