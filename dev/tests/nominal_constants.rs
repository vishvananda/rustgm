use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

const CHECK: &str = "fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}\n";
const ACCEPTED: &[(&str, &str)] = &[
    (
        "unit",
        "#[derive(PartialEq)]struct S;const C:S=S;fn f(C:S)->u8{7}fn main(){let C=S;check(f(S)==7);}",
    ),
    (
        "empty-tuple",
        "#[derive(PartialEq)]struct S();const C:S=S();fn f(C:S)->u8{7}fn main(){let C=S();check(f(S())==7);}",
    ),
    (
        "empty-record-update",
        "#[derive(PartialEq)]struct S{}const C:S=S{..S{}};fn f(C:S)->u8{7}fn main(){let C=S{};check(f(S{})==7);}",
    ),
    (
        "tuple",
        "#[derive(PartialEq)]struct S(u8,bool);const C:S=S(7,true);fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S(7,true))==1&&f(S(7,false))==2&&f(S(0,true))==2);}",
    ),
    (
        "record-order",
        "#[derive(PartialEq)]struct S{a:u8,b:bool}const C:S=S{b:true,a:7};fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S{a:7,b:true})==1&&f(S{a:7,b:false})==2&&f(S{a:0,b:true})==2);}",
    ),
    (
        "update",
        "#[derive(PartialEq)]struct S{a:u8,b:bool}const BASE:S=S{a:2,b:true};const C:S=S{a:7,..BASE};fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S{a:7,b:true})==1&&f(S{a:2,b:true})==2&&f(S{a:7,b:false})==2);}",
    ),
    (
        "update-all-fields",
        "#[derive(PartialEq)]struct S{a:u8}const C:S=S{a:7,..S{a:2}};fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S{a:7})==1&&f(S{a:2})==2);}",
    ),
    (
        "projection",
        "struct T(u8,bool);struct S{a:T}const C:S=S{a:T(7,true)};const N:u8=C.a.0;fn f(x:u8)->u8{match x{N=>1,_=>2}}fn main(){check(f(7)==1&&f(0)==2);}",
    ),
    (
        "forward",
        "#[derive(PartialEq)]struct S(u8);const C:S=D;const D:S=S(N);const N:u8=7;fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S(7))==1&&f(S(0))==2);}",
    ),
    (
        "enum",
        "#[derive(PartialEq)]enum E{A(u8),B{b:bool},C}const A:E=E::A(7);const B:E=E::B{b:true};const C:E=E::C;fn f(x:E)->u8{match x{A=>1,B=>2,C=>3,_=>4}}fn main(){check(f(E::A(7))==1&&f(E::A(0))==4&&f(E::B{b:true})==2&&f(E::B{b:false})==4&&f(E::C)==3);}",
    ),
    (
        "enum-exhaustive",
        "#[derive(PartialEq)]enum E{A(bool),B}const A:E=E::A(true);const B:E=E::A(false);const C:E=E::B;fn f(x:E)->u8{match x{A=>1,B=>2,C=>3}}fn main(){check(f(E::A(true))==1&&f(E::A(false))==2&&f(E::B)==3);}",
    ),
    (
        "enum-zero-constructors",
        "#[derive(PartialEq)]enum E{A(),B{},C}const A:E=E::A();const B:E=E::B{};const C:E=E::C;fn f(x:E)->u8{match x{A=>1,B=>2,C=>3}}fn main(){check(f(E::A())==1&&f(E::B{})==2&&f(E::C)==3);}",
    ),
    (
        "inactive-pointer",
        "#[derive(PartialEq)]enum E{Good(u8),Bad(fn())}const C:E=E::Good(7);fn f(x:E)->u8{match x{C=>1,_=>2}}fn empty(){}fn main(){check(f(E::Good(7))==1&&f(E::Good(0))==2&&f(E::Bad(empty))==2);}",
    ),
    (
        "packed",
        "#[repr(packed)]#[derive(PartialEq)]struct S(u8,u64);const C:S=S(2,7);fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S(2,7))==1&&f(S(2,0))==2&&f(S(0,7))==2);}",
    ),
    (
        "nested-packed-copy",
        "#[derive(PartialEq,Copy,Clone)]struct X(u8);#[repr(packed)]#[derive(PartialEq)]struct S(X);const C:S=S(X(7));fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S(X(7)))==1&&f(S(X(0)))==2);}",
    ),
    (
        "eq",
        "#[derive(PartialEq,Eq)]struct S(u8);const C:S=S(7);fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S(7))==1&&f(S(0))==2);}",
    ),
    (
        "float",
        "#[derive(PartialEq)]struct S(f64);const C:S=S(0.0);fn f(x:S)->bool{match x{v@C=>1.0/v.0<0.0,_=>false}}fn main(){check(f(S(-0.0))&&!f(S(0.0))&&!f(S(1.0))&&!f(S(0.0/0.0)));}",
    ),
    (
        "empty-float",
        "#[derive(PartialEq)]struct S([f32;0]);const C:S=S([0.0/0.0;0]);fn f(C:S)->u8{7}fn main(){check(f(S([]))==7);}",
    ),
    (
        "empty-fn-array",
        "const C:[fn();0]=[];fn f(C:[fn();0])->u8{7}fn main(){check(f([])==7);}",
    ),
    (
        "private-fields",
        "mod m{#[derive(PartialEq)]pub struct S{a:u8}pub const C:S=S{a:7};pub fn new(a:u8)->S{S{a}}}fn f(x:m::S)->u8{match x{m::C=>1,_=>2}}fn main(){check(f(m::new(7))==1&&f(m::new(0))==2);}",
    ),
    (
        "noncopy-guard",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn f(x:S)->u8{match x{v@C if v.0>0=>v.0,S(n)=>n}}fn main(){check(f(S(7))==7&&f(S(0))==0);}",
    ),
    (
        "guard-retry",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn main(){let mut n=0;match S(7){v@C|v@S(7) if {n+=1;v.0==0}=>(),S(x)=>check(x==7)}check(n==2);}",
    ),
    (
        "guard-move-after-success",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn take(x:S)->u8{x.0}fn f(x:S)->u8{match x{v@C if v.0>0=>take(v),S(n)=>n}}fn main(){check(f(S(7))==7&&f(S(0))==0);}",
    ),
    (
        "conditional-guard",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn f(x:S)->u8{match x{v if let C=S(v.0)=>v.0,_=>0}}fn main(){check(f(S(7))==7&&f(S(2))==0);}",
    ),
    (
        "if-let",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn f(x:S)->u8{if let v@C=x&&v.0>0{v.0}else{0}}fn main(){check(f(S(7))==7&&f(S(2))==0);}",
    ),
    (
        "let-else",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn f(x:S)->u8{let v@C=x else{return 0;};v.0}fn main(){check(f(S(7))==7&&f(S(2))==0);}",
    ),
    (
        "while-let",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn main(){let mut x=S(7);let mut n=0;while let C=x{n+=1;x=S(0);}check(n==1);}",
    ),
    (
        "nested-repeat",
        "#[derive(PartialEq,Copy,Clone)]struct S{v:(u8,bool)}const C:[S;2]=[S{v:(7,true)};2];fn f(x:[S;2])->u8{match x{C=>1,_=>2}}fn main(){check(f([S{v:(7,true)};2])==1&&f([S{v:(0,true)},S{v:(7,true)}])==2&&f([S{v:(7,true)},S{v:(7,false)}])==2);}",
    ),
    (
        "lazy",
        "#[derive(PartialEq)]struct S(u8);const C:S=if true{S(7)}else{S(1/0)};fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S(7))==1&&f(S(0))==2);}",
    ),
    (
        "callee-group",
        "#[derive(PartialEq)]struct S(u8);const C:S=(S)(7);fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S(7))==1&&f(S(0))==2);}",
    ),
    (
        "callee-block",
        "#[derive(PartialEq)]struct S();const C:S=({S})();fn f(C:S)->u8{7}fn main(){check(f(S())==7);}",
    ),
    (
        "callee-lazy",
        "#[derive(PartialEq)]struct S(u8);const C:S=(if true{S}else{S})(7);fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S(7))==1&&f(S(0))==2);}",
    ),
    (
        "macro",
        "#[derive(PartialEq)]struct S(u8);macro_rules! make{($x:expr)=>{S($x)}}const C:S=make!(7);macro_rules! choose{($x:expr,$p:pat)=>{match $x{$p=>1,_=>2}}}fn main(){check(choose!(S(7),C)==1&&choose!(S(0),C)==2);}",
    ),
    (
        "attributed-field",
        "#[derive(PartialEq)]struct S{a:u8,b:bool}const C:S=S{#[cfg(any())]a:0,#[cfg(all())]b:true,a:7};fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S{a:7,b:true})==1&&f(S{a:0,b:true})==2);}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "mutual-non-equality-field",
        "#[derive(PartialEq)]struct A{b:&'static B,bad:X}#[derive(PartialEq)]struct B{a:&'static A}struct X;",
    ),
    (
        "packed-noncopy",
        "#[derive(PartialEq)]struct X(u8);#[repr(packed)]#[derive(PartialEq)]struct S(X);",
    ),
    (
        "packed-aligned-noncopy",
        "#[derive(PartialEq)]struct X(u8);#[repr(packed(8))]#[derive(PartialEq)]struct S(X);",
    ),
    ("eq-alone", "#[derive(Eq)]struct S(u8);"),
    ("eq-float", "#[derive(PartialEq,Eq)]struct S(f32);"),
    (
        "nan-field",
        "#[derive(PartialEq)]struct S(f32);const C:S=S(0.0/0.0);fn f(x:S){match x{C=>(),_=>()}}",
    ),
    (
        "no-derive",
        "struct S(u8);const C:S=S(7);fn f(x:S){match x{C=>(),_=>()}}",
    ),
    (
        "copy-not-equality",
        "#[derive(Copy,Clone)]struct S(u8);const C:S=S(7);fn f(x:S){match x{C=>(),_=>()}}",
    ),
    ("non-eq-field", "struct X;#[derive(PartialEq)]struct S(X);"),
    (
        "inactive-non-eq-field",
        "struct X;#[derive(PartialEq)]enum E{A,B(X)}const C:E=E::A;fn f(x:E){match x{C=>(),_=>()}}",
    ),
    (
        "duplicate-derive",
        "#[derive(PartialEq,PartialEq)]struct S;",
    ),
    (
        "nonexhaustive",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn f(x:S){match x{C=>()}}",
    ),
    (
        "refutable-param",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn f(C:S){}",
    ),
    (
        "refutable-let",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn f(x:S){let C=x;}",
    ),
    (
        "guard-not-exhaustive",
        "#[derive(PartialEq)]struct S();const C:S=S();fn f(x:S){match x{C if true=>()}}",
    ),
    (
        "different-nominal-type",
        "#[derive(PartialEq)]struct S(u8);struct T(u8);const C:S=S(7);fn f(x:T){match x{C=>(),_=>()}}",
    ),
    (
        "nested-nan",
        "#[derive(PartialEq)]struct S([f64;2]);const C:S=S([1.0,0.0/0.0]);fn f(x:S){match x{C=>(),_=>()}}",
    ),
    (
        "eq-empty-float",
        "#[derive(PartialEq,Eq)]struct S([f32;0]);",
    ),
    (
        "noeq-empty-array",
        "struct S;const C:[S;0]=[];fn f(x:[S;0]){match x{C=>()}}",
    ),
    (
        "cycle",
        "#[derive(PartialEq)]struct S(u8);const C:S=D;const D:S=C;fn f(x:S){match x{C=>(),_=>()}}",
    ),
    (
        "overflow",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(255+1);fn f(x:S){match x{C=>(),_=>()}}",
    ),
    (
        "move-in-guard",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn take(x:S)->bool{true}fn f(x:S){match x{v@C if take(v)=>(),_=>()}}",
    ),
    (
        "mutate-tested-input",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn f(mut x:S){match x{C if {x=S(0);false}=>(),_=>()}}",
    ),
    (
        "callee-effects-unit",
        "#[derive(PartialEq)]struct S();fn effect(){}const C:S=({effect();S})();fn f(x:S){match x{C=>()}}",
    ),
    (
        "callee-effects-value",
        "#[derive(PartialEq)]struct S(u8);fn effect(){}const C:S=({effect();S})(7);fn f(x:S){match x{C=>(),_=>()}}",
    ),
    (
        "callee-effects-discriminant",
        "enum E{A()}fn effect(){}const C:isize=({effect();E::A})() as isize;fn f(x:isize){match x{C=>(),_=>()}}",
    ),
    (
        "update-base-empty",
        "#[derive(PartialEq)]struct S{}fn base()->S{S{}}const C:S=S{..base()};fn f(x:S){match x{C=>()}}",
    ),
    (
        "update-base-all-fields",
        "#[derive(PartialEq)]struct S{a:u8}fn base()->S{S{a:0}}const C:S=S{a:7,..base()};fn f(x:S){match x{C=>(),_=>()}}",
    ),
];

fn folder() -> Result<PathBuf, std::io::Error> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../obj")
        .join(format!("nominal-constants-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}
fn frontend(path: &Path, mode: &str) -> Result<Output, std::io::Error> {
    Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(mode)
        .arg(path)
        .output()
}
#[test]
fn nominal_constant_patterns_select_and_execute_with_rustc()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?;
    for &(name, source) in ACCEPTED {
        let path = directory.join(format!("{name}.rs"));
        std::fs::write(&path, format!("{CHECK}{source}"))?;
        for stage in ["patterns", "lowir"] {
            let emitted = frontend(&path, &format!("--emit-{stage}"))?;
            assert!(
                emitted.status.success(),
                "{name}/{stage}: {}",
                String::from_utf8_lossy(&emitted.stderr)
            );
            assert!(emitted.stderr.is_empty());
            std::fs::write(path.with_extension(stage), &emitted.stdout)?;
            let repeated = frontend(&path, &format!("--emit-{stage}"))?;
            assert!(repeated.status.success() && repeated.stderr.is_empty());
            assert_eq!(repeated.stdout, emitted.stdout);
            let count = frontend(&path, &format!("--count-{stage}"))?;
            assert!(count.status.success() && count.stderr.is_empty());
            if stage == "patterns" {
                let view = std::str::from_utf8(&emitted.stdout)?;
                let footer = view
                    .lines()
                    .last()
                    .and_then(|s| s.strip_prefix("eof "))
                    .ok_or("missing selected-pattern footer")?;
                assert_eq!(count.stdout, format!("{footer}\n").as_bytes());
            }
        }
        let executable = path.with_extension("rustc-exe");
        let rustc = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "nominal_patterns",
                "-C",
                "panic=abort",
                "-C",
                "overflow-checks=yes",
            ])
            .arg(&path)
            .arg("-o")
            .arg(&executable)
            .output()?;
        std::fs::write(path.with_extension("rustc-stderr"), &rustc.stderr)?;
        assert!(
            rustc.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&rustc.stderr)
        );
        let ran = Command::new(executable).output()?;
        assert!(
            ran.status.success() && ran.stdout.is_empty() && ran.stderr.is_empty(),
            "{name}"
        );
    }
    Ok(())
}
#[test]
fn invalid_nominal_patterns_and_initializers_agree_with_rustc()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?;
    for &(name, source) in REJECTED {
        let path = directory.join(format!("reject-{name}.rs"));
        std::fs::write(&path, format!("{source}fn main(){{}}"))?;
        let emitted = frontend(&path, "--emit-lowir")?;
        let counted = frontend(&path, "--count-lowir")?;
        for output in [&emitted, &counted] {
            assert_eq!(output.status.code(), Some(1), "{name}");
            assert!(output.stdout.is_empty() && !output.stderr.is_empty());
        }
        assert_eq!(emitted.stderr, counted.stderr);
        let rustc = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "nominal_reject",
                "--emit=metadata",
            ])
            .arg(&path)
            .arg("-o")
            .arg(path.with_extension("rmeta"))
            .output()?;
        std::fs::write(path.with_extension("rustc-stderr"), &rustc.stderr)?;
        assert_eq!(rustc.status.code(), Some(1), "{name}");
    }
    Ok(())
}

#[test]
fn nominal_patterns_keep_use_sites_and_checked_borrows() -> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?.join("package");
    std::fs::create_dir_all(&directory)?;
    std::fs::write(
        directory.join("values.rs"),
        "#[derive(PartialEq)]pub struct S{a:u8}pub const C:S=S{a:7};pub fn new(a:u8)->S{S{a}}",
    )?;
    let path = directory.join("main.rs");
    std::fs::write(
        &path,
        format!(
            "{CHECK}mod values;use values::C as D;fn f(x:values::S)->u8{{match x{{D=>1,_=>2}}}}fn main(){{check(f(values::new(7))==1&&f(values::new(0))==2);}}"
        ),
    )?;
    for stage in ["patterns", "lowir"] {
        let output = frontend(&path, &format!("--emit-{stage}"))?;
        assert!(
            output.status.success() && output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::fs::write(path.with_extension(stage), &output.stdout)?;
        if stage == "patterns" {
            let view = std::str::from_utf8(&output.stdout)?;
            assert!(
                view.lines()
                    .filter(|s| s.starts_with("pattern "))
                    .all(|s| s.contains("source=0:"))
            );
            assert!(view.contains("nominal="));
        }
    }
    let executable = directory.join("rustc");
    let rustc = Command::new("rustc")
        .args(["--edition=2024", "--crate-name", "nominal_files"])
        .arg(&path)
        .arg("-o")
        .arg(&executable)
        .output()?;
    assert!(
        rustc.status.success(),
        "{}",
        String::from_utf8_lossy(&rustc.stderr)
    );
    let ran = Command::new(executable).output()?;
    assert!(ran.status.success() && ran.stdout.is_empty() && ran.stderr.is_empty());
    let path = directory.join("borrowed.rs");
    std::fs::write(
        &path,
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn f(x:&S){match x{&C=>(),_=>()}}fn main(){}",
    )?;
    let selected = frontend(&path, "--emit-patterns")?;
    assert!(selected.status.success() && selected.stderr.is_empty());
    let executable = frontend(&path, "--emit-lowir")?;
    assert!(executable.status.success() && executable.stderr.is_empty());
    let rustc = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-name",
            "nominal_borrow",
            "--emit=metadata",
        ])
        .arg(&path)
        .arg("-o")
        .arg(path.with_extension("rmeta"))
        .output()?;
    assert!(
        rustc.status.success(),
        "{}",
        String::from_utf8_lossy(&rustc.stderr)
    );
    Ok(())
}

#[test]
fn recursive_derive_obligations_and_operator_boundary_are_explicit()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?;
    for (name, source, boundary) in [
        (
            "recursive",
            "#[derive(PartialEq,Eq)]struct S{next:&'static S}",
            None,
        ),
        (
            "mutual",
            "#[derive(PartialEq,Eq)]struct A{b:&'static B}#[derive(PartialEq,Eq)]struct B{a:&'static A}",
            None,
        ),
        (
            "operator-boundary",
            "#[derive(PartialEq)]struct S(u8);fn f()->bool{S(7)==S(7)}",
            Some("operation requires a scalar type"),
        ),
    ] {
        let path = directory.join(format!("{name}.rs"));
        std::fs::write(&path, format!("{source}fn main(){{}}"))?;
        let output = frontend(&path, "--emit-patterns")?;
        if let Some(message) = boundary {
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr).contains(message));
        } else {
            assert!(
                output.status.success() && output.stderr.is_empty(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let rustc = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "nominal_obligations",
                "--emit=metadata",
            ])
            .arg(&path)
            .arg("-o")
            .arg(path.with_extension("rmeta"))
            .output()?;
        std::fs::write(path.with_extension("rustc-stderr"), &rustc.stderr)?;
        assert!(
            rustc.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&rustc.stderr)
        );
    }
    Ok(())
}

#[test]
fn nominal_repeated_values_share_patterns_and_bound_expansion()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?;
    let path = directory.join("shared-pattern.rs");
    std::fs::write(
        &path,
        "#[derive(PartialEq,Copy,Clone)]struct S(u8);const C:[S;64]=[S(7);64];fn f(x:[S;64]){match x{C=>(),_=>()}}",
    )?;
    let output = frontend(&path, "--emit-patterns")?;
    assert!(
        output.status.success() && output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let view = std::str::from_utf8(&output.stdout)?;
    assert!(view.lines().filter(|s| s.starts_with("pattern ")).count() < 10);
    assert_eq!(view.lines().filter(|s| s.contains(" product ")).count(), 2);
    assert_eq!(view.lines().filter(|s| s.contains("nominal=")).count(), 1);
    let rustc = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--crate-name",
            "nominal_repeat",
            "--emit=metadata",
        ])
        .arg(&path)
        .arg("-o")
        .arg(path.with_extension("rmeta"))
        .output()?;
    assert!(
        rustc.status.success(),
        "{}",
        String::from_utf8_lossy(&rustc.stderr)
    );
    // This size is a RustGM resource-limit check, not a rustc scaling comparison.
    let path = directory.join("expansion-budget.rs");
    std::fs::write(
        &path,
        "#[derive(PartialEq,Copy,Clone)]struct S(u8);const C:[S;1000001]=[S(7);1000001];fn f(x:[S;1000001]){match x{C=>(),_=>()}}",
    )?;
    let emitted = frontend(&path, "--emit-patterns")?;
    let counted = frontend(&path, "--count-patterns")?;
    for output in [&emitted, &counted] {
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("constant pattern expansion work budget exceeded")
        );
    }
    assert_eq!(emitted.stderr, counted.stderr);
    Ok(())
}

#[test]
fn constant_autoderef_follows_the_reference_with_explicit_rustc_discrepancy()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?.join("reference-autoderef");
    std::fs::create_dir_all(&directory)?;
    // patterns.ident.binding.non-reference excludes reference-typed constants.
    // rustc 1.98.0 instead requires an explicit & for every named constant here.
    for (name, declaration, ty, pattern, rustc_accepts) in [
        ("scalar", "const C:u8=7;", "u8", "C", false),
        (
            "tuple",
            "const C:(u8,bool)=(7,true);",
            "(u8,bool)",
            "C",
            false,
        ),
        ("array", "const C:[u8;2]=[2,7];", "[u8;2]", "C", false),
        (
            "nominal",
            "#[derive(PartialEq)]struct S(u8);const C:S=S(7);",
            "S",
            "C",
            false,
        ),
        (
            "enum",
            "#[derive(PartialEq)]enum E{A,B}const C:E=E::A;",
            "E",
            "C",
            false,
        ),
        ("unit", "const C:()=();", "()", "C", false),
        ("constructor-control", "struct S;", "S", "S", true),
    ] {
        for explicit in [false, true] {
            let path = directory.join(format!("{name}-{explicit}.rs"));
            let reference = if explicit { "&" } else { "" };
            std::fs::write(
                &path,
                format!(
                    "{declaration}fn f(x:&{ty}){{match x{{{reference}{pattern}=>(),_=>()}}}}fn main(){{}}"
                ),
            )?;
            let emitted = frontend(&path, "--emit-patterns")?;
            assert!(
                emitted.status.success() && emitted.stderr.is_empty(),
                "{name}: {}",
                String::from_utf8_lossy(&emitted.stderr)
            );
            let view = std::str::from_utf8(&emitted.stdout)?;
            assert!(view.contains(&format!("implicit={}", !explicit)));
            std::fs::write(path.with_extension("patterns"), &emitted.stdout)?;
            let rustc = Command::new("rustc")
                .args([
                    "--edition=2024",
                    "--crate-name",
                    "constant_autoderef",
                    "--emit=metadata",
                ])
                .arg(&path)
                .arg("-o")
                .arg(path.with_extension("rmeta"))
                .output()?;
            std::fs::write(path.with_extension("rustc-stderr"), &rustc.stderr)?;
            assert_eq!(
                rustc.status.code(),
                Some(i32::from(!explicit && !rustc_accepts)),
                "{name}"
            );
            if !explicit && !rustc_accepts {
                assert!(String::from_utf8_lossy(&rustc.stderr).contains("E0308"));
            }
        }
    }
    Ok(())
}
