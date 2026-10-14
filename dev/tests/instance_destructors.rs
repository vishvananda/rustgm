//! Concrete Drop instances follow selected cleanup plans and keep their original proofs.
#[expect(
    dead_code,
    reason = "this suite uses the typed renderer rather than the CLI harness"
)]
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "permuted-const-values",
        "struct D<'a,const N:usize,const M:usize>(&'a mut u8,[u8;N],[u8;M]);impl<const A:usize,const B:usize> Drop for D<'_,B,A>{fn drop(&mut self){*self.0+=A as u8*10+B as u8;}}fn main(){let mut n=0;{let _d=D(&mut n,[1;2],[1;3]);}check(n==32);}",
        false,
    ),
    (
        "permuted-lifetimes",
        "struct D<'a,'b,T>(&'a mut u8,&'b u8,T);impl<'x,'y,T> Drop for D<'y,'x,T>{fn drop(&mut self){*self.0+=*self.1;}}fn main(){let mut n=0;let k=17;{let _d=D(&mut n,&k,9u16);}check(n==17);}",
        false,
    ),
    (
        "unused-impl-lifetime",
        "struct D<'a,T>(&'a mut u8,T);impl<'a,'u,T> Drop for D<'a,T> where 'a:'u{fn drop(&mut self){*self.0+=1;}}fn main(){let mut n=0;{let _d=D(&mut n,7u8);}check(n==1);}",
        false,
    ),
    (
        "unused-lifetime-chain",
        "struct D<'a,'b,T>(&'a mut u8,&'b u8,T)where 'a:'b;impl<'a,'b,'u,T> Drop for D<'a,'b,T> where 'a:'u,'u:'b{fn drop(&mut self){*self.0+=*self.1;}}fn main(){let mut n=0;let k=3;{let _d=D(&mut n,&k,7u8);}check(n==3);}",
        false,
    ),
    (
        "nested-reference-argument",
        "struct D<'a,T>(&'a mut u8,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0+=1;}}fn id<T>(x:T)->T{x}fn main(){let mut n=0;let k=7;let p=&k;{let _d=id(D(&mut n,&p));}check(n==1);}",
        false,
    ),
    (
        "unused-growing-drop",
        "struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){grow::<T>();}}fn grow<T>(){grow::<(T,T)>();}fn main(){check(true);}",
        false,
    ),
    (
        "zero-array-prototype",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn id<T>(x:T)->T{x}fn main(){let mut n=0;{let _a=id([D(&mut n);0]);}check(n==1);}",
        false,
    ),
    (
        "permuted-const-kinds",
        "struct D<'a,const B:bool,const C:char,const N:i8>(&'a mut u8);impl<const K:i8,const H:char,const A:bool> Drop for D<'_,A,H,K>{fn drop(&mut self){*self.0+=if A{1}else{2};*self.0+=H as u8;*self.0+=if K == -7{1}else{2};}}fn main(){let mut n=0;{let _d=D::<true,'A',{-7}>(&mut n);}check(n==67);}",
        false,
    ),
    (
        "identity-owned-value",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn id<T>(x:T)->T{x}fn main(){let mut n=0;{let _d=id(D(&mut n));}check(n==1);}",
        false,
    ),
    (
        "generic-drop-two-types",
        "struct D<'a,T>(&'a mut u8,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0+=1;}}fn main(){let mut n=0;{let _a=D(&mut n,7u8);}{let _b=D(&mut n,19u16);}check(n==2);}",
        false,
    ),
    (
        "permuted-drop-arguments",
        "struct D<'a,T,U>(&'a mut u8,T,U);impl<A,B> Drop for D<'_,B,A>{fn drop(&mut self){*self.0+=1;}}fn main(){let mut n=0;{let _a=D(&mut n,7u8,1000u16);}check(n==1);}",
        false,
    ),
    (
        "drop-const-lengths",
        "struct D<'a,const N:usize>(&'a mut u8,[u8;N]);impl<const N:usize> Drop for D<'_,N>{fn drop(&mut self){*self.0+=1;}}fn main(){let mut n=0;{let _a=D(&mut n,[1;0]);}{let _b=D(&mut n,[2;4]);}check(n==2);}",
        false,
    ),
    (
        "outer-before-generic-fields",
        "struct D<'a,T>(&'a mut u16,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0=*self.0*10+1;}}struct W<'a,T>(D<'a,T>);impl<T> Drop for W<'_,T>{fn drop(&mut self){*self.0.0=*self.0.0*10+2;}}fn main(){let mut n=0;{let _w=W(D(&mut n,7u8));}check(n==21);}",
        false,
    ),
    (
        "generic-enum-fields",
        "struct D<'a,T>(&'a mut u8,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0+=1;}}enum E<T>{A(T),B(T,T)}fn main(){let mut a=0;let mut b=0;{let _e=E::B(D(&mut a,7u8),D(&mut b,8u8));}check(a==1&&b==1);}",
        false,
    ),
    (
        "partly-moved-generic-wrapper",
        "struct D<'a,T>(&'a mut u8,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0+=1;}}struct W<T>(T,T);fn take<T>(x:T){}fn main(){let mut a=0;let mut b=0;{let w=W(D(&mut a,7u8),D(&mut b,8u8));take(w.0);}check(a==1&&b==1);}",
        false,
    ),
    (
        "zero-array-generic-prototype",
        "struct D<'a,T>(&'a mut u8,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0+=1;}}fn id<T>(x:T)->T{x}fn main(){let mut n=0;{let _a=id([D(&mut n,7u8);0]);}check(n==1);}",
        false,
    ),
    (
        "conditional-generic-drop",
        "struct D<'a,T>(&'a mut u8,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0+=1;}}fn f<const X:bool>(n:&mut u8){let d;if X{d=D(n,7u8);}}fn main(){let mut n=0;f::<false>(&mut n);check(n==0);f::<true>(&mut n);check(n==1);}",
        false,
    ),
    (
        "generic-array-cleanup",
        "struct D<'a,T>(&'a mut u8,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0+=1;}}fn array<T,const N:usize>(x:[T;N])->[T;N]{x}fn main(){let mut a=0;let mut b=0;{let _a=array([D(&mut a,7u8),D(&mut b,8u8)]);}check(a==1&&b==1);}",
        false,
    ),
];
fn compile(source: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let lowir = rustgm::lower::instances::build(&report, &mut facts, entry)?;
    let mut bytes = Vec::new();
    rustgm::lowir::writer::write(&lowir, &mut bytes)?;
    Ok(bytes)
}
#[test]
fn selected_destructors_lower_with_separate_template_and_instance_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, _) in CASES {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(name, &source, true)?;
        let bytes = compile(&source).map_err(|e| format!("{name}: {e}"))?;
        assert!(!bytes.is_empty());
        assert_eq!(bytes, compile(&source)?, "{name}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn selected_destructors_execute_against_rustc() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases_with("instance-destructors", CASES, |source, _| compile(source))
}

#[test]
fn sequential_cleanup_scopes_do_not_rescan_every_inactive_lifetime_value()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, extra, start, scope, end) in [
        (
            "sequential-generic-cleanup",
            "",
            "let mut n=0;",
            "{let d=D(&mut n,7u8);take(d);}",
            "check(n==2048);}",
        ),
        (
            "sequential-partial-cleanup",
            "struct W<T>(T,T);",
            "let mut a=0;let mut b=0;",
            "{let w=W(D(&mut a,7u8),D(&mut b,8u8));take(w.0);}",
            "check(a==2048&&b==2048);}",
        ),
    ] {
        let mut source = format!(
            "{}struct D<'a,T>(&'a mut usize,T);impl<T>Drop for D<'_,T>{{fn drop(&mut self){{*self.0+=1;}}}}fn take<T>(x:T){{}}{extra}fn main(){{{start}",
            native::CHECK
        );
        for _ in 0..2048 {
            source.push_str(scope);
        }
        source.push_str(end);
        rustc::qualify(name, &source, true)?;
        compile(&source).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
