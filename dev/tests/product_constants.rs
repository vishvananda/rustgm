use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

const CHECK: &str = "fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}\n";
const ACCEPTED: &[(&str, &str)] = &[
    (
        "empty-block-unit",
        "const C:()={};fn f(C:())->u8{7}fn main(){check(f(())==7);}",
    ),
    (
        "unit",
        "const C:()=();fn f(C:())->u8{7}fn main(){let C=();check(f(())==7);}",
    ),
    (
        "tuple",
        "const C:(u8,bool)=(7,true);fn f(x:(u8,bool))->u8{match x{C=>1,_=>2}}fn main(){check(f((7,true))==1&&f((7,false))==2&&f((0,true))==2);}",
    ),
    (
        "tuple-partition",
        "const A:(bool,bool)=(false,false);const B:(bool,bool)=(false,true);const C:(bool,bool)=(true,false);const D:(bool,bool)=(true,true);fn f(x:(bool,bool))->u8{match x{A|D=>1,B|C=>2}}fn main(){check(f((false,false))==1&&f((true,true))==1&&f((true,false))==2&&f((false,true))==2);}",
    ),
    (
        "array",
        "const C:[u8;2]=[2,7];fn f(x:[u8;2])->u8{match x{C=>1,_=>2}}fn main(){check(f([2,7])==1&&f([7,2])==2&&f([2,2])==2&&f([7,7])==2);}",
    ),
    (
        "empty-array",
        "const C:[bool;0]=[];fn f(C:[bool;0])->u8{7}fn main(){let C=[];check(f([])==7);}",
    ),
    (
        "repeated-array",
        "const C:[u8;4]=[7;4];fn f(x:[u8;4])->u8{match x{C=>1,_=>2}}fn main(){check(f([7,7,7,7])==1&&f([0,7,7,7])==2&&f([7,0,7,7])==2&&f([7,7,0,7])==2&&f([7,7,7,0])==2);}",
    ),
    (
        "nested",
        "const C:([u8;2],(bool,()))=([2,7],(true,()));fn f(x:([u8;2],(bool,())))->u8{match x{C=>1,_=>2}}fn main(){check(f(([2,7],(true,())))==1&&f(([2,7],(false,())))==2&&f(([7,2],(true,())))==2);}",
    ),
    (
        "shared-products",
        "const C:[(u8,bool);2]=[(7,true);2];fn f(x:[(u8,bool);2])->u8{match x{C=>1,_=>2}}fn main(){check(f([(7,true),(7,true)])==1&&f([(0,true),(7,true)])==2&&f([(7,true),(0,true)])==2&&f([(7,true),(7,false)])==2);}",
    ),
    (
        "nested-repeats",
        "const C:[[u8;2];3]=[[7;2];3];fn f(x:[[u8;2];3])->u8{match x{C=>1,_=>2}}fn main(){check(f([[7;2];3])==1&&f([[7,7],[0,7],[7,7]])==2&&f([[7,7],[7,7],[7,0]])==2);}",
    ),
    (
        "forward-chain",
        "const C:(u8,bool)=(N,true);const N:u8=7;const D:(u8,bool)=C;fn f(x:(u8,bool))->u8{match x{D=>1,_=>2}}fn main(){check(f((7,true))==1&&f((7,false))==2);}",
    ),
    (
        "field-selection",
        "const C:(u8,bool)=(7,true);const D:u8=C.0;fn f(x:u8)->u8{match x{D=>1,_=>2}}fn main(){check(f(7)==1&&f(0)==2);}",
    ),
    (
        "index-selection",
        "const C:[u8;2]=[2,7];const D:u8=C[1];fn f(x:u8)->u8{match x{D=>1,_=>2}}fn main(){check(f(7)==1&&f(2)==2);}",
    ),
    (
        "nested-projection",
        "const C:([u8;2],bool)=([2,7],true);const D:u8=C.0[1];fn f(x:u8)->u8{match x{D=>1,_=>2}}fn main(){check(f(7)==1&&f(2)==2);}",
    ),
    (
        "lazy-product",
        "const C:(u8,bool)=if true{(7,true)}else{(1/0,false)};fn f(x:(u8,bool))->u8{match x{C=>1,_=>2}}fn main(){check(f((7,true))==1&&f((7,false))==2);}",
    ),
    (
        "guard-retries",
        "const C:(u8,bool)=(7,true);const D:(u8,bool)=C;fn main(){let mut n=0;match (7u8,true){C|D|(7,true) if {n+=1;false}=>(),_=>()}check(n==3);}",
    ),
    (
        "guard-capture",
        "const C:([u8;2],bool)=([2,7],true);fn f(x:([u8;2],bool))->u8{match x{v@C if v.0[1]==7=>v.0[0],_=>9}}fn main(){check(f(([2,7],true))==2&&f(([2,7],false))==9);}",
    ),
    (
        "conditional-chain",
        "const C:(u8,bool)=(7,true);fn f(x:(u8,bool))->u8{if let n@C=x&&n.1{n.0}else{0}}fn main(){check(f((7,true))==7&&f((7,false))==0);}",
    ),
    (
        "let-else",
        "const C:[u8;2]=[2,7];fn f(x:[u8;2])->u8{let n@C=x else{return 0;};n[1]}fn main(){check(f([2,7])==7&&f([7,2])==0);}",
    ),
    (
        "while-let",
        "const C:(u8,bool)=(7,true);fn main(){let mut x=(7u8,true);let mut n=0;while let C=x{n+=1;x=(0,false);}check(n==1);}",
    ),
    (
        "macro-pattern",
        "const C:[u8;2]=[2,7];macro_rules! m{($x:expr,$p:pat)=>{match $x{$p=>1,_=>2}}}fn main(){check(m!([2u8,7],C)==1&&m!([7u8,2],C)==2);}",
    ),
    (
        "module-constant",
        "mod m{pub const C:(u8,bool)=(7,true);}use m::C as D;fn f(x:(u8,bool))->u8{match x{D=>1,_=>2}}fn main(){check(f((7,true))==1&&f((7,false))==2);}",
    ),
    (
        "float-zero-capture",
        "const C:(f64,)=(0.0,);fn f(x:(f64,))->bool{match x{v@C=>1.0/v.0<0.0,_=>false}}fn main(){check(f((-0.0,))&&!f((0.0,))&&!f((1.0,)));}",
    ),
    (
        "empty-nan-repeat",
        "const C:[f32;0]=[0.0/0.0;0];fn main(){match []{C=>()}check(true);}",
    ),
    (
        "discriminant-product",
        "const C:(isize,bool)=(7,true);enum E{A=C.0,B}fn f(x:(isize,bool))->u8{match x{C=>1,_=>2}}fn main(){check(f((E::A as isize,true))==1&&f((E::B as isize,true))==2);}",
    ),
];
const REJECTED: &[&str] = &[
    "const C:(u8,bool)=(7,true);fn main(){let C=(7u8,true);}",
    "const C:(u8,bool)=(7,true);fn f(C:(u8,bool)){}fn main(){}",
    "const C:(bool,bool)=(true,true);fn main(){match (true,true){C=>()}}",
    "const C:[u8;2]=[2,7];fn main(){match [2u8,7]{C if true=>()}}",
    "const C:(f64,)=(0.0/0.0,);fn main(){match (0.0,){_=>(),C=>()}}",
    "const C:[f32;2]=[1.0,0.0/0.0];fn main(){match [1.0,0.0]{C=>(),_=>()}}",
    "const C:(u16,bool)=(7,true);fn main(){match (7u8,true){C=>(),_=>()}}",
    "const C:[u8;2]=[2,7];fn main(){match [2u8]{C=>(),_=>()}}",
    "const C:[u8;2]=[2,7];const D:u8=C[2];fn main(){match 7u8{D=>(),_=>()}}",
    "const C:(u8,bool)=C;fn main(){match (7u8,true){C=>(),_=>()}}",
    "const C:(u8,bool)=D;const D:(u8,bool)=C;fn main(){match (7u8,true){C=>(),_=>()}}",
    "const C:[u8;2]=[7,255+1];fn main(){match [7u8,0]{C=>(),_=>()}}",
    "const C:(u8,bool)=(1/0,true);fn main(){match (7u8,true){C=>(),_=>()}}",
    "struct S;const C:[S;0]=[];fn main(){match []{C=>()}}",
    "static C:(u8,bool)=(7,true);fn main(){match (7u8,true){C=>(),_=>()}}",
    "const C:(u8,bool)=(7,true);fn main(){let x:(u8,bool);match x{C=>(),_=>()}}",
    "const C:(u8,bool)=(7,true);fn main(){let mut x=(7u8,true);match x{C if {x.0=0;false}=>(),_=>()}}",
    "const C:(u8,bool)=(7,true);fn main(){let ref C=(7u8,true);}",
];
fn folder() -> Result<PathBuf, std::io::Error> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work/product-vectors")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
fn frontend(path: &Path, mode: &str) -> Result<Output, std::io::Error> {
    Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(mode)
        .arg(path)
        .output()
}
#[test]
fn product_constant_patterns_select_and_execute_with_rustc()
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
                let text = std::str::from_utf8(&emitted.stdout)?;
                let footer = text
                    .lines()
                    .last()
                    .and_then(|s| s.strip_prefix("eof "))
                    .ok_or("missing footer")?;
                assert_eq!(count.stdout, format!("{footer}\n").as_bytes());
            }
        }
        let executable = path.with_extension("rustc-exe");
        let rustc = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "product_patterns",
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
fn invalid_product_patterns_and_initializers_agree_with_rustc()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?;
    for (index, source) in REJECTED.iter().enumerate() {
        let path = directory.join(format!("reject-{index}.rs"));
        std::fs::write(&path, source)?;
        let emitted = frontend(&path, "--emit-lowir")?;
        let counted = frontend(&path, "--count-lowir")?;
        for output in [&emitted, &counted] {
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert!(output.stdout.is_empty() && !output.stderr.is_empty());
        }
        assert_eq!(emitted.stderr, counted.stderr);
        let rustc = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "product_reject",
                "--emit=metadata",
            ])
            .arg(&path)
            .arg("-o")
            .arg(path.with_extension("rmeta"))
            .output()?;
        std::fs::write(path.with_extension("rustc-stderr"), &rustc.stderr)?;
        assert_eq!(rustc.status.code(), Some(1), "{source}");
    }
    Ok(())
}

#[test]
fn product_values_share_repeats_and_bound_pattern_expansion()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?;
    for (name, source) in [
        (
            "compact-index",
            "const A:[u8;1000000000]=[7;1000000000];const C:u8=A[999999999];fn f(x:u8){match x{C=>(),_=>()}}",
        ),
        (
            "compact-pattern",
            "const A:[(u8,bool);64]=[(7,true);64];fn f(x:[(u8,bool);64]){match x{A=>(),_=>()}}",
        ),
    ] {
        let path = directory.join(format!("{name}.rs"));
        std::fs::write(&path, source)?;
        let emitted = frontend(&path, "--emit-patterns")?;
        assert!(
            emitted.status.success() && emitted.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&emitted.stderr)
        );
        let view = std::str::from_utf8(&emitted.stdout)?;
        assert!(view.lines().filter(|s| s.starts_with("pattern ")).count() < 10);
        if name == "compact-pattern" {
            assert_eq!(view.lines().filter(|s| s.contains(" product ")).count(), 2);
            let rustc = Command::new("rustc")
                .args([
                    "--edition=2024",
                    "--crate-type=lib",
                    "--crate-name",
                    "compact_pattern",
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
        }
    }
    // The billion-element constant is queried by index, never materialized. This
    // stress check makes no claim about rustc's resource use at that size.
    let path = directory.join("expansion-budget.rs");
    std::fs::write(
        &path,
        "const C:[u8;1000001]=[7;1000001];fn f(x:[u8;1000001]){match x{C=>(),_=>()}}",
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
fn product_patterns_keep_use_site_files_and_pending_borrows()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?.join("package");
    std::fs::create_dir_all(&directory)?;
    let path = directory.join("main.rs");
    std::fs::write(
        directory.join("values.rs"),
        "pub const C:([u8;2],bool)=([2,7],true);pub const UNIT:()=();",
    )?;
    std::fs::write(
        &path,
        format!(
            "{CHECK}mod values;fn f(x:([u8;2],bool))->u8{{match x{{values::C=>1,_=>2}}}}fn g(values::UNIT:())->u8{{7}}fn main(){{check(f(([2,7],true))==1&&f(([7,2],true))==2&&g(())==7);}}"
        ),
    )?;
    for stage in ["patterns", "lowir"] {
        let emitted = frontend(&path, &format!("--emit-{stage}"))?;
        assert!(
            emitted.status.success() && emitted.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&emitted.stderr)
        );
        std::fs::write(path.with_extension(stage), &emitted.stdout)?;
        if stage == "patterns" {
            let view = std::str::from_utf8(&emitted.stdout)?;
            assert!(
                view.lines()
                    .filter(|s| s.starts_with("pattern "))
                    .all(|s| s.contains("source=0:"))
            );
        }
    }
    let exe = directory.join("rustc");
    let rustc = Command::new("rustc")
        .args(["--edition=2024", "--crate-name", "product_files"])
        .arg(&path)
        .arg("-o")
        .arg(&exe)
        .output()?;
    assert!(
        rustc.status.success(),
        "{}",
        String::from_utf8_lossy(&rustc.stderr)
    );
    assert!(Command::new(exe).status()?.success());
    let path = directory.join("borrowed.rs");
    std::fs::write(
        &path,
        "const C:(u8,bool)=(7,true);fn f(x:&(u8,bool)){match x{&C=>(),_=>()}}fn main(){}",
    )?;
    let selected = frontend(&path, "--emit-patterns")?;
    assert!(selected.status.success() && selected.stderr.is_empty());
    let executable = frontend(&path, "--emit-lowir")?;
    assert_eq!(executable.status.code(), Some(1));
    assert!(executable.stdout.is_empty());
    assert!(String::from_utf8_lossy(&executable.stderr).contains("region and loan proofs"));
    Ok(())
}
