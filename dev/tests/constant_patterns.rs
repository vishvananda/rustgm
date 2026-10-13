use std::{
    path::{Path, PathBuf},
    process::Command,
};

const CHECK: &str = "fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}\n";
const ACCEPTED: &[(&str, &str)] = &[
    (
        "signed-range",
        "const LO:i8=-5;const HI:i8=LO+8;fn f(x:i8)->u8{match x{LO..=HI=>7,_=>9}}fn main(){check(f(-5)==7&&f(3)==7&&f(4)==9);}",
    ),
    (
        "signed-extrema",
        "const MIN:i128=-170141183460469231731687303715884105728;const MAX:i128=170141183460469231731687303715884105727;fn f(x:i128)->u8{match x{MIN=>1,MAX=>2,_=>3}}fn main(){check(f(-170141183460469231731687303715884105728)==1&&f(170141183460469231731687303715884105727)==2&&f(-1)==3);}",
    ),
    (
        "unsigned-extrema",
        "const TOP:u128=!0u128;fn f(x:u128)->u8{match x{TOP=>7,_=>9}}fn main(){check(f(340282366920938463463374607431768211455)==7&&f(0)==9);}",
    ),
    (
        "boolean-exhaustive",
        "const YES:bool=true;const NO:bool=!YES;fn f(x:bool)->u8{match x{YES=>7,NO=>9}}fn main(){check(f(true)==7&&f(false)==9);}",
    ),
    (
        "character-gap",
        "const LO:char='\u{d7ff}';const HI:char='\u{e000}';fn f(x:char)->u8{match x{..=LO=>1,HI.. =>2}}fn main(){check(f('\u{0}')==1&&f('\u{d7ff}')==1&&f('\u{e000}')==2&&f('\u{10ffff}')==2);}",
    ),
    (
        "character-exclusive",
        "const HI:char='\u{e000}';fn f(x:char)->u8{match x{'\u{d7ff}'..HI=>1,_=>2}}fn main(){check(f('\u{d7ff}')==1&&f('\u{e000}')==2);}",
    ),
    (
        "full-range-uninitialized",
        "const LO:i8=-128;const HI:i8=127;fn main(){let x:i8;match x{LO..=HI=>()}check(true);}",
    ),
    (
        "irrefutable-parameter",
        "const LO:u8=0;const HI:u8=255;fn f(LO..=HI:u8)->u8{7}fn main(){check(f(3)==7);}",
    ),
    (
        "irrefutable-let",
        "const YES:bool=true;const NO:bool=false;fn main(){let (YES|NO)=true;check(true);}",
    ),
    (
        "forward-alias",
        "const A:u8=B+1;fn f(x:u8)->u8{match x{A=>7,_=>9}}const B:u8=C;const C:u8=2;fn main(){check(f(3)==7&&f(2)==9);}",
    ),
    (
        "import-qualified",
        "mod m{pub const A:u8=3;pub const B:u8=7;}use m::A as LO;fn f(x:u8)->u8{match x{LO..=m::B=>1,_=>2}}fn main(){check(f(3)==1&&f(7)==1&&f(8)==2);}",
    ),
    (
        "local-forward",
        "fn f(x:u8)->u8{let n=match x{N=>7,_=>9};const N:u8=3;n}fn main(){check(f(3)==7&&f(2)==9);}",
    ),
    (
        "macro-constant",
        "const N:u8=3;macro_rules! choose{($x:expr)=>{match $x{N=>7,_=>9}}}fn main(){check(choose!(3u8)==7&&choose!(4u8)==9);}",
    ),
    (
        "macro-pattern-fragment",
        "const N:u8=3;macro_rules! choose{($x:expr,$p:pat)=>{match $x{$p=>7,_=>9}}}fn main(){check(choose!(3u8,N|5)==7&&choose!(4u8,N|5)==9);}",
    ),
    (
        "lazy-branches",
        "const N:u8=if true{7}else{1/0};const B:bool=false&&1/0>0;const C:bool=true||1/0>0;fn f(x:u8)->bool{match x{N=>match true{B=>false,C=>true},_=>false}}fn main(){check(f(7)&&!f(6));}",
    ),
    (
        "typed-arithmetic",
        "const N:i8=((-7i8/2)+(-7i8%2))<<1;const M:u8=(256u16+3) as u8;fn f(x:i8)->u8{match x{N=>1,_=>2}}fn g(x:u8)->u8{match x{M=>3,_=>4}}fn main(){check(f(-8)==1&&f(-7)==2&&g(3)==3&&g(0)==4);}",
    ),
    (
        "float-to-integer",
        "const N:i8=(1.0f64/0.0) as i8;const M:i8=(-1.0f32/0.0) as i8;fn f(x:i8)->u8{match x{N=>1,M=>2,_=>3}}fn main(){check(f(127)==1&&f(-128)==2&&f(0)==3);}",
    ),
    (
        "discriminant-shared-cache",
        "const N:isize=7;enum E{A=N,B}fn f(x:isize)->u8{match x{N=>1,_=>2}}fn main(){check(f(E::A as isize)==1&&f(E::B as isize)==2);}",
    ),
    (
        "completed-discriminant",
        "enum E{A,B,C}const N:isize=E::C as isize;fn f(x:isize)->u8{match x{N=>1,_=>2}}fn main(){check(f(2)==1&&f(1)==2);}",
    ),
    (
        "enum-alias-cache",
        "#[repr(i8)]enum E{A=-2,B}const V:E=E::A;const N:u128=V as u128;fn f(x:u128)->u8{match x{N=>1,_=>2}}fn main(){check(f(340282366920938463463374607431768211454)==1&&f(254)==2);}",
    ),
    (
        "empty-enum-constructors",
        "enum E{A(),B{}}const A:isize=E::A() as isize;const B:isize=E::B{} as isize;fn f(x:isize)->u8{match x{A=>1,B=>2,_=>3}}fn main(){check(f(0)==1&&f(1)==2&&f(2)==3);}",
    ),
    (
        "scalar-in-product",
        "const N:u8=7;fn f(x:(u8,bool))->u8{match x{(N,true)=>1,(N,false)=>2,_=>3}}fn main(){check(f((7,true))==1&&f((7,false))==2&&f((0,true))==3);}",
    ),
    (
        "scalar-in-array",
        "const N:u8=7;fn f(x:[u8;2])->u8{match x{[N,N]=>1,[N,_]=>2,_=>3}}fn main(){check(f([7,7])==1&&f([7,0])==2&&f([0,7])==3);}",
    ),
    (
        "scalar-in-enum",
        "const N:u8=7;enum E{A(u8),B}fn f(x:E)->u8{match x{E::A(N)=>1,E::A(_)=>2,E::B=>3}}fn main(){check(f(E::A(7))==1&&f(E::A(0))==2&&f(E::B)==3);}",
    ),
    (
        "guard-retry",
        "const N:u8=7;const LO:u8=0;const HI:u8=9;fn main(){let mut c=0;match 7u8{N|LO..=HI|_ if {c+=1;false}=>(),_=>()}check(c==3);}",
    ),
    (
        "guard-conditional",
        "const N:u8=7;fn f(x:u8)->u8{match x{n if let N=n=>1,_=>2}}fn main(){check(f(7)==1&&f(0)==2);}",
    ),
    (
        "conditional-chain",
        "const N:u8=7;fn f(x:u8)->u8{if let n@N=x&&n>0{n}else{0}}fn main(){check(f(7)==7&&f(0)==0);}",
    ),
    (
        "while-let",
        "const N:u8=7;fn main(){let mut n=7u8;let mut c=0;while let N=n{c+=1;n=0;}check(c==1);}",
    ),
    (
        "let-else",
        "const N:u8=7;fn f(x:u8)->u8{let n@N=x else{return 0;};n}fn main(){check(f(7)==7&&f(0)==0);}",
    ),
    (
        "module-scope-coverage",
        "mod a{enum Void{}pub struct S{v:Void}const N:u8=0;pub fn f(x:S,n:u8){match n{N=>(),_=>()}match x{}}}mod b{const N:u8=1;pub fn f(x:u8)->u8{match x{N=>7,_=>9}}}fn main(){check(b::f(1)==7);}",
    ),
];
const REJECTED: &[&str] = &[
    "const N:u8=0;fn main(){match 0u8{N=>()}}",
    "const N:bool=true;fn main(){match true{N=>()}}",
    "const LO:u8=1;const HI:u8=255;fn main(){match 0u8{LO..=HI=>()}}",
    "const N:u8=7;fn main(){let N=7u8;}",
    "const N:u8=7;fn f(N:u8){}fn main(){}",
    "const LO:u8=7;const HI:u8=3;fn main(){match 0u8{LO..=HI=>(),_=>()}}",
    "const N:u8=7;fn main(){match 0u8{N..N=>(),_=>()}}",
    "const N:bool=false;fn main(){match true{N..=true=>(),_=>()}}",
    "const N:u16=7;fn main(){match 7u8{N=>(),_=>()}}",
    "const N:u16=7;fn main(){match 7u8{0..=N=>(),_=>()}}",
    "static N:u8=7;fn main(){match 7u8{N=>(),_=>()}}",
    "const N:u8=N;fn main(){match 7u8{N=>(),_=>()}}",
    "const N:u8=M;const M:u8=N;fn main(){match 7u8{N=>(),_=>()}}",
    "const N:u8=255+1;fn main(){match 7u8{N=>(),_=>()}}",
    "const N:i8=-128/-1;fn main(){match 7i8{N=>(),_=>()}}",
    "const N:u8=1<<8;fn main(){match 7u8{N=>(),_=>()}}",
    "const N:u8=1/0;fn main(){match 7u8{N=>(),_=>()}}",
    "const N:u8=if false{1}else{1/0};fn main(){match 7u8{N=>(),_=>()}}",
    "const N:u8=7;fn main(){let x:u8;match x{N=>(),_=>()}}",
    "const N:u8=7;fn main(){let mut x=7u8;match x{N if {x=0;false}=>(),_=>()}}",
    "const N:u8=7;fn main(){let ref N=7u8;}",
    "const N:u8=7;fn main(){match 0u8{N|x=>{let _=x;}}}",
    "mod a{enum Void{}pub struct S{v:Void}}const N:u8=0;fn f(x:a::S,n:u8){match n{N=>(),_=>()}match x{}}fn main(){}",
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
fn constant_pattern_values_agree_with_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let folder = folder("constant-vectors")?;
    for (name, source) in ACCEPTED {
        let file = folder.join(format!("{name}.rs"));
        std::fs::write(&file, format!("{CHECK}{source}"))?;
        let compiler = env!("CARGO_BIN_EXE_rustgm");
        for stage in ["patterns", "lowir"] {
            let first = Command::new(compiler)
                .arg(format!("--emit-{stage}"))
                .arg(&file)
                .output()?;
            assert!(
                first.status.success(),
                "{name}/{stage}: {}",
                String::from_utf8_lossy(&first.stderr)
            );
            assert!(first.stderr.is_empty());
            std::fs::write(file.with_extension(stage), &first.stdout)?;
            let again = Command::new(compiler)
                .arg(format!("--emit-{stage}"))
                .arg(&file)
                .output()?;
            assert!(again.status.success());
            assert_eq!(first.stdout, again.stdout);
            if stage == "patterns" {
                let count = Command::new(compiler)
                    .arg("--count-patterns")
                    .arg(&file)
                    .output()?;
                assert!(count.status.success());
                let view = std::str::from_utf8(&first.stdout)?;
                let footer = view.lines().last().ok_or("missing footer")?;
                assert_eq!(
                    count.stdout,
                    format!("{}\n", footer.strip_prefix("eof ").ok_or("bad footer")?).as_bytes()
                );
            }
        }
        let executable = folder.join(name);
        let rustc = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "constant_values",
                "-C",
                "panic=abort",
                "-C",
                "overflow-checks=yes",
            ])
            .arg(&file)
            .arg("-o")
            .arg(&executable)
            .output()?;
        std::fs::write(file.with_extension("rustc-stderr"), &rustc.stderr)?;
        assert!(
            rustc.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&rustc.stderr)
        );
        let ran = Command::new(executable).output()?;
        assert!(ran.status.success(), "{name}");
        assert!(ran.stdout.is_empty() && ran.stderr.is_empty());
    }
    Ok(())
}
#[test]
fn constant_pattern_rejections_agree_with_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let folder = folder("constant-errors")?;
    for (index, source) in REJECTED.iter().enumerate() {
        let file = folder.join(format!("reject{index}.rs"));
        std::fs::write(&file, source)?;
        let compiler = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&file)
            .output()?;
        assert_eq!(compiler.status.code(), Some(1), "{source}");
        assert!(compiler.stdout.is_empty() && !compiler.stderr.is_empty());
        let rustc = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "constant_reject",
                "--emit=metadata",
            ])
            .arg(&file)
            .arg("-o")
            .arg(folder.join("metadata"))
            .output()?;
        std::fs::write(file.with_extension("rustc-stderr"), &rustc.stderr)?;
        assert_eq!(rustc.status.code(), Some(1), "{source}");
    }
    Ok(())
}

#[test]
fn later_constant_capabilities_remain_explicit_boundaries() -> Result<(), Box<dyn std::error::Error>>
{
    let folder = folder("constant-boundaries")?;
    for (index, (source, boundary)) in [
        (
            "const C:&'static str=\"a\";fn main(){match \"a\"{C=>(),_=>()}}",
            "text value execution requires reference lowering and loan proofs",
        ),
        (
            "const C:u8={let n=7;n};fn main(){match 7u8{C=>(),_=>()}}",
            "general constant evaluation",
        ),
        (
            "const N:u8=7;const C:u8=match 7u8{N=>1,_=>2};fn main(){match 1u8{C=>(),_=>()}}",
            "general constant evaluation",
        ),
        (
            "const N:isize=7;enum E{A=match 7isize{N=>1,_=>2}}fn main(){}",
            "general constant evaluation",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let file = folder.join(format!("boundary{index}.rs"));
        std::fs::write(&file, source)?;
        let compiler = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&file)
            .output()?;
        assert_eq!(compiler.status.code(), Some(1), "{source}");
        assert!(compiler.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&compiler.stderr).contains(boundary),
            "{source}: {}",
            String::from_utf8_lossy(&compiler.stderr)
        );
        let rustc = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "constant_boundary",
                "--emit=metadata",
            ])
            .arg(&file)
            .arg("-o")
            .arg(folder.join("metadata"))
            .output()?;
        std::fs::write(file.with_extension("rustc-stderr"), &rustc.stderr)?;
        assert!(
            rustc.status.success(),
            "{source}: {}",
            String::from_utf8_lossy(&rustc.stderr)
        );
    }
    Ok(())
}

#[test]
fn deferred_ranges_keep_their_source_file() -> Result<(), Box<dyn std::error::Error>> {
    let directory = folder("constant-files")?;
    let main = directory.join("main.rs");
    std::fs::write(
        &main,
        format!(
            "{CHECK}mod values;const N:i8=-2;fn f(x:i8)->u8{{match x{{N=>1,_=>2}}}}fn main(){{check(f(-2)==1&&f(0)==2&&values::f('🦀')==1&&values::f('a')==2);}}"
        ),
    )?;
    std::fs::write(
        directory.join("values.rs"),
        "const N:char='🦀';pub fn f(x:char)->u8{match x{N=>1,_=>2}}",
    )?;
    let ir = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-lowir")
        .arg(&main)
        .output()?;
    assert!(
        ir.status.success(),
        "{}",
        String::from_utf8_lossy(&ir.stderr)
    );
    std::fs::write(directory.join("main.lowir"), &ir.stdout)?;
    let executable = directory.join("rustc");
    let build = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-name",
            "constant_files",
            "-C",
            "panic=abort",
            "-C",
            "overflow-checks=yes",
        ])
        .arg(main)
        .arg("-o")
        .arg(&executable)
        .output()?;
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let ran = Command::new(executable).output()?;
    assert!(ran.status.success() && ran.stdout.is_empty() && ran.stderr.is_empty());
    Ok(())
}
