use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
    process::Command,
};
const CHECK: &str = "fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}\n";
const ACCEPTED: &[(&str, &str)] = &[
    (
        "irrefutable-or-else",
        "fn f(x:u8)->u8{let (n@0|n)=x else{return 99;};n}fn main(){check(f(0)==0&&f(7)==7);}",
    ),
    (
        "never-input-else",
        "fn f(){let x:u8;if let _=(return){}else{let _=x+1;}}fn main(){f();check(true);}",
    ),
    (
        "if-never",
        "enum E{A(u8),B}fn f()->u8{if let E::A(n)=(return 7){n}else{9}}fn main(){check(f()==7);}",
    ),
    (
        "let-never",
        "enum E{A(u8),B}fn f()->u8{let E::A(n)=return 7 else{return 9;};n}fn main(){check(f()==7);}",
    ),
    (
        "diverging-condition-types",
        "fn a()->!{if let _=loop{} {};}fn b()->!{if loop{}&&let _=0u8 {};}fn c(x:u8)->u8{let n@0=x else{if let _=loop{} {};};n}fn main(){check(true);}",
    ),
    (
        "huge-unreachable-else",
        "fn main(){let _=0u8 else{let huge:[u8;1000000000];let _=huge;loop{}};check(true);}",
    ),
    (
        "let-else-scalar",
        "fn f(x:u8)->u8{let n@1..=9=x else{return 99;};n+1}fn main(){check(f(3)==4&&f(10)==99);}",
    ),
    (
        "let-else-enum",
        "enum E{A(u8),B}fn f(x:E)->u8{let E::A(n)=x else{return 99;};n}fn main(){check(f(E::A(7))==7&&f(E::B)==99);}",
    ),
    (
        "let-else-or",
        "enum E{A(u8,u64),B(u64,u8),C}fn f(x:E)->u64{let (E::A(a,b)|E::B(b,a))=x else{return 99;};a as u64+b}fn main(){check(f(E::A(3,100))==103&&f(E::B(200,7))==207&&f(E::C)==99);}",
    ),
    (
        "let-else-failure-move",
        "struct N{n:u8}enum E{A(N),B(N)}fn other(x:E)->u8{match x{E::A(n)|E::B(n)=>n.n+1}}fn f(x:E)->u8{let E::A(n)=x else{return other(x);};n.n}fn main(){check(f(E::A(N{n:7}))==7&&f(E::B(N{n:8}))==9);}",
    ),
    (
        "let-else-shadow",
        "fn f(x:u8)->u8{let n=17;let n@0..=9=x else{return n;};n}fn main(){check(f(3)==3&&f(20)==17);}",
    ),
    (
        "let-else-break",
        "fn f(x:u8)->u8{'out:{let n@0..=9=x else{break 'out 99;};n}}fn main(){check(f(3)==3&&f(20)==99);}",
    ),
    (
        "let-else-continue",
        "fn f()->u8{let mut n=4;loop{n-=1;let 0=n else{continue;};break 7;}}fn main(){check(f()==7);}",
    ),
    (
        "let-else-nested-failure",
        "fn f(x:u8,y:u8)->u8{let n@0..=9=x else{let m@0..=9=y else{return 99;};return m+10;};n}fn main(){check(f(3,5)==3&&f(20,5)==15&&f(20,20)==99);}",
    ),
    (
        "let-else-initializer-once",
        "fn main(){let mut calls=0;let (1,n)=({calls+=1;(calls,7)}) else{return;};check(calls==1&&n==7);}",
    ),
    (
        "let-else-annotation",
        "fn main(){let n:u8=7 else{return;};check(n==7);}",
    ),
    (
        "let-else-wildcard",
        "fn main(){let x:u8;let _=x else{return;};let 0..=255=x else{return;};check(true);}",
    ),
    (
        "let-else-array",
        "fn f(x:[u8;4])->u8{let [0,middle@..,last]=x else{return 99;};middle[0]+middle[1]+last}fn main(){check(f([0,2,3,4])==9&&f([1,2,3,4])==99);}",
    ),
    (
        "let-else-macro",
        "macro_rules! bind{($n:ident,$x:expr)=>{let (0,$n)=$x else{return 99;};}}fn f(x:(u8,u8))->u8{bind!(n,x);n}fn main(){check(f((0,7))==7&&f((1,7))==99);}",
    ),
    (
        "if-let-enum",
        "enum E{A(u8),B}fn f(x:E)->u8{if let E::A(n)=x{n}else{99}}fn main(){check(f(E::A(7))==7&&f(E::B)==99);}",
    ),
    (
        "if-let-or",
        "fn f(x:(u8,u8))->u8{if let (0,n)|(n,0)=x{n}else{99}}fn main(){check(f((0,7))==7&&f((8,0))==8&&f((8,9))==99);}",
    ),
    (
        "if-let-outer-shadow",
        "enum E{A(u8),B}fn f(x:E)->u8{let n=17;if let E::A(n)=x{n}else{n}}fn main(){check(f(E::A(7))==7&&f(E::B)==17);}",
    ),
    (
        "if-let-object-join",
        "struct N(u8);enum E{A(N),B}fn f(x:E)->u8{let n=if let E::A(n)=x{n}else{N(99)};n.0}fn main(){check(f(E::A(N(7)))==7&&f(E::B)==99);}",
    ),
    (
        "if-let-failed-move",
        "struct N(u8);fn f(x:(N,bool))->u8{if let (n,true)=x{n.0}else{x.0.0+1}}fn main(){check(f((N(7),true))==7&&f((N(8),false))==9);}",
    ),
    (
        "if-let-wildcard",
        "fn main(){let x:u8;let y=if let _=x{7}else{9};let z=if let 0..=255=x{3}else{5};check(y==7&&z==3);}",
    ),
    (
        "if-let-mut",
        "fn f(x:u8)->u8{if let mut n@0..=9=x{n+=1;n}else{99}}fn main(){check(f(7)==8&&f(20)==99);}",
    ),
    (
        "else-if-let",
        "enum E{A(u8),B(u8),C}fn f(x:E)->u8{if let E::A(n)=x{n}else if let E::B(n)=x{n+1}else{99}}fn main(){check(f(E::A(7))==7&&f(E::B(8))==9&&f(E::C)==99);}",
    ),
    (
        "chain-dependencies",
        "enum E{A(u8),B}fn f(x:E,y:E)->u8{if let E::A(a)=x&&a>0&&let E::A(b)=y&&b>a{a+b}else{99}}fn main(){check(f(E::A(3),E::A(5))==8&&f(E::A(0),E::A(5))==99&&f(E::A(3),E::B)==99&&f(E::B,E::A(5))==99&&f(E::A(7),E::A(5))==99);}",
    ),
    (
        "chain-order",
        "fn f(first:bool,second:bool)->u8{let mut n=0;if first&&let x={n+=1;n}&&second&&let y={n+=10;n}{check(x==1&&y==11);}n}fn main(){check(f(false,true)==0&&f(true,false)==1&&f(true,true)==11);}",
    ),
    (
        "chain-shadow",
        "enum E{A(u8)}fn main(){let n=17;let out=if let E::A(n)=E::A(3)&&let E::A(n)=E::A(n+1)&&n==4{n}else{n};check(out==4&&n==17);}",
    ),
    (
        "chain-nested-boolean",
        "fn f(x:u8,ok:bool)->u8{if let n@0..=9=x&&(ok||n==7)&&true{n}else{99}}fn main(){check(f(7,false)==7&&f(3,true)==3&&f(3,false)==99);}",
    ),
    (
        "chain-failure-disjoint",
        "struct N(u8);fn f(x:(N,N,bool),ok:bool)->u8{if let (n,_,true)=x&&ok{n.0}else{x.1.0}}fn main(){check(f((N(3),N(7),true),false)==7&&f((N(3),N(7),false),true)==7&&f((N(3),N(7),true),true)==3);}",
    ),
    (
        "chain-macro",
        "macro_rules! choose{($x:expr)=>{if let n@0..=9=$x&&n>0{n}else{99}}}fn main(){check(choose!(7u8)==7&&choose!(0u8)==99);}",
    ),
    (
        "while-let",
        "fn f()->u8{let mut x=3u8;let mut sum=0;while let n@1..=3=x{sum+=n;x-=1;}sum}fn main(){check(f()==6);}",
    ),
    (
        "while-chain",
        "enum E{A(u8),B}fn next(n:u8)->E{if n>0{E::A(n)}else{E::B}}fn f()->u8{let mut x=5;let mut sum=0;while x>0&&let E::A(n)=next(x)&&n>1{sum+=n;x-=1;}sum}fn main(){check(f()==14);}",
    ),
    (
        "while-continue",
        "fn f()->u8{let mut x=4u8;let mut sum=0;while let mut n@1..=4=x{x-=1;if n==3{continue;}n+=1;sum+=n;}sum}fn main(){check(f()==10);}",
    ),
    (
        "while-break",
        "fn f()->u8{let mut x=4u8;while let n@1..=4=x{if n==2{break;}x-=1;}x}fn main(){check(f()==2);}",
    ),
    (
        "while-reinitialize-moves",
        "struct N(u8);enum E{A(N),B}fn f()->u8{let mut e=E::A(N(3));let mut sum=0;while let E::A(n)=e{sum+=n.0;e=if n.0>1{E::A(N(n.0-1))}else{E::B};}sum}fn main(){check(f()==6);}",
    ),
    (
        "nested-condition-transfer",
        "fn f()->u8{let mut x=3u8;'outer:loop{while let n@1..=3=x{if n==2{break 'outer 7;}x-=1;}break 99;}}fn main(){check(f()==7);}",
    ),
    (
        "while-wildcard-uninitialized",
        "fn main(){let x:u8;while let _=x{break;}check(true);}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "irrefutable-or-dead-else",
        "fn main(){let x:u8;let (0|_)=7u8 else{let _=x+1;return;};}",
    ),
    (
        "dead-init",
        "fn main(){let x:u8;if let _=0u8{}else{x=1;}x=2;let _=x;}",
    ),
    (
        "dead-move-after",
        "struct N;fn take(_:N){}fn main(){let x=N;if let _=0u8{}else{take(x);}take(x);}",
    ),
    (
        "irrefutable-dead-else-read",
        "fn main(){let x:u8;let _=0u8 else{let _=x+1;return;};}",
    ),
    (
        "if-dead-else-read",
        "fn main(){let x:u8;if let _=0u8{}else{let _=x+1;}}",
    ),
    (
        "irrefutable-dead-else-move",
        "struct N;fn main(){let x=N;let y=x;let _=0u8 else{let z=x;let _=z;return;};let _=y;}",
    ),
    (
        "if-dead-else-move",
        "struct N;fn main(){let x=N;let y=x;if let _=0u8{}else{let z=x;let _=z;}let _=y;}",
    ),
    (
        "condition-diverges-tail",
        "fn f()->!{if let _=loop{} {}}fn main(){}",
    ),
    (
        "chain-right-diverges",
        "fn f()->!{if let _=0u8&&loop{} {};}fn main(){}",
    ),
    (
        "while-own-continue",
        "fn main(){while let _={continue;} {}}",
    ),
    ("while-own-break", "fn main(){while let _={break;} {}}"),
    (
        "macro-let-condition",
        "macro_rules! p{($n:ident,$x:expr)=>{let $n=$x}}fn main(){if p!(n,7u8){let _=n;}}",
    ),
    ("else-does-not-diverge", "fn main(){let 0=1u8 else{};}"),
    ("else-returns-value", "fn main(){let 0=1u8 else{3};}"),
    (
        "else-conditional-return",
        "fn f(x:u8){let 0=x else{if x>0{return;}};}fn main(){}",
    ),
    (
        "else-loop-break",
        "fn main(){let 0=1u8 else{loop{break;}};}",
    ),
    (
        "else-new-binding-scope",
        "fn main(){let n@0=1u8 else{let _=n;return;};}",
    ),
    (
        "if-let-else-scope",
        "fn main(){if let n@0=1u8{}else{let _=n;}}",
    ),
    (
        "if-let-outside-scope",
        "fn main(){if let n@0=1u8{}let _=n;}",
    ),
    (
        "while-outside-scope",
        "fn main(){while let n@0=1u8{break;}let _=n;}",
    ),
    ("chain-before-binding", "fn main(){if n==0&&let n=0u8{}}"),
    ("chain-scrutinee-before-binding", "fn main(){if let n=n{}}"),
    (
        "chain-failure-move",
        "struct N;fn f(x:(N,bool),ok:bool){if let (n,true)=x&&ok{let _=n;}else{let y=x.0;let _=y;}}fn main(){}",
    ),
    (
        "while-move-without-reinit",
        "struct N;enum E{A(N),B}fn f(e:E){while let E::A(n)=e{let _=n;}}fn main(){}",
    ),
    (
        "uninitialized-condition",
        "fn main(){let x:u8;if let 0=x{}}",
    ),
    (
        "uninitialized-else",
        "fn main(){let x:u8;let 0=x else{return;};}",
    ),
    ("condition-not-bool", "fn main(){if let n=1u8&&n{}}"),
    (
        "different-arm-types",
        "fn main(){let _=if let n=1u8{n}else{false};}",
    ),
    (
        "or-missing-binding",
        "fn main(){if let (0,n)|(1,_ )=(1u8,2u8){let _=n;}}",
    ),
    ("conditional-parameter", "fn f(0:u8){}fn main(){}"),
    ("chain-or", "fn main(){if let n=1u8||true{let _=n;}}"),
    ("let-expression", "fn main(){let x=let n=1u8;let _=x;}"),
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
fn conditional_programs_agree_with_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let dir = folder("conditional-execution")?;
    for (name, source) in ACCEPTED {
        let file = dir.join(format!("{name}.rs"));
        std::fs::write(&file, format!("{CHECK}{source}"))?;
        let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&file)
            .output()?;
        std::fs::write(file.with_extension("lowir"), &result.stdout)?;
        std::fs::write(file.with_extension("candidate.stderr"), &result.stderr)?;
        assert!(
            result.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stderr.is_empty());
        let exe = dir.join(name);
        let independent = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "conditional_values",
                "-C",
                "panic=abort",
                "-C",
                "overflow-checks=yes",
            ])
            .arg(&file)
            .arg("-o")
            .arg(&exe)
            .output()?;
        std::fs::write(file.with_extension("rustc.stderr"), &independent.stderr)?;
        assert!(
            independent.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        let ran = Command::new(exe).output()?;
        assert!(
            ran.status.success() && ran.stdout.is_empty() && ran.stderr.is_empty(),
            "{name}: {ran:?}"
        );
        for stage in ["patterns", "lowir"] {
            let first = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(format!("--emit-{stage}"))
                .arg(&file)
                .output()?;
            let again = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(format!("--emit-{stage}"))
                .arg(&file)
                .output()?;
            assert!(first.status.success() && again.status.success());
            assert_eq!(first.stdout, again.stdout);
            let count = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(format!("--count-{stage}"))
                .arg(&file)
                .output()?;
            assert!(count.status.success());
            if stage == "patterns" {
                let view = std::str::from_utf8(&first.stdout)?;
                assert!(view.contains("condition source="));
                let footer = view
                    .lines()
                    .last()
                    .and_then(|s| s.strip_prefix("eof "))
                    .ok_or("missing footer")?;
                assert_eq!(count.stdout, format!("{footer}\n").as_bytes());
            }
        }
    }
    Ok(())
}
#[test]
fn conditional_rejections_agree_with_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let dir = folder("conditional-errors")?;
    for (name, source) in REJECTED {
        let file = dir.join(format!("{name}.rs"));
        std::fs::write(&file, source)?;
        let candidate = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&file)
            .output()?;
        std::fs::write(file.with_extension("candidate.stderr"), &candidate.stderr)?;
        assert_eq!(candidate.status.code(), Some(1), "{name}");
        assert!(candidate.stdout.is_empty() && !candidate.stderr.is_empty());
        let independent = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "conditional_rejections",
                "--emit=metadata",
            ])
            .arg(&file)
            .arg("-o")
            .arg(dir.join("metadata"))
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
fn conditional_body_storage_grows_linearly() -> Result<(), Box<dyn std::error::Error>> {
    let dir = folder("conditional-structure")?;
    for width in [16, 64, 256, 1024] {
        let chain = (0..width)
            .map(|i| {
                if i == 0 {
                    "let n0@0..=9=x".to_owned()
                } else {
                    format!("let n{i}@0..=9=n{}", i - 1)
                }
            })
            .collect::<Vec<_>>()
            .join("&&");
        let mut declarations = String::new();
        for i in 0..width {
            let input = if i == 0 {
                "x".to_owned()
            } else {
                format!("n{}", i - 1)
            };
            write!(declarations, "let n{i}@0..=9={input} else{{return 99;}};")?;
        }
        for (name, body) in [
            ("chain", format!("if {chain}{{n{}}}else{{99}}", width - 1)),
            ("else", format!("{declarations}n{}", width - 1)),
        ] {
            let file = dir.join(format!("{name}{width}.rs"));
            std::fs::write(&file, format!("fn f(x:u8)->u8{{{body}}}"))?;
            let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg("--count-body-ir")
                .arg(&file)
                .output()?;
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let text = std::str::from_utf8(&output.stdout)?;
            let blocks: usize = text
                .split("blocks=")
                .nth(1)
                .ok_or("missing blocks")?
                .trim()
                .parse()?;
            assert!(blocks < 8 * width + 10, "{name}/{width}: {blocks}");
            let statements: usize = text
                .split("statements=")
                .nth(1)
                .ok_or("missing statements")?
                .split_whitespace()
                .next()
                .ok_or("missing statement count")?
                .parse()?;
            assert!(statements < 20 * width + 20, "{name}/{width}: {statements}");
            std::fs::write(file.with_extension("counts"), output.stdout)?;
        }
    }
    Ok(())
}

#[test]
fn borrowed_conditions_remain_obligations() -> Result<(), Box<dyn std::error::Error>> {
    let dir = folder("conditional-borrows")?;
    for (index, source) in [
        "fn f(x:&(u8,u8)){if let (a,b)=x{let _=a;let _=b;}}",
        "fn f(x:&(u8,u8)){let (a,b)=x else{loop{}};let _=a;let _=b;}",
        "fn f(x:&(u8,u8)){while let (a,b)=x&&true{let _=a;let _=b;break;}}",
    ]
    .iter()
    .enumerate()
    {
        let file = dir.join(format!("borrow{index}.rs"));
        std::fs::write(&file, format!("{source}fn main(){{}}"))?;
        let view = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-patterns")
            .arg(&file)
            .output()?;
        assert!(
            view.status.success(),
            "{}",
            String::from_utf8_lossy(&view.stderr)
        );
        assert!(String::from_utf8_lossy(&view.stdout).contains("loan=pending"));
        let executable = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&file)
            .output()?;
        assert_eq!(executable.status.code(), Some(1));
        assert!(executable.stdout.is_empty());
        assert!(String::from_utf8_lossy(&executable.stderr).contains("reference lowering"));
        let independent = Command::new("rustc")
            .args(["--edition=2024", "--crate-type=lib", "--emit=metadata"])
            .arg(&file)
            .arg("-o")
            .arg(dir.join("metadata"))
            .output()?;
        std::fs::write(file.with_extension("stderr"), &independent.stderr)?;
        assert!(independent.status.success());
    }
    Ok(())
}

#[test]
fn imaginary_branches_do_not_allocate_runtime_storage() -> Result<(), Box<dyn std::error::Error>> {
    let dir = folder("conditional-storage-elision")?;
    let file = dir.join("huge.rs");
    std::fs::write(
        &file,
        "fn main(){let _=0u8 else{let huge:[u8;1000000000];let _=huge;loop{}};}",
    )?;
    let body = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-body-ir")
        .arg(&file)
        .output()?;
    assert!(body.status.success());
    assert!(String::from_utf8_lossy(&body.stdout).contains("false-edge"));
    let lowir = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-lowir")
        .arg(&file)
        .output()?;
    assert!(lowir.status.success());
    assert!(!String::from_utf8_lossy(&lowir.stdout).contains("obj<1000000000"));
    Ok(())
}
