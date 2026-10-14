//! Source generic aliases must survive ownership, region checking, lowering and execution.
#[path = "harness/native_references.rs"]
mod native;
use std::{path::Path, process::Command};
const CASES: &[(&str, &str, bool)] = &[
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
