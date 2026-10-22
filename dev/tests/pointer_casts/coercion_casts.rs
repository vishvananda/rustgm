//! Reference array casts reuse the checked coercion and selected slice metadata.
use super::{native, source};
use std::{path::Path, process::Command};
const CASES: &[(&str, bool, &str)] = &[
    (
        "shared",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn cast<'a>(p:&'a [u8;3])->&'a [u8]{p as &'a [u8]}fn main(){let a=[3,5,7];let s=cast(&a);check(s[1]==5);check(match s{[_,_,_]=>true,_=>false});}
",
    ),
    (
        "mutable",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn cast<'a>(p:&'a mut [u8;3])->&'a mut [u8]{p as &'a mut [u8]}fn main(){let mut a=[3,5,7];let s=cast(&mut a);s[1]=9;check(match s{[_,_,_]=>true,_=>false});check(a[1]==9);}
",
    ),
    (
        "mutable-shared",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn cast<'a>(p:&'a mut [u8;2])->&'a [u8]{p as &'a [u8]}fn main(){let mut a=[3,5];let s=cast(&mut a);check(s[1]==5);check(match s{[_,_]=>true,_=>false});}
",
    ),
    (
        "aliases",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<T,const N:usize>=[T;N];type S<'a,T>=&'a [T];fn cast<'a>(p:&'a A<u16,2>)->S<'a,u16>{p as S<'a,u16>}fn main(){let a=[300,500];let s=cast(&a);check(s[1]==500);}
",
    ),
    (
        "generic",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn cast<T,const N:usize>(p:&[T;N])->&[T]{p as &[T]}fn main(){let a=[3u8,5,7];let s=cast(&a);check(s[2]==7);check(match s{[_,_,_]=>true,_=>false});}
",
    ),
    (
        "generic-mutable",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn cast<T,const N:usize>(p:&mut [T;N])->&mut [T]{p as &mut [T]}fn main(){let mut a=[3u8,5];let s=cast(&mut a);s[0]=8;check(a[0]==8);}
",
    ),
    (
        "shared-covariance",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn cast<'a,'b>(p:&'b [&'a u8;1])->&'b [&'b u8] where 'a:'b{p as &'b [&'b u8]}fn main(){let x=7;let a=[&x];check(*cast(&a)[0]==7);}
",
    ),
    (
        "empty",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a:[u8;0]=[];let s=&a as &[u8];check(match s{[]=>true,_=>false});}
",
    ),
    (
        "zst",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=[();3];let s=&a as &[()];check(match s{[_,_,_]=>true,_=>false});}
",
    ),
    (
        "byte-constant",
        true,
        r#"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const A:&[u8]=b"abc" as &[u8];const N:usize=match A{[_,_,_]=>3,_=>0};fn main(){check(N==3);check(A[1]==98);}
"#,
    ),
    (
        "named-array",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const B:[u8;3]=[3,5,7];const A:&[u8]=&B as &[u8];fn main(){check(A[2]==7);check(match A{[_,_,_]=>true,_=>false});}
",
    ),
    (
        "const-generic",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const fn cast<const N:usize>(p:&[u8;N])->&[u8]{p as &[u8]}const A:&[u8]=cast(&[3,5]);fn main(){check(A[1]==5);check(match A{[_,_]=>true,_=>false});}
",
    ),
    (
        "evaluate-once",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn make(n:&mut u8)->&'static [u8;2]{*n+=1;&[3,5]}fn main(){let mut n=0;let s=make(&mut n) as &[u8];check(n==1);check(s[1]==5);}
",
    ),
    (
        "reborrow",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut a=[3u8,5];let r=&mut a;let s=r as &mut [u8];s[0]=8;r[1]=9;check(a[0]==8&&a[1]==9);}
",
    ),
    (
        "shared-reborrow",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut a=[3u8,5];let r=&mut a;let s=r as &[u8];check(s[0]==3);r[1]=9;check(a[1]==9);}
",
    ),
    (
        "noncopy",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn main(){let a=[S(3),S(5)];let s=&a as &[S];check(s[1].0==5);check(a[0].0==3);}
",
    ),
    (
        "wrong-element",
        false,
        r"fn cast(p:&[u16;2])->&[u8]{p as &[u8]}fn main(){}
",
    ),
    (
        "shared-mutable",
        false,
        r"fn cast(p:&[u8;2])->&mut [u8]{p as &mut [u8]}fn main(){}
",
    ),
    (
        "escape",
        false,
        r"fn cast<'a>(p:&'a [u8;2])->&'static [u8]{p as &'static [u8]}fn main(){}
",
    ),
    (
        "unpaid-element-lifetime",
        false,
        r"fn cast<'a>(p:&'a [&'a u8;1])->&'a [&'static u8]{p as &'a [&'static u8]}fn main(){}
",
    ),
    (
        "mutable-invariance",
        false,
        r"fn cast<'a,'b>(p:&'b mut [&'a u8;1])->&'b mut [&'b u8] where 'a:'b{p as &'b mut [&'b u8]}fn main(){}
",
    ),
    (
        "nested",
        false,
        r"fn cast<'a>(p:&'a &'a [u8;2])->&'a &'a [u8]{p as &'a &'a [u8]}fn main(){}
",
    ),
    (
        "reverse",
        false,
        r"fn cast(p:&[u8])->&[u8;2]{p as &[u8;2]}fn main(){}
",
    ),
    (
        "active-loan",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut a=[3u8,5];let r=&mut a;let s=r as &mut [u8];r[0]=9;check(s[0]==3);}
",
    ),
    (
        "grouped",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=[3u8,5];let s=(&a) as &[u8];check(s[1]==5);check(match s{[_,_]=>true,_=>false});}
",
    ),
    (
        "grouped-reborrow",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut a=[3u8,5];let r=&mut a;let s=(r) as &mut [u8];s[0]=8;r[1]=9;check(a[0]==8&&a[1]==9);}
",
    ),
    (
        "grouped-escape",
        false,
        r"fn cast<'a>(p:&'a [u8;2])->&'static [u8]{(p) as &'static [u8]}fn main(){}
",
    ),
    (
        "grouped-shared-mutable",
        false,
        r"fn cast(p:&[u8;2])->&mut [u8]{(p) as &mut [u8]}fn main(){}
",
    ),
];
#[test]
fn declared_reference_unsizing_casts() -> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa26/slice-casts/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for &(name, accepted, text) in CASES {
        source::qualify_at(&folder.join("rustc"), name, text, accepted)?;
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, text)?;
        let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&path)
            .output()?;
        std::fs::write(path.with_extension("stderr"), &result.stderr)?;
        assert_eq!(
            result.status.code(),
            Some(i32::from(!accepted)),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stderr.is_empty(), accepted, "{name}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn reference_unsizing_casts_execute() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|c| c.1)
        .map(|&(name, _, text)| {
            Ok((
                name,
                text.strip_prefix(native::CHECK)
                    .ok_or("missing native prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26/slice-casts/native", &cases)
}
