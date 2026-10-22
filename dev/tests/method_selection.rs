//! Prefix selection preserves caller rigidity, unknown aliases and ordinary lowering.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
use rustgm::resolve::model::{DefId, DefKind};
use std::fmt::Write;
const ACCEPT: &[(&str, &str, bool)] = &[
    (
        "concrete-types",
        "struct S<T>(T);impl S<u8>{fn f(self)->u16{self.0 as u16+1}}impl S<u16>{fn f(self)->u16{self.0+2}}fn main(){check(S::<u8>::f(S(7))==8);check(S::<u16>::f(S(517))==519);}",
        false,
    ),
    (
        "explicit-type",
        "struct S<T>(T);impl S<u8>{fn f(self)->u16{self.0 as u16+1}}impl S<u16>{fn f(self)->u16{self.0+2}}fn main(){let f:fn(S<u16>)->u16=<S<u16>>::f;check(f(S(517))==519);}",
        false,
    ),
    (
        "partial-target",
        "struct S<T,U>(T,U);impl<U>S<u8,U>{fn f(self)->U{self.1}}impl<U>S<u16,U>{fn f(self)->U{self.1}}fn main(){check(S::<u8,_>::f(S(7,517u16))==517);}",
        false,
    ),
    (
        "alias-partial",
        "struct S<T,U>(T,U);type A<T>=S<u8,T>;impl<U>S<u8,U>{fn f(self)->U{self.1}}impl<U>S<u16,U>{fn f(self)->U{self.1}}fn main(){check(A::f(S(7,517u16))==517);}",
        false,
    ),
    (
        "alias-default-explicit",
        "struct S<T>(T);type A<T=u8>=S<T>;impl S<u8>{fn f()->u16{7}}impl S<u16>{fn f()->u16{517}}fn main(){check(<A>::f()==7);check(A::<u16>::f()==517);}",
        false,
    ),
    (
        "repeated-type-hole",
        "struct S<T,U>(T,U);type A<T>=S<T,T>;impl S<u8,u8>{fn f()->u8{7}}impl S<u8,u16>{fn f()->u8{9}}fn main(){check(A::<_>::f()==7);}",
        false,
    ),
    (
        "repeated-const-hole",
        "struct S<const N:usize,const M:usize>;type A<const N:usize>=S<N,N>;impl S<1,1>{fn f()->u8{7}}impl S<2,3>{fn f()->u8{9}}fn main(){check(A::<_>::f()==7);}",
        false,
    ),
    (
        "const-partial",
        "struct S<const N:usize,const M:usize>([u8;N]);impl<const N:usize>S<N,1>{fn f(self)->u8{self.0[N-1]}}impl<const N:usize>S<N,2>{fn f(self)->u8{self.0[0]}}fn main(){check(S::<_,1>::f(S([2,7]))==7);}",
        false,
    ),
    (
        "array-term",
        "struct S<T>(T);impl<const N:usize>S<[u8;N]>{fn f(self)->u8{self.0[N-1]}}impl S<u8>{fn f(self)->u8{self.0}}fn main(){check(S::<[u8;2]>::f(S([2,7]))==7);}",
        false,
    ),
    (
        "copy-and-local",
        "struct N(u8);struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->u8{1}}impl S<N>{fn f(self)->u8{self.0.0}}fn main(){check(S::<u8>::f(S(5))==1);check(S::<N>::f(S(N(7)))==7);}",
        false,
    ),
    (
        "caller-copy",
        "struct N;struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->T{self.0}}impl S<N>{fn f(self)->N{self.0}}fn g<T:Copy>(x:T)->T{S::<T>::f(S(x))}fn main(){check(g(7u8)==7);}",
        false,
    ),
    (
        "caller-composite-copy",
        "struct N;struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->T{self.0}}impl S<N>{fn f(self)->N{self.0}}fn g<T,U>(x:(T,U))->(T,U)where (T,U):Copy{S::<(T,U)>::f(S(x))}fn main(){check(g((7u8,9u16)).0==7);}",
        false,
    ),
    (
        "nested-copy-hole",
        "struct N;struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->T{self.0}}impl S<N>{fn f(self)->N{self.0}}fn main(){check(S::<(u8,_)>::f(S((7,9u16))).1==9);}",
        false,
    ),
    (
        "local-nonderived-hole",
        "struct N<T>(T);struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->u8{1}}impl<T>S<N<T>>{fn f(self)->u8{7}}fn main(){check(S::<N<_>>::f(S(N(9u16)))==7);}",
        false,
    ),
    (
        "sized-and-str",
        "struct S<T:?Sized>(*const T);impl<T>S<T>{fn f()->u8{1}}impl S<str>{fn f()->u8{7}}fn main(){check(S::<u8>::f()==1);check(S::<str>::f()==7);}",
        false,
    ),
    (
        "clone-and-local",
        "struct N(u8);struct S<T>(T);impl<T:Clone>S<T>{fn f(self)->u8{1}}impl S<N>{fn f(self)->u8{self.0.0}}fn main(){check(S::<u8>::f(S(3))==1);check(S::<N>::f(S(N(7)))==7);}",
        false,
    ),
    (
        "own-generics",
        "struct S<T>(T);impl S<u8>{fn f<U>(self,x:U)->U{x}}impl S<u16>{fn f(self)->u16{self.0}}fn main(){check(S::<u8>::f::<u16>(S(7),517)==517);check(S::<u16>::f(S(17))==17);}",
        false,
    ),
    (
        "captured-reference",
        "struct S<T>(T);impl<'a>S<&'a u8>{fn f(self)->&'a u8{self.0}}impl S<u8>{fn f(self)->u8{self.0}}fn main(){let x=7;check(*S::<&u8>::f(S(&x))==7);}",
        false,
    ),
    (
        "mutable-receiver",
        "struct S<T>(T);impl S<u8>{fn set(&mut self){self.0=7;}}impl S<u16>{fn set(&mut self){self.0=517;}}fn main(){let mut s=S(1u16);S::<u16>::set(&mut s);check(s.0==517);}",
        false,
    ),
    (
        "self-bound",
        "struct N;struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->T{self.0}fn g(self)->T{Self::f(self)}}impl S<N>{fn f(self)->N{self.0}}fn main(){check(S::<u8>::g(S(7))==7);}",
        false,
    ),
    (
        "function-hole",
        "struct S<T>(T);impl S<fn(u8)->u16>{fn f()->u8{7}}impl S<fn(u16)->u8>{fn f()->u8{9}}fn main(){check(S::<fn(u8)->_>::f()==7);}",
        false,
    ),
    (
        "pointer-hole",
        "struct S<T>(T);impl S<*const u8>{fn f()->u8{7}}impl S<*mut u8>{fn f()->u8{9}}fn main(){check(S::<*const _>::f()==7);}",
        false,
    ),
    (
        "failed-first-type",
        "struct S<T,U>(T,U);type A<T>=S<T,T>;impl S<u8,u16>{fn f()->u8{9}}impl S<u8,u8>{fn f()->u8{7}}fn main(){check(A::<_>::f()==7);}",
        false,
    ),
    (
        "failed-first-const",
        "struct S<const N:usize,const M:usize>;type A<const N:usize>=S<N,N>;impl S<2,3>{fn f()->u8{9}}impl S<1,1>{fn f()->u8{7}}fn main(){check(A::<_>::f()==7);}",
        false,
    ),
    (
        "nested-repeated-hole",
        "struct S<T>(T);type A<T>=S<(T,T)>;impl S<(u8,u16)>{fn f()->u8{9}}impl S<(u8,u8)>{fn f()->u8{7}}fn main(){check(A::<_>::f()==7);}",
        false,
    ),
    (
        "rigid-const-caller",
        "struct S<const N:usize,const M:usize>;impl<const N:usize>S<N,1>{fn f()->usize{1}}impl<const N:usize>S<N,2>{fn f()->usize{N}}fn g<const N:usize>()->usize{S::<N,2>::f()}fn main(){check(g::<7>()==7);}",
        false,
    ),
    (
        "private-unrelated",
        "mod m{pub struct S<T>(pub T);impl S<u8>{pub fn f()->u8{7}}impl S<u16>{fn f()->u8{9}}}fn main(){check(m::S::<u8>::f()==7);}",
        false,
    ),
    (
        "private-unknown",
        "mod m{pub struct S<T>(pub T);impl S<u8>{pub fn f()->u8{7}}impl S<u16>{fn f()->u8{9}}}fn main(){check(m::S::f()==7);}",
        false,
    ),
    (
        "mutable-local-hole",
        "struct N<T>(T);struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->u8{1}}impl<'a,T>S<&'a mut N<T>>{fn f(self)->u8{7}}fn main(){let mut n=N(9u16);check(S::<&mut N<_>>::f(S(&mut n))==7);}",
        false,
    ),
    (
        "conditional-derive-hole",
        "struct N<T>(T);#[derive(Clone,Copy)]struct D<T>(T);struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->u8{1}}impl<T>S<D<N<T>>>{fn f(self)->u8{7}}fn main(){check(S::<D<N<_>>>::f(S(D(N(9u16))))==7);}",
        false,
    ),
    (
        "imported-alias",
        "mod m{pub struct S<T>(pub T);pub type A<T=u8>=S<T>;impl S<u8>{pub fn f()->u8{7}}impl S<u16>{pub fn f()->u8{9}}}use m::A;fn main(){check(<A>::f()==7);check(A::<u16>::f()==9);}",
        false,
    ),
];
const REJECT: &[(&str, &str)] = &[
    (
        "argument-not-discriminator",
        "struct S<T>(T);impl S<u8>{fn f(self)->u8{7}}impl S<u16>{fn f(self)->u8{9}}fn main(){let _=S::f(S(1u8));}",
    ),
    (
        "result-not-discriminator",
        "struct S<T>(T);impl S<u8>{fn f()->u8{7}}impl S<u16>{fn f()->u16{9}}fn main(){let _:u8=S::f();}",
    ),
    (
        "hole-result-not-discriminator",
        "struct S<T>(T);impl S<u8>{fn f()->u8{7}}impl S<u16>{fn f()->u16{9}}fn main(){let _:u8=S::<_>::f();}",
    ),
    (
        "alias-default-infers",
        "struct S<T>(T);type A<T=u8>=S<T>;impl S<u8>{fn f()->u8{7}}impl S<u16>{fn f()->u8{9}}fn main(){let _=A::f();}",
    ),
    (
        "own-generics-not-discriminator",
        "struct S<T>(T);impl S<u8>{fn f<U>(){}}impl S<u16>{fn f(){}}fn main(){S::f::<u8>();}",
    ),
    (
        "caller-needs-copy",
        "struct N;struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->T{self.0}}impl S<N>{fn f(self)->N{self.0}}fn g<T>(x:T)->T{S::<T>::f(S(x))}fn main(){}",
    ),
    (
        "unknown-rigid-cannot-be-local",
        "struct N;struct S<T>(T);impl S<u8>{fn f(self)->u8{1}}impl S<N>{fn f(self)->u8{7}}fn g<T>(x:T)->u8{S::<T>::f(S(x))}fn main(){}",
    ),
    (
        "missing-target",
        "struct S<T>(T);impl S<u8>{fn f(){}}impl S<u16>{fn f(){}}fn main(){S::<u32>::f();}",
    ),
    (
        "method-bound-after-selection",
        "struct N;struct S<T>(T);impl S<u8>{fn f<T:Copy>(x:T){}}impl S<u16>{fn f<T>(x:T){}}fn main(){S::<u8>::f(N);}",
    ),
    (
        "current-copy-mutable-hole",
        "struct N;struct S<T>(T);impl<T:Copy>S<T>{fn f(self){}}impl S<N>{fn f(self){}}fn main(){let mut x=7u8;S::<(&mut _,)>::f(S((&mut x,)));}",
    ),
    (
        "independent-const-holes",
        "struct S<const N:usize,const M:usize>;impl S<1,1>{fn f(){}}impl S<2,3>{fn f(){}}fn main(){S::<_,_>::f();}",
    ),
    (
        "function-argument-hole",
        "struct S<T>(T);impl S<fn(u8)>{fn f(){}}impl S<fn(u16)>{fn f(){}}fn main(){S::<fn(_)>::f();}",
    ),
    (
        "private-known",
        "mod m{pub struct S<T>(pub T);impl S<u8>{pub fn f()->u8{7}}impl S<u16>{fn f()->u8{9}}}fn main(){let _=m::S::<u16>::f();}",
    ),
    (
        "private-in-owner-scope",
        "mod m{pub struct S<T>(pub T);impl S<u8>{pub fn f()->u8{7}}impl S<u16>{fn f()->u8{9}}pub fn g(){let _=S::f();}}fn main(){m::g();}",
    ),
    (
        "caller-environment-isolation",
        "struct N;struct S<T>(T);impl<T:Copy>S<T>{fn f(self)->T{self.0}}impl S<N>{fn f(self)->N{self.0}}fn good<T:Copy>(x:T)->T{S::<T>::f(S(x))}fn bad<T>(x:T)->T{S::<T>::f(S(x))}fn main(){}",
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
fn prefixes_select_before_call_arguments_and_results() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in ACCEPT {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(&format!("method-selection-{name}"), &source, true)?;
        prove(&source).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn ambiguous_and_unsatisfied_candidates_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source) in REJECT {
        rustc::qualify(&format!("method-selection-{name}"), source, false)?;
        assert!(prove(source).is_err(), "{name}");
    }
    Ok(())
}
#[test]
fn concrete_prefixes_use_the_retained_shape_index() -> Result<(), Box<dyn std::error::Error>> {
    let mut source = "struct S<const N:usize>;".to_owned();
    for index in 0..256 {
        write!(source, "impl S<{index}>{{fn f(){{}}}}")?;
    }
    source.push_str("fn main(){");
    for index in 0..256 {
        write!(source, "S::<{index}>::f();")?;
    }
    source.push('}');
    rustc::qualify("method-selection-index", &source, true)?;
    let (_, facts) = semantic::analyze(&source)?;
    assert_eq!(facts.methods.selection_probes, 256);
    assert!(facts.methods.selection_steps < 256 * 64);
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn selected_inherent_methods_execute() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases("method-selection", ACCEPT)
}

#[test]
fn repeated_alias_prefixes_share_closed_shape_folding() -> Result<(), Box<dyn std::error::Error>> {
    let mut source = "struct S<T>(T);type A=S<fn(".to_owned();
    for index in 0..128 {
        write!(source, "&'static [u8;{index}],")?;
    }
    source.push_str(")>;impl A{fn f(){}}impl S<u8>{fn f(){}}fn main(){");
    for _ in 0..128 {
        source.push_str("A::f();");
    }
    source.push('}');
    rustc::qualify("method-selection-shape-sharing", &source, true)?;
    let (_, facts) = semantic::analyze(&source)?;
    assert_eq!(facts.methods.selection_probes, 128);
    assert_eq!(facts.methods.selection_shape_steps, 0);
    Ok(())
}
