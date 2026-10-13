use std::{
    path::Path,
    process::{Command, Output},
};

const CHECK: &str = "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\n";
const CASES: &[(&str, &str)] = &[
    (
        "constant-pattern-read",
        "const N:u8=7;fn main(){match 7u8{N=>{let _=N;},_=>()}}",
    ),
    (
        "negative-enum",
        "#[derive(Clone,Copy)]#[repr(i8)]enum E{A=-3,B=4}static A:E=E::A;fn main(){check(A as i64 == -3);}",
    ),
    (
        "scalars",
        "const A:i128=-170141183460469231731687303715884105728;const B:u128=340282366920938463463374607431768211455;static C:(i128,u128)=(A,B);fn main(){check(C.0==A && C.1==B);}",
    ),
    (
        "static-fields",
        "struct S{a:u8,b:[u16;3]}static A:S=S{a:3,b:[4,5,6]};static B:S=S{a:3,b:[4,5,6]};fn main(){check(A.a==3 && A.b[2]==6 && B.b[0]==4);}",
    ),
    (
        "constant-snapshots",
        "const A:([u16;2],u8)=([7,11],3);fn main(){let mut a=A;let b=A;a.0[1]=99;check(a.0[1]==99 && b.0[1]==11 && A.1==3);}",
    ),
    (
        "enum-data",
        "#[derive(Clone,Copy)]#[repr(C,u8)]enum E{A(u64,u8),B{n:u128}}static A:E=E::A(42,7);const B:E=E::B{n:340282366920938463463374607431768211455};fn main(){let a=match A{E::A(n,b)=>n+b as u64,_=>0};let b=match B{E::B{n}=>n,_=>0};check(a==49 && b==340282366920938463463374607431768211455);}",
    ),
    (
        "packed-scalars",
        "#[repr(C,packed)]struct P{a:u8,b:u128,c:u16}static A:P=P{a:9,b:340282366920938463463374607431768211455,c:513};fn main(){check(A.a==9 && A.b==340282366920938463463374607431768211455 && A.c==513);}",
    ),
    (
        "function-data",
        "fn inc(x:u64)->u64{x+1}static F:fn(u64)->u64=inc;const G:fn(u64)->u64=inc;fn main(){check(F(7)==8 && G(9)==10);}",
    ),
    (
        "packed-functions",
        "fn inc(x:u64)->u64{x+1}#[repr(C,packed)]struct P{a:u8,f:fn(u64)->u64,b:u16}static A:P=P{a:9,f:inc,b:513};fn main(){check((A.f)(41)==42 && A.a==9 && A.b==513);}",
    ),
    (
        "repeated-packed-functions",
        "fn inc(x:u64)->u64{x+1}#[derive(Clone,Copy)]#[repr(C,packed)]struct P{a:u8,f:fn(u64)->u64}static A:[P;3]=[P{a:7,f:inc};3];fn main(){check((A[0].f)(1)==2 && (A[1].f)(4)==5 && (A[2].f)(8)==9 && A[1].a==7);}",
    ),
    (
        "zero-repeat",
        "static A:[[(u64,u8);1000];1000]=[[(0,0);1000];1000];fn main(){check(A[999][999].0==0 && A[1][2].1==0);}",
    ),
    (
        "zst-repeat",
        "static A:[();18446744073709551615]=[();18446744073709551615];fn main(){let _=A[7000];check(true);}",
    ),
    (
        "float-data",
        "static A:(f64,f32,f64)=(-0.0,0.0/0.0,1.0/0.0);fn main(){check(1.0/A.0 == -A.2 && A.1!=A.1 && A.2>1.0);}",
    ),
    (
        "aligned-data",
        "#[repr(align(64))]struct A(u64);static S:A=A(42);fn main(){check(S.0==42);}",
    ),
];
fn success(name: &str, output: Output) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "{name}: {:?}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}
#[test]
fn named_constants_and_static_storage_execute_like_rustc() -> Result<(), Box<dyn std::error::Error>>
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!("obj/constant-storage-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for &(name, body) in CASES {
        let source = folder.join(format!("{name}.rs"));
        std::fs::write(&source, format!("{CHECK}{body}"))?;
        let native = folder.join(format!("{name}.rustc"));
        success(
            name,
            Command::new("rustc")
                .args([
                    "+1.98.0",
                    "--edition=2024",
                    "--crate-name=probe",
                    "--cap-lints=allow",
                    "-C",
                    "overflow-checks=yes",
                    "-C",
                    "panic=abort",
                ])
                .arg(&source)
                .arg("-o")
                .arg(&native)
                .output()?,
        )?;
        assert!(success(name, Command::new(&native).output()?)?.is_empty());
        let full = success(
            name,
            Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg("--emit-lowir")
                .arg(&source)
                .output()?,
        )?;
        let again = success(
            name,
            Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg("--emit-lowir")
                .arg(&source)
                .output()?,
        )?;
        assert_eq!(full, again, "{name}");
        let text = std::str::from_utf8(&full)?;
        match name {
            "zero-repeat" => {
                assert!(text.contains("zero 16000000"));
                assert!(full.len() < 20_000);
            }
            "static-fields" => assert_eq!(text.matches("global @").count(), 2),
            "constant-snapshots" => assert_eq!(text.matches("global @").count(), 1),
            "packed-functions" | "repeated-packed-functions" => {
                assert!(text.contains("store ptr"));
                let globals = text.split("function @").next().ok_or("globals")?;
                assert!(!globals.contains("storage=readonly"));
                assert!(!globals.contains("ptr addr"));
            }
            "aligned-data" => assert!(text.contains("obj<64x64>")),
            _ => {}
        }
        let lowir = folder.join(format!("{name}.lowir"));
        std::fs::write(&lowir, full)?;
        let executable = folder.join(format!("{name}.native"));
        success(
            name,
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg(&lowir)
                .arg("-o")
                .arg(&executable)
                .output()?,
        )?;
        assert!(success(name, Command::new(executable).output()?)?.is_empty());
    }
    Ok(())
}
#[test]
fn persistent_data_growth_is_bounded_before_expanding_nonzero_repeats()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/constant-storage-budget-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let path = folder.join("input.rs");
    std::fs::write(
        &path,
        "static A:[u8;1000001]=[7;1000001];fn main(){let _=A[0];}",
    )?;
    let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-lowir")
        .arg(path)
        .output()?;
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8(result.stderr)?.contains("persistent data work budget exceeded"));
    Ok(())
}
