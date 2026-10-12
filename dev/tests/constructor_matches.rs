use std::{
    path::{Path, PathBuf},
    process::Command,
};
const CHECK: &str = "fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}\n";
const ACCEPTED: &[(&str, &str)] = &[
    (
        "uninitialized-bindings",
        "enum E{A(u8),B(u8)}fn main(){let (E::A(n)|E::B(n)):E;n=7;let (m@0..=255):u8;m=9;let (E::A(_)|E::B(_)):E;let ((0,k)|(_,k)):(u8,u8);k=11;check(n+m+k==27);}",
    ),
    (
        "former-pa12-boundary",
        "enum E{A,B}fn main(){let n=match E::A{E::A=>1,E::B=>2};}",
    ),
    (
        "let-alternatives",
        "enum E{A(u8,u64),B(u64,u8)}fn f(e:E)->u64{let (E::A(a,b)|E::B(b,a))=e;a as u64+b}fn main(){check(f(E::A(3,100))==103&&f(E::B(200,7))==207);}",
    ),
    (
        "parameter-alternatives",
        "enum E{A(u8),B(u8)}fn f((E::A(n)|E::B(n)):E)->u8{n}fn main(){check(f(E::A(3))==3&&f(E::B(7))==7);}",
    ),
    (
        "full-domain-binding",
        "fn main(){let x:u8;let (0..=255)=x;let (true|false)=true;check(true);}",
    ),
    (
        "composite-irrefutable-or",
        "fn main(){let ((n@(0|1),true|false)|(n@2..,true|false))=(7u8,false);check(n==7);}",
    ),
    (
        "unit-variants",
        "enum E{A,B,C}fn f(e:E)->u8{match e{E::A=>1,E::B=>2,E::C=>3}}fn main(){check(f(E::A)==1&&f(E::B)==2&&f(E::C)==3);}",
    ),
    (
        "signed-tags",
        "#[repr(i8)]enum E{A=-128,B=0,C=127}fn f(e:E)->u8{match e{E::A=>1,E::B=>2,E::C=>3}}fn main(){check(f(E::A)==1&&f(E::B)==2&&f(E::C)==3);}",
    ),
    (
        "large-tags",
        "#[repr(u64)]enum E{A=0,B=18446744073709551615}fn f(e:E)->u8{match e{E::A=>1,E::B=>2}}fn main(){check(f(E::B)==2);}",
    ),
    (
        "payload-offsets",
        "enum E{A(u8,u64),B(u64,u8),C}fn f(e:E)->u64{match e{E::A(a,b)=>a as u64+b,E::B(b,a)=>b-a as u64,E::C=>0}}fn main(){check(f(E::A(3,100))==103&&f(E::B(200,7))==193&&f(E::C)==0);}",
    ),
    (
        "repr-c-payloads",
        "#[repr(C)]enum E{A(u8,u64),B(u64,u8)}fn f(e:E)->u64{match e{E::A(a,b)=>a as u64+b,E::B(b,a)=>b-a as u64}}fn main(){check(f(E::A(3,100))==103&&f(E::B(200,7))==193);}",
    ),
    (
        "repr-u8-payloads",
        "#[repr(u8)]enum E{A(u8,u64)=7,B(u64,u8)=9}fn f(e:E)->u64{match e{E::A(a,b)=>a as u64+b,E::B(b,a)=>b-a as u64}}fn main(){check(f(E::A(3,100))==103&&f(E::B(200,7))==193);}",
    ),
    (
        "variant-or-captures",
        "enum E{A(u8,u64),B(u64,u8)}fn f(e:E)->u64{match e{E::A(a,b)|E::B(b,a)=>a as u64+b}}fn main(){check(f(E::A(3,100))==103&&f(E::B(200,7))==207);}",
    ),
    (
        "tuple-or-captures",
        "fn f(x:(u8,u8))->u8{match x{(0,n)|(n,0)=>n,_=>99}}fn main(){check(f((0,5))==5&&f((7,0))==7&&f((8,9))==99&&f((0,0))==0);}",
    ),
    (
        "first-alternative",
        "fn f(x:(u8,u8))->u8{match x{(n,_)|(_,n)=>n}}fn main(){check(f((7,9))==7);}",
    ),
    (
        "nested-or-captures",
        "fn f(x:((u8,u8),(u8,u8)))->u8{match x{((0,a)|(a,0),(0,b)|(b,0))=>a+b,_=>99}}fn main(){check(f(((0,3),(0,5)))==8&&f(((4,0),(0,6)))==10&&f(((4,0),(7,0)))==11&&f(((1,2),(0,3)))==99);}",
    ),
    (
        "parent-or-choice",
        "fn f(x:((u8,u8),u8))->u8{match x{((0,n)|(n,0),1)|((n,0),2)=>n,_=>99}}fn main(){check(f(((0,7),1))==7&&f(((8,0),1))==8&&f(((9,0),2))==9&&f(((0,7),2))==99);}",
    ),
    (
        "record-order",
        "struct S{a:u8,b:u64}fn f(x:S)->u64{match x{S{b:n,a:1}=>n,S{a:n,b:m}=>m+n as u64}}fn main(){check(f(S{a:1,b:99})==99&&f(S{a:3,b:55})==58);}",
    ),
    (
        "record-variant",
        "enum E{A{a:u8,b:u64},B{b:u8,a:u64}}fn f(e:E)->u64{match e{E::A{b:n,a:1}=>n,E::A{a:n,b:m}=>m+n as u64,E::B{a:n,b:m}=>n-m as u64}}fn main(){check(f(E::A{a:1,b:99})==99&&f(E::A{a:3,b:55})==58&&f(E::B{a:90,b:9})==81);}",
    ),
    (
        "nested-enum",
        "enum E{A(u8),B}struct S{e:E,n:u8}fn f(x:S)->u8{match x{S{e:E::A(0|1),n}=>n,S{e:E::A(n),..}=>n,S{e:E::B,..}=>99}}fn main(){check(f(S{e:E::A(1),n:7})==7&&f(S{e:E::A(8),n:3})==8&&f(S{e:E::B,n:0})==99);}",
    ),
    (
        "array-rest",
        "fn f(x:[u8;4])->u8{match x{[0,middle@..,last]=>middle[0]+middle[1]+last,[first,..]=>first}}fn main(){check(f([0,2,3,4])==9&&f([8,2,3,4])==8);}",
    ),
    (
        "array-or-captures",
        "fn f(x:[u8;2])->u8{match x{[0,n]|[n,0]=>n,_=>99}}fn main(){check(f([0,7])==7&&f([8,0])==8&&f([8,9])==99);}",
    ),
    (
        "delayed-move",
        "struct N{n:u8}enum E{A(N,u8),B}fn f(x:E)->u8{match x{E::A(n,0)=>n.n,E::A(n,_)=>n.n+1,E::B=>99}}fn main(){check(f(E::A(N{n:7},0))==7&&f(E::A(N{n:8},2))==9&&f(E::B)==99);}",
    ),
    (
        "move-or-captures",
        "struct N{n:u8}fn f(x:(N,N,bool))->u8{match x{(n,_,true)|(_,n,false)=>n.n}}fn main(){check(f((N{n:3},N{n:8},true))==3&&f((N{n:4},N{n:9},false))==9);}",
    ),
    (
        "at-copy-before-move",
        "struct N{n:u8}fn f(x:N)->u8{match x{whole@N{n:1..=10}=>whole.n,whole@N{n}=>whole.n+n}}fn main(){check(f(N{n:7})==7&&f(N{n:20})==40);}",
    ),
    (
        "partial-product",
        "struct N{n:u8}fn f(x:(N,u8))->u8{let (n,_)=x;match x{(_,v)=>n.n+v}}fn main(){check(f((N{n:7},9))==16);}",
    ),
    (
        "single-uninitialized",
        "enum E{A(u8)}fn main(){let e:E;match e{E::A(_)=>()}check(true);}",
    ),
    (
        "tuple-uninitialized",
        "fn main(){let x:(u8,u8);match x{(_|0,_)=>()}check(true);}",
    ),
    (
        "empty-variant",
        "enum V{}enum E{A(V),B}fn f(e:E)->u8{match e{E::B=>7}}fn main(){check(f(E::B)==7);}",
    ),
    (
        "discard-void",
        "enum V{}fn main(){let x:V;match x{_=>()}check(true);}",
    ),
    (
        "scrutinee-once",
        "fn main(){let mut n=0;let x=match {n+=1;(n,7)}{(1,v)=>v,_=>9};check(n==1&&x==7);}",
    ),
    (
        "loop-and-nested",
        "enum E{A(u8),B}fn f()->u8{let mut n=2;loop{match E::A(n){E::A(0)=>break 7,E::A(v)=>{n=match (v,true){(v,true)=>v-1,_=>0};continue},E::B=>return 9}}}fn main(){check(f()==7);}",
    ),
    (
        "float-wildcard",
        "fn f(x:(f32,u8))->u8{match x{(_,n)=>n}}fn main(){check(f((2.0,7))==7);}",
    ),
    (
        "zst-large-array",
        "fn f(x:[();1000000000000])->u8{match x{[(),..,()]=>7}}fn main(){check(f([();1000000000000])==7);}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "partial-reachable",
        "struct N;fn f(x:(N,N,bool)){let a=x.0;match x{(_,n,true)|(n,_,false)=>{let _=n;}}let _=a;}fn main(){}",
    ),
    (
        "partial-redundant",
        "struct N;fn main(){let x=(N,N,3u8);let a=x.0;match x{(_,n,0..=254)|(_,n,255)|(n,_,_)=>{let _=n;}}let _=a;}",
    ),
    (
        "partial-unreachable-range",
        "struct N;fn main(){let x=(N,N,3u8);let a=x.0;match x{(_,n,0..=255)|(n,_,_)=>{let _=n;}}let _=a;}",
    ),
    (
        "partial-unreachable",
        "struct N{n:u8}fn main(){let x=(N{n:3},N{n:7});let a=x.0;let b=match x{(_,n)|(n,_)=>n};let _=a.n+b.n;}",
    ),
    (
        "tag-uninitialized",
        "enum E{A(u8),B}fn main(){let e:E;match e{E::A(_)=>(),E::B=>()}}",
    ),
    (
        "tag-after-move",
        "struct N;enum E{A(N),B}fn f(e:E){match e{E::A(n)=>{let _=n;},E::B=>()};match e{E::A(_)=>(),E::B=>()}}fn main(){}",
    ),
    (
        "payload-after-move",
        "struct N;enum E{A(N),B}fn f(e:E){match e{E::A(n)=>{let _=n;},E::B=>()};match e{E::A(n)=>{let _=n;},E::B=>()}}fn main(){}",
    ),
    (
        "product-read",
        "fn main(){let x:(u8,u8);match x{(0|_,_)=>()}}",
    ),
    (
        "array-read",
        "fn main(){let x:[u8;2];match x{[0,_]=>(),_=>()}}",
    ),
    ("void-read", "enum V{}fn main(){let x:V;match x{}}"),
    ("empty-product", "enum V{}fn main(){let x:(V,u8);match x{}}"),
    (
        "overlapping-moves",
        "struct N;enum E{A(N),B}fn f(x:E){match x{whole@E::A(n)=>{let _=whole;let _=n;},E::B=>()}}fn main(){}",
    ),
];
fn folder(name: &str) -> Result<PathBuf, std::io::Error> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work")
        .join(name)
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
#[test]
fn constructors_select_values_without_early_moves() -> Result<(), Box<dyn std::error::Error>> {
    let folder = folder("constructor-execution")?;
    for (name, source) in ACCEPTED {
        let file = folder.join(format!("{name}.rs"));
        std::fs::write(&file, format!("{CHECK}{source}"))?;
        let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&file)
            .output()?;
        std::fs::write(file.with_extension("lowir"), &result.stdout)?;
        assert!(
            result.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stderr.is_empty());
        let again = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&file)
            .output()?;
        assert_eq!(result.stdout, again.stdout, "{name}");
        assert!(again.status.success() && again.stderr.is_empty());
        let executable = folder.join(name);
        let independent = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "constructor_values",
                "-C",
                "panic=abort",
                "-C",
                "overflow-checks=yes",
            ])
            .arg(&file)
            .arg("-o")
            .arg(&executable)
            .output()?;
        std::fs::write(file.with_extension("rustc.stderr"), &independent.stderr)?;
        assert!(
            independent.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        let ran = Command::new(executable).output()?;
        assert!(
            ran.status.success() && ran.stdout.is_empty() && ran.stderr.is_empty(),
            "{name}: {ran:?}"
        );
    }
    Ok(())
}
#[test]
fn constructor_reads_and_moves_reject_invalid_places() -> Result<(), Box<dyn std::error::Error>> {
    let folder = folder("constructor-errors")?;
    for (name, source) in REJECTED {
        let file = folder.join(format!("{name}.rs"));
        std::fs::write(&file, source)?;
        let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&file)
            .output()?;
        assert_eq!(result.status.code(), Some(1), "{name}");
        assert!(result.stdout.is_empty() && !result.stderr.is_empty());
        std::fs::write(file.with_extension("candidate.stderr"), &result.stderr)?;
        let independent = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "constructor_errors",
                "--emit=metadata",
            ])
            .arg(&file)
            .arg("-o")
            .arg(folder.join("metadata"))
            .output()?;
        std::fs::write(file.with_extension("rustc.stderr"), &independent.stderr)?;
        assert_eq!(
            independent.status.code(),
            Some(1),
            "{name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
    }
    Ok(())
}
#[test]
fn decision_and_capture_graphs_do_not_expand_products_of_alternatives()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = folder("constructor-cfg-scaling")?;
    let mut previous = None;
    for width in [8, 32, 128] {
        let types = vec!["(u8,u8)"; width].join(",");
        let patterns = (0..width)
            .map(|i| format!("(0,n{i})|(n{i},0)"))
            .collect::<Vec<_>>()
            .join(",");
        let file = folder.join(format!("columns{width}.rs"));
        std::fs::write(
            &file,
            format!("fn f(x:({types}))->u8{{match x{{({patterns})=>n0,_=>99}}}}"),
        )?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--count-body-ir")
            .arg(&file)
            .output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let view = std::str::from_utf8(&output.stdout)?;
        let blocks: usize = view
            .split("blocks=")
            .nth(1)
            .ok_or("missing block count")?
            .trim()
            .parse()?;
        assert!(blocks < 25 * width, "{width}: {blocks}");
        if let Some(previous) = previous {
            assert!(blocks < previous * 5);
        }
        previous = Some(blocks);
        std::fs::write(file.with_extension("counts"), &output.stdout)?;
    }
    Ok(())
}
