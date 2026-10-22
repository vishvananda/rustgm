//! Receiver calls retain selected callable identity, source evaluation and ownership proofs.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
use rustgm::resolve::model::{DefId, DefKind};
const ACCEPT: &[(&str, &str, bool)] = &[
    (
        "former-pa17-boundary",
        "struct D;impl Drop for D{fn drop(&mut self){}}impl D{fn touch(&self){}}fn main(){let d=D;d.touch();}\n",
        false,
    ),
    (
        "former-pa18-boundary",
        "struct S<T>(T);impl<T:Copy> S<T>{fn get(&self)->T{self.0}}fn main(){let _=S(7u8).get();}\n",
        false,
    ),
    (
        "known-before-call",
        "struct S<T>(T);impl S<u8>{fn f(self)->u8{7}}impl S<u16>{fn f(self)->u8{9}}fn main(){check(S(1u8).f()==7);}",
        false,
    ),
    (
        "generic-argument-infers",
        "struct S<T>(T);impl<T>S<T>{fn f(&self,x:T)->T{x}}fn main(){let s=S(1);check(s.f(7u8)==7);}",
        false,
    ),
    (
        "generic-result-infers",
        "struct S<T>(T);impl<T>S<T>{fn f(self)->T{self.0}}fn main(){let s=S(7);let x:u8=s.f();check(x==7);}",
        false,
    ),
    (
        "single-concrete-infers",
        "struct S<T>(T);impl S<u8>{fn f(self)->u8{self.0}}fn main(){check(S(7).f()==7);}",
        false,
    ),
    (
        "nested-receiver-from-reference",
        "struct S(u8);impl S{fn f(self:&&Self)->u8{self.0}}fn main(){let s=S(7);check((&s).f()==7);}",
        false,
    ),
    (
        "mutable-reference-reborrow",
        "struct S(u8);impl S{fn get(&self)->u8{self.0}fn set(&mut self,x:u8){self.0=x;}}fn main(){let mut s=S(7);let r=&mut s;r.set(r.get()+1);check(s.0==8);}",
        false,
    ),
    (
        "implicit-borrow-is-two-phase",
        "struct S(u8);impl S{fn get(&self)->u8{self.0}fn set(&mut self,x:u8){self.0=x;}}fn main(){let mut s=S(7);s.set(s.get()+1);check(s.0==8);}",
        false,
    ),
    (
        "by-value-copy-through-reference",
        "#[derive(Copy,Clone)]struct S(u8);impl S{fn get(self)->u8{self.0}}fn main(){let s=S(7);let r=&&s;check(r.get()==7);check(s.0==7);}",
        false,
    ),
    (
        "lifetime-receiver-return",
        "struct S(u8);impl S{fn get(&self)->&u8{&self.0}}fn main(){let s=S(7);let r=s.get();check(*r==7);}",
        false,
    ),
    (
        "method-type-and-const-arguments",
        "struct S;impl S{fn f<T:Copy,const N:usize>(&self,x:T)->[T;N]{[x;N]}}fn main(){check(S.f::<u8,2>(7)[1]==7);}",
        false,
    ),
    (
        "const-argument-from-caller",
        "struct S;impl S{fn f<const N:usize>(&self)->[u8;N]{[7;N]}}fn g<const N:usize>()->[u8;N]{S.f::<N>()}fn main(){check(g::<2>()[1]==7);}",
        false,
    ),
    (
        "single-evaluation-reference",
        "struct S(u8);impl S{fn get(&self)->u8{self.0}}fn take<'a>(s:&'a S,n:&mut u8)->&'a S{*n+=1;s}fn main(){let s=S(7);let mut n=0u8;check(take(&s,&mut n).get()==7);check(n==1);}",
        false,
    ),
    (
        "inherited-and-own-generics",
        "struct S<T,const N:usize>([T;N]);impl<T:Copy,const N:usize>S<T,N>{fn f<U>(&self,x:U)->(T,U){(self.0[N-1],x)}}fn main(){let s=S([2u8,7]);let p=s.f::<u16>(517);check(p.0==7&&p.1==517);}",
        false,
    ),
    (
        "own-type-hole",
        "struct S;impl S{fn f<T:Copy,const N:usize>(&self,x:T)->[T;N]{[x;N]}}fn main(){check(S.f::<_,3>(7u8)[2]==7);}",
        false,
    ),
    (
        "named-const",
        "const N:usize=2;struct S;impl S{fn f<const N:usize>(&self)->[u8;N]{[7;N]}}fn main(){check(S.f::<N>()[1]==7);}",
        false,
    ),
    (
        "const-block",
        "struct S;impl S{fn f<const N:usize>(&self)->[u8;N]{[7;N]}}fn main(){check(S.f::<{1usize+1}>()[1]==7);}",
        false,
    ),
    (
        "early-method-lifetime",
        "struct S;impl S{fn f<'a:'a>(&self,x:&'a u8)->&'a u8{x}}fn main(){let n=7u8;check(*S.f::<'_>(&n)==7);}",
        false,
    ),
    (
        "captured-impl-lifetime",
        "struct S<'a>(&'a u8);impl<'a>S<'a>{fn f(&self)->&'a u8{self.0}}fn g<'a>(x:&'a u8)->&'a u8{S(x).f()}fn main(){let n=7u8;check(*g(&n)==7);}",
        false,
    ),
    (
        "mutable-result",
        "struct S(u8);impl S{fn f(&mut self)->&mut u8{&mut self.0}}fn main(){let mut s=S(7);*s.f()=9;check(s.0==9);}",
        false,
    ),
    (
        "mutable-temporary",
        "struct S(u8);impl S{fn f(&mut self,x:u8)->u8{self.0+=x;self.0}}fn main(){check(S(7).f(2)==9);}",
        false,
    ),
    (
        "generic-self-call",
        "struct S<T>(T);impl<T:Copy>S<T>{fn f(&self)->T{self.0}fn g(&self)->T{self.f()}}fn main(){check(S(7u8).g()==7);}",
        false,
    ),
    (
        "disjoint-const-impls",
        "struct S<const N:usize>;impl S<1>{fn f(&self)->u8{7}}impl S<2>{fn f(&self)->u8{9}}fn main(){check(S::<1>.f()==7);check(S::<2>.f()==9);}",
        false,
    ),
    (
        "drop-shared-receiver",
        "struct S<'a>(u8,&'a mut u8);impl Drop for S<'_>{fn drop(&mut self){*self.1+=self.0;}}impl S<'_>{fn f(&self)->u8{self.0}}fn main(){let mut n=0u8;{let s=S(7,&mut n);check(s.f()==7);}check(n==7);}",
        false,
    ),
    (
        "drop-value-receiver",
        "struct S<'a>(u8,&'a mut u8);impl Drop for S<'_>{fn drop(&mut self){*self.1+=self.0;}}impl S<'_>{fn f(self)->u8{self.0}}fn main(){let mut n=0u8;{let s=S(7,&mut n);check(s.f()==7);}check(n==7);}",
        false,
    ),
    (
        "function-type-argument",
        "struct S;impl S{fn f<T>(&self,x:T)->T{x}}fn id(x:&u8)->&u8{x}fn main(){let p=S.f::<for<'a> fn(&'a u8)->&'a u8>(id);let n=7u8;check(*p(&n)==7);}",
        false,
    ),
    (
        "partial-reference-argument",
        "struct S;impl S{fn f<T>(&self,x:T)->T{x}}fn main(){let n=7u8;let p=S.f::<(&_,_)>((&n,517u16));check(*p.0==7&&p.1==517);}",
        false,
    ),
    (
        "bool-char-consts",
        "struct S;impl S{fn f<const B:bool,const C:char>(&self)->(bool,char){(B,C)}}fn main(){let p=S.f::<true,'x'>();check(p.0&&p.1=='x');}",
        false,
    ),
    (
        "nested-call-two-phase",
        "struct S(u8);impl S{fn f(&mut self,x:u8){self.0=x;}fn g(&self)->u8{self.0}}fn main(){let mut s=S(7);s.f({let x=s.g();x+1});check(s.0==8);}",
        false,
    ),
    (
        "alias-type-argument",
        "type A<T>=(u8,T);struct S;impl S{fn f<T>(&self,x:T)->T{x}}fn main(){let p=S.f::<A<_>>((7,517u16));check(p.0==7&&p.1==517);}",
        false,
    ),
    (
        "reference-const-array",
        "struct S;impl S{fn f<T:Copy,const N:usize>(&self,x:&[T;N])->T{x[N-1]}}fn main(){check(S.f::<u16,_>(&[2,517])==517);}",
        false,
    ),
    (
        "generic-lifetime-receiver-value",
        "struct S<'a,T>(&'a T);impl<'a,T>S<'a,T>{fn get(self)->&'a T{self.0}}fn value<'a,T>(x:S<'a,T>)->&'a T{x.get()}fn main(){let n=7u8;check(*value(S(&n))==7);}",
        false,
    ),
    (
        "generic-lifetime-copy-receiver",
        "#[derive(Copy,Clone)]struct S<'a,T>(&'a T);impl<'a,T>S<'a,T>{fn get(self)->&'a T{self.0}}fn value<'a,T:Copy>(x:&S<'a,T>)->&'a T{x.get()}fn main(){let n=7u8;let s=S(&n);check(*value(&s)==7);check(*s.0==7);}",
        false,
    ),
];
const REJECT: &[(&str, &str, bool)] = &[
    (
        "unknown-integer-before-fallback",
        "struct S<T>(T);impl S<u8>{fn f(self)->u8{7}}impl S<u16>{fn f(self)->u8{9}}fn main(){let s=S(1);s.f();}",
        false,
    ),
    (
        "late-local-annotation",
        "struct S<T>(T);impl S<u8>{fn f(&self)->u8{7}}impl S<u16>{fn f(&self)->u8{9}}fn main(){let s=S(1);s.f();let _:S<u8>=s;}",
        false,
    ),
    (
        "argument-does-not-select",
        "struct S<T>(T);impl S<u8>{fn f(&self,x:u8)->u8{x}}impl S<u16>{fn f(&self,x:u16)->u16{x}}fn main(){let s=S(1);s.f(7u8);}",
        false,
    ),
    (
        "nested-receiver-no-double-autoref",
        "struct S(u8);impl S{fn f(self:&&Self)->u8{self.0}}fn main(){S(7).f();}",
        false,
    ),
    (
        "explicit-borrow-is-immediate",
        "struct S(u8);impl S{fn get(&self)->u8{self.0}fn set(&mut self,x:u8){self.0=x;}}fn main(){let mut s=S(7);(&mut s).set(s.get());}",
        false,
    ),
    (
        "immutable-selected-mut-receiver",
        "struct S(u8);impl S{fn set(&mut self,x:u8){self.0=x;}}fn main(){let s=S(7);s.set(8);}",
        false,
    ),
    (
        "shared-chain-cannot-mutate",
        "struct S(u8);impl S{fn set(&mut self,x:u8){self.0=x;}}fn main(){let s=S(7);let r=&&s;r.set(8);}",
        false,
    ),
    (
        "by-value-move-through-reference",
        "struct S(u8);impl S{fn get(self)->u8{self.0}}fn main(){let s=S(7);let r=&s;r.get();}",
        false,
    ),
    (
        "associated-function-is-not-method",
        "struct S;impl S{fn f()->u8{7}}fn main(){S.f();}",
        false,
    ),
    (
        "lifetime-receiver-still-borrowed",
        "struct S(u8);impl S{fn get(&self)->&u8{&self.0}fn set(&mut self,x:u8){self.0=x;}}fn main(){let mut s=S(7);let r=s.get();s.set(8);check(*r==7);}",
        false,
    ),
    (
        "method-generic-arity",
        "struct S;impl S{fn f<T>(&self,x:T)->T{x}}fn main(){S.f::<u8,u16>(7);}",
        false,
    ),
    (
        "const-kind-error",
        "struct S;impl S{fn f<const N:usize>(&self){}}fn main(){S.f::<u8>();}",
        false,
    ),
    (
        "type-kind-error",
        "struct S;impl S{fn f<T>(&self){}}fn main(){S.f::<2>();}",
        false,
    ),
    (
        "wrong-const-type",
        "struct S;impl S{fn f<const N:usize>(&self){}}fn main(){S.f::<2u8>();}",
        false,
    ),
    (
        "own-bound-error",
        "struct N;struct S;impl S{fn f<T:Copy>(&self,x:T){}}fn main(){S.f::<N>(N);}",
        false,
    ),
    (
        "caller-bound-error",
        "struct S<T>(T);impl<T:Copy>S<T>{fn f(&self){}}fn g<T>(x:S<T>){x.f();}fn main(){}",
        false,
    ),
    (
        "packed-receiver",
        "struct S(u32);impl S{fn f(&self)->u32{self.0}}#[repr(packed)]struct P{a:u8,s:S}fn main(){let p=P{a:0,s:S(7)};p.s.f();}",
        false,
    ),
    (
        "private-receiver",
        "mod m{pub struct S;impl S{fn f(&self){}}}fn main(){m::S.f();}",
        false,
    ),
    (
        "outer-generic-const-capture",
        "struct S;impl S{fn f<const N:usize>(&self){}}fn g<const N:usize>(){S.f::<{N+1}>();}fn main(){}",
        false,
    ),
    (
        "nested-mutable-borrow",
        "struct S(u8);impl S{fn f(&mut self,x:u8){self.0=x;}fn g(&mut self)->u8{self.0+=1;self.0}}fn main(){let mut s=S(7);s.f(s.g());}",
        false,
    ),
    (
        "shared-before-argument-mutation",
        "struct S(u8);impl S{fn f(&self,x:u8)->u8{self.0+x}}fn main(){let mut s=S(7);s.f({s.0=8;1});}",
        false,
    ),
    (
        "ill-formed-own-type",
        "struct W<T:Copy>(T);struct N;struct S;impl S{fn f<T>(&self){}}fn main(){S.f::<W<N>>();}",
        false,
    ),
    (
        "unconstrained-own-type",
        "struct S;impl S{fn f<T>(&self){}}fn main(){S.f();}",
        false,
    ),
    (
        "unconstrained-own-const",
        "struct S;impl S{fn f<const N:usize>(&self){}}fn main(){S.f::<_>();}",
        false,
    ),
];

fn prove(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(source)?;
    let main = report
        .resolution
        .definitions
        .iter()
        .position(|d| d.kind == DefKind::Function && report.package.names.get(d.name) == "main")
        .ok_or("missing main")?;
    rustgm::lower::instances::build(&report, &mut facts, DefId(u32::try_from(main)?))?;
    Ok(())
}
#[test]
fn receiver_calls_preserve_inference_and_place_conversions()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in ACCEPT {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(&format!("dot-method-{name}"), &source, true)?;
        prove(&source).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn selection_and_borrow_errors_remain_distinct_rejections() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, source, _) in REJECT {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(&format!("dot-method-{name}"), &source, false)?;
        assert!(prove(&source).is_err(), "{name}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn receiver_calls_execute_against_rustc() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases("dot-methods", ACCEPT)
}

#[test]
fn general_constant_statements_remain_a_later_assignment() -> Result<(), Box<dyn std::error::Error>>
{
    let source = "struct S;impl S{fn f<const N:usize>(&self)->[u8;N]{[7;N]}}fn main(){let _=S.f::<{let n=1usize;n+1}>();}";
    rustc::qualify("dot-method-general-constant", source, true)?;
    let error = prove(source)
        .err()
        .ok_or("general constant statements crossed their assignment boundary")?;
    assert!(
        error
            .to_string()
            .contains("constant statements require the general constant evaluation assignment")
    );
    Ok(())
}

#[test]
fn closed_receiver_buckets_skip_other_impls() -> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    let mut source = "struct S<const N:usize>;".to_owned();
    for index in 0..256 {
        write!(source, "impl S<{index}>{{fn f(&self){{}}}}")?;
    }
    source.push_str("fn main(){");
    for index in 0..256 {
        write!(source, "S::<{index}>.f();")?;
    }
    source.push('}');
    rustc::qualify("dot-method-index", &source, true)?;
    let (_, facts) = semantic::analyze(&source)?;
    assert_eq!(facts.methods.calls.len(), 256);
    assert_eq!(facts.methods.selection_probes, 256);
    assert!(facts.methods.selection_steps < 256 * 80);
    Ok(())
}
#[test]
fn reference_depth_filters_before_copying_inference_graphs()
-> Result<(), Box<dyn std::error::Error>> {
    let depth = 96;
    let mut source = "struct S;impl S{fn f(&self){}}fn g(x:".to_owned();
    source.push_str(&"&".repeat(depth));
    source.push_str("S){x.f();}fn main(){}");
    rustc::qualify("dot-method-reference-depth", &source, true)?;
    let (_, facts) = semantic::analyze(&source)?;
    assert_eq!(facts.methods.selection_probes, 1);
    assert!(facts.methods.selection_steps < 80);
    Ok(())
}
