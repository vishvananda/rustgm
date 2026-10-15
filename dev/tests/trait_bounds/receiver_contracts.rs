//! Selected trait receivers retain default-body and ordinary lifetime/loan proofs.
use super::cases;

#[test]
fn borrowed_results_keep_their_source_contract() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "receiver-returned-shared-borrow",
            "trait A{fn f(&self)->&u8;}struct S(u8);impl A for S{fn f(&self)->&u8{&self.0}}fn g<T:A>(x:&T)->&u8{x.f()}fn main(){let s=S(7);let x=g(&s);}",
            true,
        ),
        (
            "receiver-returned-borrow-cannot-escape",
            "trait A{fn f(&self)->&u8;}struct S(u8);impl A for S{fn f(&self)->&u8{&self.0}}fn g()->&'static u8{let s=S(7);s.f()}fn main(){}",
            false,
        ),
        (
            "receiver-returned-exclusive-borrow",
            "trait A{fn f(&mut self)->&mut u8;}struct S(u8);impl A for S{fn f(&mut self)->&mut u8{&mut self.0}}fn main(){let mut s=S(7);let r=s.f();*r=9;}",
            true,
        ),
        (
            "receiver-exclusive-borrow-conflict",
            "trait A{fn f(&mut self)->&mut u8;}struct S(u8);impl A for S{fn f(&mut self)->&mut u8{&mut self.0}}fn main(){let mut s=S(7);let r=s.f();s.0=9;let x=*r;}",
            false,
        ),
    ])
}

#[test]
fn defaults_and_callable_arguments_share_normal_inference() -> Result<(), Box<dyn std::error::Error>>
{
    cases(&[
        (
            "receiver-default-recursion",
            "trait A:Sized{fn f(self,n:u8)->u8{if n==0{7}else{self.f(n-1)}}}impl A for u8{}fn main(){let x=3u8.f(2);}",
            true,
        ),
        (
            "receiver-two-phase-argument",
            "trait A{fn get(&self)->u8;fn set(&mut self,x:u8);}impl A for u8{fn get(&self)->u8{*self}fn set(&mut self,x:u8){*self=x;}}fn main(){let mut n=7u8;n.set(n.get()+1);}",
            true,
        ),
        (
            "receiver-trait-capture",
            "trait A<'a>{fn f(self,x:&'a u8)->&'a u8;}impl<'a>A<'a>for u8{fn f(self,x:&'a u8)->&'a u8{x}}fn g<'a>(x:&'a u8)->&'a u8{3u8.f(x)}fn main(){let n=7u8;let r=g(&n);}",
            true,
        ),
        (
            "receiver-method-const",
            "trait A{fn f<const N:usize>(self,x:[u8;N])->[u8;N];}impl A for u8{fn f<const N:usize>(self,x:[u8;N])->[u8;N]{x}}fn main(){let x=3u8.f::<2>([7,9]);}",
            true,
        ),
        (
            "receiver-trait-type-inference",
            "trait A<T=u8>{fn f(self,x:T)->T;}impl A<u16>for u8{fn f(self,x:u16)->u16{x}}fn main(){let x=3u8.f(7u16);}",
            true,
        ),
        (
            "receiver-call-own-bound-ambiguity",
            "trait A{fn f<T:Copy>(self,x:T);}trait B{fn f<T>(self,x:T);}impl A for u8{fn f<T:Copy>(self,x:T){}}impl B for u8{fn f<T>(self,x:T){}}struct N;fn main(){3u8.f(N);}",
            false,
        ),
    ])
}
