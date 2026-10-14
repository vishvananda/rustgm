//! Source generic aliases must survive ownership, region checking, lowering and execution.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
use std::{path::Path, process::Command};
const CASES: &[(&str, &str, bool)] = &[
    (
        "early-field-default",
        "struct S(usize);type A<const N:usize={S(3).0}>=[u8;N];fn read(x:A)->u8{x[2]}fn main(){check(read([2,5,9])==9);}",
        false,
    ),
    (
        "early-reference-default",
        "struct S<'a>(&'a usize);type A<const N:usize={*S(&3).0}>=[u8;N];fn read(x:A)->u8{x[2]}fn main(){check(read([1,2,7])==7);}",
        false,
    ),
    (
        "early-enum-cast-default",
        "type I<T>=T;enum E{A=C as I<isize>,B}const C:u8=7;type A<const N:usize={E::B as usize}>=[u8;N];fn read(x:A)->u8{x[7]}fn main(){check(read([13;8])==13);}",
        false,
    ),
    (
        "early-packed-default",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967297}#[repr(packed(8))]struct S{e:E}type A<const N:usize={*(&S{e:E::B}.e) as u8 as usize}>=[u8;N];static V:S=S{e:E::B};fn read(x:A)->u8{x[0]}fn main(){check(read([19])==19);check(V.e as u64==4294967297);}",
        false,
    ),
    (
        "early-drop-target-default",
        "struct Count<'a>(&'a mut u8);struct Size(usize);type D<'a,T=Count<'a>,const N:usize={Size(3).0}>=T;impl<'a> Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}fn consume<'a>(_:Count<'a>){}fn main(){let mut n=0;consume(Count(&mut n));check(n==1);}",
        false,
    ),
    (
        "early-drop-static-target",
        "struct S{n:usize}static V:S=S{n:3};type A<T=S,const N:usize={V.n}>=T;impl Drop for A{fn drop(&mut self){check(false);}}fn consume(_:S){}fn main(){consume(S{n:V.n});}",
        true,
    ),
    (
        "aligned-large-enum-argument",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967297}#[repr(packed(8))]struct S{e:E}type A<const N:usize>=[u8;N];static V:S=S{e:E::B};fn main(){let x:A<{*(&S{e:E::B}.e) as u8 as usize}>=[9;1];check(x[0]==9);check(V.e as u64==4294967297);}",
        false,
    ),
    (
        "aligned-zero-array-argument",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(8))]struct S{e:[E;0]}type A<const N:usize>=[u8;N];fn main(){let x:A<{(&S{e:[]}.e,1).1}>=[7;1];check(x[0]==7);check(E::B as u64==4294967296);}",
        false,
    ),
    (
        "aligned-dead-arithmetic",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(8))]struct S{e:E}type A<const N:usize>=[u8;N];fn main(){let x:A<{if true{1}else{(*(&S{e:E::B}.e) as u8 as usize)/(E::A as usize)}}>=[5;1];check(x[0]==5);}",
        false,
    ),
    (
        "enum-argument",
        "enum E{A=7,B}type A<const N:usize>=[u8;N];fn main(){let x:A<{E::B as usize}>=[3;8];check(x[7]==3);}",
        false,
    ),
    (
        "signed-enum-argument",
        "#[repr(i8)]enum E{A=-3,B}type A<const N:usize>=[u8;N];fn main(){let x:A<{(E::B as isize+3) as usize}>=[17;1];check(x[0]==17);}",
        false,
    ),
    (
        "extended-drop-reference",
        "struct S{n:usize}impl Drop for S{fn drop(&mut self){check(false);}}const R:&S=&S{n:5};type A<const N:usize>=[u8;N];fn main(){let x:A<{R.n}>=[2;5];check(x[4]==2);check(R.n==5);}",
        false,
    ),
    (
        "returned-drop-constant",
        "struct S(u8);impl Drop for S{fn drop(&mut self){check(false);}}const C:S=S(7);fn consume(_:S){}fn main(){consume(C);}",
        true,
    ),
    (
        "self-dependent-default",
        "type A<T=A<u16>,U=T>=(T,U);fn sum(x:A)->u16{x.0.0+x.0.1+x.1.0+x.1.1}fn main(){check(sum(((1,2),(3,4)))==10);}",
        false,
    ),
    (
        "self-later-const-default",
        "type A<T=A<u8>,const N:usize=3>=(T,[u8;N]);fn f(x:A)->u8{x.0.0+x.0.1[2]+x.1[1]}fn main(){check(f(((7,[1,2,3]),[4,5,6]))==15);}",
        false,
    ),
    (
        "self-lifetime-default",
        "type A<'a,T=A<'a,&'a u16>>=T;fn f<'a>(x:A<'a>)->&'a u16{x}fn main(){let x=517;check(*f(&x)==517);}",
        false,
    ),
    (
        "self-default-owned-drop",
        "struct S<'a>(&'a mut u8);impl<'a> Drop for S<'a>{fn drop(&mut self){*self.0+=1;}}type A<'a,T=A<'a,S<'a>>>=T;fn consume<'a>(_:A<'a>){}fn main(){let mut n=0;consume(S(&mut n));check(n==1);}",
        false,
    ),
    (
        "mutual-defaults",
        "type A<T=B<u16>>=T;type B<U=A<u16>>=U;fn f(x:A)->B{x}fn main(){check(f(500)==500);}",
        false,
    ),
    (
        "nested-reference-binder",
        "type A<T>=for<'a> fn(&'a u8,T)->T;fn keep<'a,'b>(_:&'a u8,x:&'b u8)->&'b u8{x}fn apply<'b>(f:A<&'b u8>,x:&'b u8)->&'b u8{let y=7;f(&y,x)}fn main(){let z=31;check(*apply(keep,&z)==31);}",
        false,
    ),
    (
        "array-default",
        "type A<T=u16,const N:usize=3>=[T;N];fn take(x:A)->u16{x[0]+x[1]+x[2]}fn main(){check(take([2,3,4])==9);}",
        false,
    ),
    (
        "distinct-lengths",
        "type A<T,const N:usize>=[T;N];fn a(x:A<u8,2>)->u8{x[1]}fn b(x:A<u16,3>)->u16{x[2]}fn main(){check(a([1,7])==7);check(b([2,3,600])==600);}",
        false,
    ),
    (
        "default-forward",
        "type A<T,U=T>=(T,U);type B<V=u8>=A<V>;fn f(x:B)->u8{x.0+x.1}fn main(){check(f((7,8))==15);}",
        false,
    ),
    (
        "reference-default",
        "type A<'a,T=&'a u16>=T;fn f<'b>(x:A<'b>)->&'b u16{x}fn main(){let x=517;check(*f(&x)==517);}",
        false,
    ),
    (
        "callback",
        "type A<T>=fn(T)->T;fn increment(x:u16)->u16{x+1}fn call(f:A<u16>,x:u16)->u16{f(x)}fn main(){check(call(increment,345)==346);}",
        false,
    ),
    (
        "owned-drop",
        "struct S(u8);impl Drop for S{fn drop(&mut self){check(self.0==7);}}type A<T>=T;fn consume(_:A<S>){}fn main(){consume(S(7));}",
        false,
    ),
    (
        "alias-index-panic",
        "type A<const N:usize>=[u8;N];fn read(x:A<2>,i:usize)->u8{x[i]}fn main(){read([3,4],2);}",
        true,
    ),
    (
        "zero-sized",
        "struct Z;type A<T,const N:usize>=[T;N];fn take(_:A<Z,2>){}fn main(){take([Z,Z]);}",
        false,
    ),
];
#[test]
fn generic_alias_programs_complete_the_lowir_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/generic-alias-lowir-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for &(name, source, _) in CASES {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, format!("{}{source}", native::CHECK))?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(path)
            .output()?;
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !output.stdout.is_empty() && output.stderr.is_empty(),
            "{name}"
        );
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn generic_alias_programs_match_independent_native_execution()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases("generic-alias-native", CASES)
}

#[test]
fn ordinary_drop_temporaries_cannot_acquire_static_storage()
-> Result<(), Box<dyn std::error::Error>> {
    let source =
        "struct S(u8);impl Drop for S{fn drop(&mut self){}}fn bad()->&'static S{&S(3)}fn main(){}";
    rustc::qualify("drop-static-provenance-negative", source, false)?;
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/drop-static-provenance-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let input = folder.join("input.rs");
    std::fs::write(&input, source)?;
    let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-lowir")
        .arg(input)
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
    Ok(())
}
