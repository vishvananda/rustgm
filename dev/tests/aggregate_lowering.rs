use std::{
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};
const CHECK: &str = "fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}\n";
const CASES: &[(&str, &str)] = &[
    (
        "zst-effects",
        "#[derive(Clone,Copy)]struct Z;fn main(){let mut n=0;let a=[{n+=1;Z};0];let b=[{n+=1;Z};1];let c=[{n+=1;Z};18446744073709551615];check(n==3);let _=(a,b,c);}",
    ),
    (
        "enum-array",
        "#[derive(Clone,Copy)]#[repr(i16)]enum E{A=-7,B=99}fn make()->[E;4]{[E::A;4]}fn main(){let mut a=make();a[2]=E::B;check(a[0] as i64 == -7 && a[2] as u64==99 && a[3] as i16 == -7);}",
    ),
    (
        "tuple",
        "fn pair(a:u64,b:u8)->(u64,u8){(a,b)} fn main(){let p=pair(42,7);check(p.0==42 && p.1==7);}",
    ),
    (
        "snapshot",
        "fn sum(a:(u64,u64),b:(u64,u64))->u64{a.0+a.1+b.0+b.1} fn main(){let mut p=(1,2);let n=sum(p,{p.0=9;p});check(n==14);}",
    ),
    (
        "function-pointer",
        "struct S{a:u64,b:u8} fn id(x:S)->S{x} fn main(){let f:fn(S)->S=id;let s=f(S{a:44,b:3});check(s.a==44 && s.b==3);}",
    ),
    (
        "array-repeat",
        "#[derive(Clone,Copy)]struct S(u64,u8); fn arr()->[S;7]{[S(12,3);7]} fn main(){let mut a=arr();a[5].0=20;check(a[0].0==12 && a[5].0==20 && a[6].1==3);}",
    ),
    (
        "repeat-zero-one",
        "struct N(u64); fn f()->N{N(3)} fn main(){let a=[f();0];let b=[f();1];check(b[0].0==3);let _=a;}",
    ),
    (
        "assignment",
        "fn main(){let mut a=[1u8,2];a[0]+={a[0]=9;3};check(a[0]==12);}",
    ),
    (
        "update",
        "struct S{a:u64,b:u64} fn main(){let mut s=S{a:1,b:2};let n=S{b:{s.a=9;5},..s};check(n.a==9 && n.b==5);}",
    ),
    (
        "partial-move",
        "struct N(u64);struct S{a:N,b:N} fn main(){let mut s=S{a:N(7),b:N(9)};let a=s.a;s.a=N(20);let t=s;check(a.0==7 && t.a.0==20 && t.b.0==9);}",
    ),
    (
        "packed",
        "#[repr(C)]#[derive(Clone,Copy)]struct I{a:u64,b:u8} #[repr(C,packed)]struct S{tag:u8,i:I} fn id(x:S)->S{x} fn main(){let mut s=id(S{tag:2,i:I{a:444,b:8}});let i=s.i;s.i=I{a:555,b:9};check(i.a==444 && s.i.a==555 && s.i.b==9);}",
    ),
    (
        "aligned",
        "#[repr(align(64))]struct S(u64,u8); fn id(x:S)->S{x} fn main(){let s=id(S(42,8));check(s.0==42 && s.1==8);}",
    ),
    (
        "zero-sized",
        "#[derive(Clone,Copy)]struct Z; fn id(x:[Z;18446744073709551615])->[Z;18446744073709551615]{x} fn main(){let a=id([Z;18446744073709551615]);let _=a[8];check(true);}",
    ),
    (
        "tag-i8",
        "#[repr(i8)]enum E{A=-2,B=7,C} fn id(e:E)->E{e} fn main(){check(id(E::A) as i64 == -2 && E::B as u64==7 && E::C as u8==8);}",
    ),
    (
        "tag-u128",
        "#[repr(u128)]enum E{A=340282366920938463463374607431768211454,B} fn main(){check(E::A as u128==340282366920938463463374607431768211454 && E::B as u64==18446744073709551615);}",
    ),
    (
        "tag-c",
        "#[repr(C)]enum E{A=-5,B=7} fn main(){check(E::A as i64 == -5 && E::B as u64==7);}",
    ),
    ("tag-zst", "enum E{A=42} fn main(){check(E::A as u8==42);}"),
    (
        "tag-fieldless",
        "#[repr(u8)]enum E{A=10,B(),C{},D} fn main(){check(E::B() as u8==11 && E::C{} as u8==12 && E::D as u8==13);}",
    ),
    (
        "enum-payload",
        "#[repr(C,u8)]enum E{A(u64,u8),B{a:u128}} fn id(x:E)->E{x} fn main(){let a=id(E::A(42,7));let b=id(E::B{a:99});let _=(a,b);}",
    ),
    (
        "nested-arrays",
        "fn id(x:[[u16;3];2])->[[u16;3];2]{x} fn main(){let mut a=id([[1,2,3],[4,5,6]]);a[1][2]=70;check(a[1][2]==70 && a[0][2]==3);}",
    ),
];
fn compile(source: &str, count: bool) -> Result<Output, Box<dyn std::error::Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(if count {
            "--count-lowir"
        } else {
            "--emit-lowir"
        })
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing compiler input")?
        .write_all(source.as_bytes())?;
    Ok(child.wait_with_output()?)
}
#[test]
fn aggregate_calling_convention_and_places_emit_deterministically()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa12-work/lowering-vectors")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (name, body) in CASES {
        let source = format!("{CHECK}{body}");
        let full = compile(&source, false)?;
        assert!(
            full.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&full.stderr)
        );
        assert!(full.stderr.is_empty());
        let again = compile(&source, false)?;
        assert_eq!(full.stdout, again.stdout);
        std::fs::write(folder.join(format!("{name}.rs")), &source)?;
        std::fs::write(folder.join(format!("{name}.lowir")), &full.stdout)?;
        let text = String::from_utf8(full.stdout)?;
        if *name == "function-pointer" {
            assert!(text.contains("as (%p0 : ptr, %p1 : ptr) -> void"));
            assert!(text.contains("copyobj 16x8"));
        }
        if *name == "packed" {
            assert!(text.contains("copyobj 16x1"));
        }
        if *name == "aligned" {
            assert!(text.contains("obj<64x64>"));
        }
        if *name == "zero-sized" {
            assert!(!text.contains("obj<"));
            assert!(!text.contains("copyobj"));
            assert!(text.contains("cmp uge i64 8, 18446744073709551615"));
        }
    }
    Ok(())
}
#[test]
fn array_repetition_emits_constant_instruction_count() -> Result<(), Box<dyn std::error::Error>> {
    let mut counts = Vec::new();
    for length in [2, 100_000] {
        let output = compile(&format!("fn main(){{let a=[7u8;{length}];let _=a;}}"), true)?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        counts.push(output.stdout);
    }
    assert_eq!(counts[0], counts[1]);
    Ok(())
}
#[test]
fn aggregate_erasure_cannot_bypass_move_or_reference_checks()
-> Result<(), Box<dyn std::error::Error>> {
    for source in [
        "struct N; fn main(){return;let _a=[N;2];}",
        "struct N; fn f()->[N;2]{[return f();2]}fn main(){}",
        "struct N;fn main(){let a=N;let _b=a;let _c=a;}",
        "struct N;fn main(){let a=[N];let _b=a[0];}",
        "struct S(&'static u8);fn main(){let a:[S;0]=[];let _=a;}",
        "#[repr(u8)]enum E{A{}=1,B}fn main(){let _=E::B as u8;}",
        "enum E{A(u8),B}fn main(){let _=E::B as u8;}",
        "enum E{A}fn main(){let _=E::A as bool;}",
    ] {
        let output = compile(source, false)?;
        assert_eq!(output.status.code(), Some(1), "{source}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    Ok(())
}
