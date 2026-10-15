//! Qualified methods feed ordinary call, instance and ownership machinery.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::resolve::model::{DefId, DefKind};

const CASES: &[(&str, &str, bool)] = &[
    (
        "value",
        "struct S(u8);impl S{fn get(self)->u8{self.0}}fn main(){check(S::get(S(7))==7);}",
        false,
    ),
    (
        "shared",
        "struct S(u8);impl S{fn get(&self)->u8{self.0}}fn main(){let s=S(7);check(S::get(&s)==7);check(S::get(&s)==7);}",
        false,
    ),
    (
        "mutable",
        "struct S(u8);impl S{fn set(&mut self,x:u8){self.0=x;}}fn main(){let mut s=S(1);S::set(&mut s,9);check(s.0==9);}",
        false,
    ),
    (
        "function-value",
        "struct S(u8);impl S{fn get(self)->u8{self.0}}fn main(){let f:fn(S)->u8=S::get;check(f(S(11))==11);}",
        false,
    ),
    (
        "generic-impl",
        "struct S<T>(T);impl<T>S<T>{fn make(x:T)->Self{S(x)}fn take(self)->T{self.0}}fn main(){check(S::take(S::make(13u8))==13);check(S::<u16>::take(S::<u16>::make(517))==517);}",
        false,
    ),
    (
        "generic-method",
        "struct S<T>(T);impl<T>S<T>{fn pair<U>(self,x:U)->(T,U){(self.0,x)}}fn main(){let x=S::<u8>::pair::<u16>(S(7),517);check(x.0==7);check(x.1==517);}",
        false,
    ),
    (
        "const-impl",
        "struct S<const N:usize>([u8;N]);impl<const N:usize>S<N>{fn count(&self)->usize{N}fn last(self)->u8{self.0[N-1]}}fn main(){let s=S([2,3,7]);check(S::count(&s)==3);check(S::<3>::last(s)==7);}",
        false,
    ),
    (
        "const-method",
        "struct S(u8);impl S{fn sum<const N:usize>(&self,x:[u8;N])->u8{self.0+x[N-1]}}fn main(){check(S::sum::<2>(&S(3),[5,7])==10);}",
        false,
    ),
    (
        "alias-prefix",
        "struct S<T>(T);type A<T>=S<T>;impl<T>S<T>{fn get(self)->T{self.0}}fn main(){check(A::<u16>::get(S(517))==517);}",
        false,
    ),
    (
        "captured-lifetime",
        "struct S<'a>(&'a u8);impl<'a>S<'a>{fn get(self)->&'a u8{self.0}}fn read<'a>(x:&'a u8)->&'a u8{S::get(S(x))}fn main(){let x=7;check(*read(&x)==7);}",
        false,
    ),
    (
        "unused-capture",
        "struct S<'a>(&'a u8);impl<'unused,'a>S<'a>{fn get(self)->&'a u8{self.0}}fn main(){let x=7;check(*S::get(S(&x))==7);}",
        false,
    ),
    (
        "owner-bound",
        "struct S<T:Copy>(T);impl<T:Copy>S<T>{fn get(&self)->T{self.0}}fn main(){check(S::get(&S(9u8))==9);}",
        false,
    ),
    (
        "method-bound",
        "struct S<T>(T);impl<T>S<T>{fn get(&self)->T where T:Copy{self.0}}fn main(){check(S::get(&S(9u8))==9);}",
        false,
    ),
    (
        "self-prefix",
        "struct S<T>(T);impl<T>S<T>{fn make(x:T)->Self{S(x)}fn again(x:T)->Self{Self::make(x)}}fn main(){check(S::again(17u8).0==17);}",
        false,
    ),
    (
        "qualified-type",
        "struct S(u8);impl S{fn get(self)->u8{self.0}}fn main(){let f:fn(S)->u8=<S>::get;check(f(S(19))==19);}",
        false,
    ),
    (
        "qualified-generic",
        "struct S<T>(T);impl<T>S<T>{fn pair<U>(self,x:U)->(T,U){(self.0,x)}}fn main(){let x=<S<u16>>::pair::<u8>(S(517),7);check(x.0==517);check(x.1==7);}",
        false,
    ),
    (
        "qualified-hole",
        "struct S<T>(T);impl<T>S<T>{fn get(self)->T{self.0}}fn main(){check(<S<_>>::get(S(17u16))==17);}",
        false,
    ),
    (
        "qualified-alias",
        "struct S<T>(T);type A<T>=S<T>;impl<T>S<T>{fn get(self)->T{self.0}}fn main(){check(<A<u8>>::get(S(19))==19);}",
        false,
    ),
    (
        "qualified-self",
        "struct S<T>(T);impl<T>S<T>{fn make(x:T)->Self{S(x)}fn again(x:T)->Self{<Self>::make(x)}}fn main(){check(S::again(17u8).0==17);}",
        false,
    ),
    (
        "constant-function",
        "struct S(u8);impl S{fn get(self)->u8{self.0}}const F:fn(S)->u8=<S>::get;fn main(){check(F(S(23))==23);}",
        false,
    ),
    (
        "mixed-owner-method",
        "struct S<'a,T:Copy,const N:usize>(&'a T,[u8;N]);impl<'a,T:Copy,const N:usize>S<'a,T,N>{fn pair<'b,U:Copy>(self,x:&'b U)->(&'a T,&'b U){(self.0,x)}}fn forward<'a,'b,T:Copy,U:Copy,const N:usize>(s:S<'a,T,N>,x:&'b U)->(&'a T,&'b U){S::pair::<U>(s,x)}fn main(){let a=7u8;let b=517u16;let x=forward(S(&a,[1,2,3]),&b);check(*x.0==7);check(*x.1==517);}",
        false,
    ),
    (
        "early-method-lifetime",
        "struct S<'a>(&'a u8);impl<'a>S<'a>{fn choose<'b>(&self,x:&'b u8)->&'b u8 where 'b:'a{x}}fn pick<'a>(s:S<'a>,x:&'a u8)->&'a u8{S::choose::<'a>(&s,x)}fn main(){let a=3;let b=7;check(*pick(S(&a),&b)==7);}",
        false,
    ),
    (
        "unused-bounded-capture",
        "struct S<'a>(&'a u8);impl<'unused:'a,'a>S<'a>{fn get(self)->&'a u8{self.0}}fn main(){let x=11;check(*S::get(S(&x))==11);}",
        false,
    ),
    (
        "alias-inference",
        "struct S<T>(T);type A<T>=S<T>;impl<T>S<T>{fn get(self)->T{self.0}}fn main(){check(A::get(S(517u16))==517);check(A::<>::get(S(7u8))==7);}",
        false,
    ),
    (
        "alias-default-inference",
        "struct S<T>(T);type A<T=u8>=S<T>;impl<T>S<T>{fn get(self)->T{self.0}}fn main(){let x:u16=A::get(S(517));check(x==517);}",
        false,
    ),
    (
        "alias-const-inference",
        "struct S<const N:usize>([u8;N]);type A<const N:usize=1>=S<N>;impl<const N:usize>S<N>{fn get(self)->u8{self.0[N-1]}}fn main(){check(A::get(S([3,7]))==7);}",
        false,
    ),
    (
        "alias-lifetime-inference",
        "struct S<'a,T>(&'a T);type A<'a,T>=S<'a,T>;impl<'a,T>S<'a,T>{fn get(self)->&'a T{self.0}}fn read<'a>(x:&'a u16)->&'a u16{A::<'a>::get(S(x))}fn main(){let x=517;check(*read(&x)==517);}",
        false,
    ),
    (
        "alias-variant-priority",
        "enum E{V(u8)}type A=E;impl E{fn V()->u8{9}}fn main(){let E::V(x)=A::V(7);check(x==7);}",
        false,
    ),
    (
        "qualified-variant-priority",
        "enum E{V(u8)}impl E{fn V()->u8{9}}fn main(){let E::V(x)=<E>::V(11);check(x==11);}",
        false,
    ),
    (
        "generic-variant-priority",
        "enum E<T>{V(T)}type A<T>=E<T>;impl<T>E<T>{fn V()->u8{9}}fn main(){let E::V(x)=A::V(517u16);check(x==517);let E::V(y)=<E<_>>::V(7u8);check(y==7);}",
        false,
    ),
    (
        "qualified-unit-variant",
        "enum E{V}impl E{fn V()->u8{9}}const C:E=<E>::V;fn main(){let E::V=C;let E::V=<E>::V;}",
        false,
    ),
    (
        "enum-alias-method",
        "enum E<T>{V(T)}type A<T>=E<T>;impl<T>E<T>{fn get(self)->T{let E::V(x)=self;x}}fn main(){check(A::get(A::V(517u16))==517);}",
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
fn qualified_calls_and_function_values_retain_owner_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in CASES {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(&format!("qualified-{name}"), &source, true)?;
        prove(&source).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}

#[test]
fn selected_method_constraints_do_not_disappear_in_call_formation()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "target-mismatch",
            "struct S<T>(T);impl S<u8>{fn get(self)->u8{self.0}}fn main(){let _=S::<u16>::get(S(7));}",
        ),
        (
            "private",
            "mod m{pub struct S;impl S{fn get(&self){}}}fn main(){m::S::get(&m::S);}",
        ),
        (
            "inherited-copy",
            "struct N(u8);struct S<T>(T);impl<T:Copy>S<T>{fn get(self)->T{self.0}}fn main(){let _=S::get(S(N(7)));}",
        ),
        (
            "own-copy",
            "struct N(u8);struct S<T>(T);impl<T>S<T>{fn get(self)->T where T:Copy{self.0}}fn main(){let _=S::get(S(N(7)));}",
        ),
        (
            "borrow-escape",
            "struct S<'a>(&'a u8);impl<'a>S<'a>{fn get(self)->&'a u8{self.0}}fn main(){let r;{let x=7;r=S::get(S(&x));}let _=*r;}",
        ),
        (
            "explicit-two-phase",
            "struct S(u8);impl S{fn get(&self)->u8{self.0}fn set(&mut self,x:u8){self.0=x;}}fn main(){let mut s=S(7);S::set(&mut s,S::get(&s));}",
        ),
        (
            "impl-argument-in-method",
            "struct S<T>(T);impl<T>S<T>{fn get(self)->T{self.0}}fn main(){let _=S::<u8>::get::<u8>(S(7));}",
        ),
        (
            "impl-lifetime-in-method",
            "struct S<'a>(&'a u8);impl<'a>S<'a>{fn choose<'b>(self,x:&'b u8)->&'b u8 where 'b:'a{x}}fn main(){let x=7;let _=S::choose::<'_, '_>(S(&x),&x);}",
        ),
        (
            "explicit-alias-needs-arguments",
            "struct S<T>(T);type A<T>=S<T>;impl<T>S<T>{fn get(self)->T{self.0}}fn main(){let _=<A>::get(S(7u8));}",
        ),
        (
            "partial-alias-needs-arguments",
            "struct S<T,U>(T,U);type A<T,U>=S<T,U>;impl<T,U>S<T,U>{fn get(self)->T{self.0}}fn main(){let _=A::<u8>::get(S(7,9u16));}",
        ),
        (
            "qualified-variant-arguments",
            "enum E<T>{V(T)}fn main(){let _=<E<u8>>::V::<u8>(7);}",
        ),
        (
            "explicit-alias-retains-default",
            "struct S<T>(T);type A<T=u8>=S<T>;impl<T>S<T>{fn get(self)->T{self.0}}fn main(){let _=<A>::get(S(517u16));}",
        ),
    ] {
        rustc::qualify(&format!("qualified-{name}"), source, false)?;
        assert!(prove(source).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn selected_owner_captures_remain_existential_before_instance_erasure()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{Kind, RegionId, arguments::Argument};
    let source = "struct S<'a>(&'a u8);impl<'unused,'a>S<'a>{fn get(self)->&'a u8{self.0}}fn main(){let x=7;let _=S::get(S(&x));}";
    rustc::qualify("qualified-existential-captures", source, true)?;
    let (_, facts) = semantic::analyze(source)?;
    assert_eq!(facts.methods.qualified.len(), 1);
    for key in facts.methods.qualified.keys() {
        let ty = facts
            .node(*key)
            .and_then(|f| f.ty)
            .ok_or("missing selected item type")?;
        let Some(Kind::FunctionItem { arguments, .. }) = facts.types.kind(ty) else {
            return Err("missing item".into());
        };
        let arguments = facts
            .types
            .arguments(arguments)
            .ok_or("missing arguments")?
            .collect::<Vec<_>>();
        assert_eq!(arguments.len(), 2);
        assert!(arguments.iter().all(|argument| matches!(argument, Argument::Lifetime(region) if *region != RegionId::STATIC && region.bound_parts().is_none())));
        assert_ne!(arguments[0], arguments[1]);
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn qualified_methods_execute() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases("qualified-methods", CASES)
}
