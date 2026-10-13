use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

const CHECK: &str = "fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}\n";
const LIMITS: &str = "const LO:FLOAT=-1.0/0.0;const HI:FLOAT=1.0/0.0;const NAN:FLOAT=0.0/0.0;\n";
const ACCEPTED: &[(&str, &str)] = &[
    (
        "uninitialized-wildcard",
        "fn main(){let x:FLOAT;match x{_=>()}check(true);}",
    ),
    (
        "zeros",
        "fn f(x:FLOAT)->u8{match x{0.0=>1,-0.0=>2,_=>3}}fn main(){check(f(0.0)==1&&f(-0.0)==1&&f(1.0)==3);}",
    ),
    (
        "capture-sign",
        "fn f(x:FLOAT)->FLOAT{match x{n@0.0=>n,_=>1.0}}fn main(){let n=f(-0.0);let p=f(0.0);check(1.0/n<0.0&&1.0/p>0.0);}",
    ),
    (
        "literal",
        "fn f(x:FLOAT)->u8{match x{-0.125=>1,0.125=>2,_=>3}}fn main(){check(f(-0.125)==1&&f(0.125)==2&&f(1.0)==3);}",
    ),
    (
        "partition",
        "fn f(x:FLOAT)->u8{match x{..0.0=>1,0.0=>2,0.0.. =>3,_=>4}}fn main(){check(f(-1.0/0.0)==1&&f(-1.0)==1&&f(-0.0)==2&&f(0.0)==2&&f(1.0)==3&&f(1.0/0.0)==3&&f(0.0/0.0)==4);}",
    ),
    (
        "full-ordered",
        "fn f(x:FLOAT)->u8{match x{LO..=HI=>1,_=>2}}fn main(){check(f(-1.0/0.0)==1&&f(-0.0)==1&&f(1.0/0.0)==1&&f(0.0/0.0)==2&&f(-(0.0/0.0))==2);}",
    ),
    (
        "open-upper",
        "fn f(x:FLOAT)->u8{match x{LO.. =>1,_=>2}}fn main(){check(f(-1.0/0.0)==1&&f(1.0/0.0)==1&&f(0.0/0.0)==2);}",
    ),
    (
        "open-lower",
        "fn f(x:FLOAT)->u8{match x{..=HI=>1,_=>2}}fn main(){check(f(-1.0/0.0)==1&&f(1.0/0.0)==1&&f(0.0/0.0)==2);}",
    ),
    (
        "infinity-singletons",
        "fn f(x:FLOAT)->u8{match x{LO=>1,HI=>2,_=>3}}fn main(){check(f(-1.0/0.0)==1&&f(1.0/0.0)==2&&f(1.0)==3&&f(0.0/0.0)==3);}",
    ),
    (
        "exclusive-infinity",
        "fn f(x:FLOAT)->u8{match x{..HI=>1,_=>2}}fn main(){check(f(-1.0/0.0)==1&&f(1.0)==1&&f(1.0/0.0)==2&&f(0.0/0.0)==2);}",
    ),
    (
        "inclusive-negative-zero",
        "fn f(x:FLOAT)->u8{match x{-0.0..=0.0=>1,_=>2}}fn main(){check(f(-0.0)==1&&f(0.0)==1&&f(-1.0)==2&&f(0.0/0.0)==2);}",
    ),
    (
        "negative-zero-constant",
        "const Z:FLOAT=-0.0;fn f(x:FLOAT)->u8{match x{Z=>1,_=>2}}fn main(){check(f(-0.0)==1&&f(0.0)==1&&f(0.0/0.0)==2);}",
    ),
    (
        "computed-endpoints",
        "const A:FLOAT=(1u8 as FLOAT)/2.0;const B:FLOAT=A+0.5;fn f(x:FLOAT)->u8{match x{A..B=>1,_=>2}}fn main(){check(f(0.5)==1&&f(0.75)==1&&f(1.0)==2);}",
    ),
    (
        "lazy-constant",
        "const C:FLOAT=if true{1.0}else{0.0/0.0};fn f(x:FLOAT)->u8{match x{C=>1,_=>2}}fn main(){check(f(1.0)==1&&f(0.0/0.0)==2);}",
    ),
    (
        "duplicate-zero-retries",
        "fn f(x:FLOAT)->u8{let mut n=0;match x{0.0|0.0 if {n+=1;false}=>(),_=>()}n}fn main(){check(f(-0.0)==2&&f(0.0)==2&&f(0.0/0.0)==0);}",
    ),
    (
        "range-retries",
        "fn f(x:FLOAT)->u8{let mut n=0;match x{-2.0..=2.0|0.0..=4.0|_ if {n+=1;false}=>(),_=>()}n}fn main(){check(f(1.0)==3&&f(-1.0)==2&&f(0.0/0.0)==1);}",
    ),
    (
        "product",
        "fn f(x:(FLOAT,bool))->u8{match x{(0.0,true)=>1,(..0.0,_)=>2,(_,false)=>3,_=>4}}fn main(){check(f((-0.0,true))==1&&f((-1.0,false))==2&&f((0.0/0.0,false))==3&&f((0.0/0.0,true))==4);}",
    ),
    (
        "array",
        "fn f(x:[FLOAT;2])->u8{match x{[0.0|1.0,n] if n==7.0=>1,_=>2}}fn main(){check(f([-0.0,7.0])==1&&f([1.0,7.0])==1&&f([0.0/0.0,7.0])==2);}",
    ),
    (
        "enum",
        "enum E{A(FLOAT),B}fn f(x:E)->u8{match x{E::A(..0.0)=>1,E::A(0.0..)=>2,E::A(_)=>3,E::B=>4}}fn main(){check(f(E::A(-1.0))==1&&f(E::A(-0.0))==2&&f(E::A(0.0/0.0))==3&&f(E::B)==4);}",
    ),
    (
        "conditional-chain",
        "fn f(x:FLOAT)->u8{if let n@LO..HI=x&&n>0.0{1}else{2}}fn main(){check(f(1.0)==1&&f(-1.0)==2&&f(1.0/0.0)==2&&f(0.0/0.0)==2);}",
    ),
    (
        "while-let",
        "fn f(mut x:FLOAT)->u8{let mut c=0;while let ..0.0=x{c+=1;x=0.0;}c}fn main(){check(f(-1.0)==1&&f(-0.0)==0&&f(0.0/0.0)==0);}",
    ),
    (
        "let-else",
        "fn f(x:FLOAT)->u8{let -0.0..=0.0=x else{return 2;};1}fn main(){check(f(0.0)==1&&f(-0.0)==1&&f(0.0/0.0)==2);}",
    ),
    (
        "conditional-guard",
        "fn f(x:FLOAT)->u8{match x{n if let LO..=HI=n=>1,_=>2}}fn main(){check(f(1.0/0.0)==1&&f(0.0/0.0)==2);}",
    ),
    (
        "configured-nan",
        "fn f(x:FLOAT)->u8{match x{#[cfg(no)] NAN=>0,0.0=>1,_=>2}}fn main(){check(f(-0.0)==1&&f(0.0/0.0)==2);}",
    ),
    (
        "subnormal-neighbors",
        "const T:FLOAT=TINY;const N:FLOAT=-T;fn f(x:FLOAT)->u8{match x{N..0.0=>1,0.0..T=>2,T=>3,_=>4}}fn main(){check(f(-TINY)==1&&f(-0.0)==2&&f(0.0)==2&&f(TINY)==3&&f(2.0*TINY)==4);}",
    ),
    (
        "adjacent-normal",
        "fn f(x:FLOAT)->u8{match x{1.0..NEXT=>1,_=>2}}fn main(){check(f(1.0)==1&&f(NEXT)==2&&f(0.999)==2);}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "endpoint-width",
        "const C:OTHER=0.0;fn main(){match 0.0 as FLOAT{C=>(),_=>()}}",
    ),
    (
        "product-missing-nan",
        "fn main(){match (0.0 as FLOAT,true){(LO..=HI,_)=>(),(_,false)=>()}}",
    ),
    (
        "array-missing-nan",
        "fn main(){match [0.0 as FLOAT]{[LO..=HI]=>()}}",
    ),
    ("missing-zero", "fn main(){match 0.0 as FLOAT{0.0=>()}}"),
    ("missing-nan", "fn main(){match 0.0 as FLOAT{LO..=HI=>()}}"),
    (
        "partition-missing-nan",
        "fn main(){match 0.0 as FLOAT{..0.0=>(),0.0.. =>()}}",
    ),
    ("refutable-let", "fn main(){let LO..=HI=0.0 as FLOAT;}"),
    ("refutable-parameter", "fn f(LO..=HI:FLOAT){}fn main(){}"),
    ("nan-value", "fn main(){match 0.0 as FLOAT{NAN=>(),_=>()}}"),
    (
        "nan-unreachable",
        "fn main(){match 0.0 as FLOAT{_=>(),NAN=>()}}",
    ),
    (
        "nan-low",
        "fn main(){match 0.0 as FLOAT{NAN..=1.0=>(),_=>()}}",
    ),
    (
        "nan-high",
        "fn main(){match 0.0 as FLOAT{0.0..=NAN=>(),_=>()}}",
    ),
    (
        "nan-exclusive",
        "fn main(){match 0.0 as FLOAT{0.0..NAN=>(),_=>()}}",
    ),
    (
        "empty-zero",
        "fn main(){match 0.0 as FLOAT{0.0..0.0=>(),_=>()}}",
    ),
    (
        "empty-signed-zero",
        "fn main(){match 0.0 as FLOAT{-0.0..0.0=>(),_=>()}}",
    ),
    (
        "reversed-inclusive",
        "fn main(){match 0.0 as FLOAT{1.0..=0.0=>(),_=>()}}",
    ),
    (
        "reversed-exclusive",
        "fn main(){match 0.0 as FLOAT{1.0..-1.0=>(),_=>()}}",
    ),
    (
        "empty-lower-open",
        "fn main(){match 0.0 as FLOAT{..LO=>(),_=>()}}",
    ),
    (
        "empty-infinity",
        "fn main(){match 0.0 as FLOAT{HI..HI=>(),_=>()}}",
    ),
    (
        "literal-overflow",
        "fn main(){match 0.0 as FLOAT{1e999=>(),_=>()}}",
    ),
    (
        "uninitialized-input",
        "fn main(){let x:FLOAT;match x{LO..=HI=>(),_=>()}}",
    ),
    (
        "guard-protection",
        "fn main(){let mut x:FLOAT=0.0;match x{LO..=HI if {x=1.0;false}=>(),_=>()}}",
    ),
    (
        "integer-endpoint",
        "const I:u8=0;fn main(){match 0.0 as FLOAT{I..=1.0=>(),_=>()}}",
    ),
];
fn directory() -> Result<PathBuf, std::io::Error> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work/float-vectors")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}
fn frontend(file: &Path, stage: &str) -> Result<Output, std::io::Error> {
    Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(stage)
        .arg(file)
        .output()
}
fn independent(file: &Path, executable: bool) -> Result<(), Box<dyn std::error::Error>> {
    let binary = file.with_extension("rustc");
    let mut command = Command::new("rustc");
    command
        .args([
            "--edition=2024",
            "--crate-name",
            "float_patterns",
            "-C",
            "panic=abort",
            "-C",
            "overflow-checks=yes",
        ])
        .arg(file)
        .arg("-o")
        .arg(&binary);
    if !executable {
        command.arg("--emit=metadata");
    }
    let result = command.output()?;
    std::fs::write(file.with_extension("rustc-stderr"), &result.stderr)?;
    assert_eq!(
        result.status.code(),
        Some(i32::from(!executable)),
        "{}: {}",
        file.display(),
        String::from_utf8_lossy(&result.stderr)
    );
    if executable {
        let result = Command::new(binary).output()?;
        assert!(
            result.status.success() && result.stdout.is_empty() && result.stderr.is_empty(),
            "{}",
            file.display()
        );
    }
    Ok(())
}
fn views(file: &Path, expected: i32) -> Result<(), Box<dyn std::error::Error>> {
    for stage in ["patterns", "lowir"] {
        let output = frontend(file, &format!("--emit-{stage}"))?;
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{}/{stage}: {}",
            file.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        std::fs::write(file.with_extension(stage), &output.stdout)?;
        std::fs::write(
            file.with_extension(format!("{stage}-stderr")),
            &output.stderr,
        )?;
        if expected != 0 {
            assert!(output.stdout.is_empty() && !output.stderr.is_empty());
            let count = frontend(file, &format!("--count-{stage}"))?;
            assert_eq!(count.status.code(), Some(expected));
            assert!(count.stdout.is_empty());
            assert_eq!(count.stderr, output.stderr);
            continue;
        }
        assert!(output.stderr.is_empty());
        let repeated = frontend(file, &format!("--emit-{stage}"))?;
        assert!(repeated.status.success());
        assert_eq!(output.stdout, repeated.stdout);
        let count = frontend(file, &format!("--count-{stage}"))?;
        assert!(count.status.success() && count.stderr.is_empty());
        if stage == "patterns" {
            let text = std::str::from_utf8(&output.stdout)?;
            let footer = text
                .lines()
                .last()
                .and_then(|line| line.strip_prefix("eof "))
                .ok_or("missing footer")?;
            assert_eq!(count.stdout, format!("{footer}\n").as_bytes());
        }
    }
    Ok(())
}
#[test]
fn float_pattern_programs_and_rejections_agree_with_rustc() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = directory()?;
    for (ty, tiny, next) in [
        ("f32", "1.401298464324817e-45", "1.00000011920928955078125"),
        (
            "f64",
            "4.9406564584124654e-324",
            "1.0000000000000002220446049250313080847263336181640625",
        ),
    ] {
        for (cases, expected) in [(ACCEPTED, 0), (REJECTED, 1)] {
            for (name, source) in cases {
                let file = directory.join(format!("{ty}-{expected}-{name}.rs"));
                let source = format!("{CHECK}{LIMITS}{source}")
                    .replace("FLOAT", ty)
                    .replace("OTHER", if ty == "f32" { "f64" } else { "f32" })
                    .replace("TINY", tiny)
                    .replace("NEXT", next);
                std::fs::write(&file, source)?;
                independent(&file, expected == 0)?;
                // Initialization/guard permissions are enforced after the pattern inspection view.
                if ["uninitialized-input", "guard-protection"].contains(name) {
                    let result = frontend(&file, "--emit-lowir")?;
                    assert_eq!(result.status.code(), Some(expected));
                    assert!(result.stdout.is_empty() && !result.stderr.is_empty());
                } else {
                    views(&file, expected)?;
                    if name.ends_with("missing-nan") {
                        let error =
                            std::fs::read_to_string(file.with_extension("patterns-stderr"))?;
                        assert!(error.contains("NaN"), "{name}: {error}");
                    }
                    if *name == "zeros" {
                        let patterns = std::fs::read_to_string(file.with_extension("patterns"))?;
                        assert_eq!(
                            patterns
                                .lines()
                                .filter(|line| line.starts_with("arm ")
                                    && line.contains("useful=false"))
                                .count(),
                            1
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn mixed_zero_retries_follow_the_reference_and_record_the_rustc_discrepancy()
-> Result<(), Box<dyn std::error::Error>> {
    // Reference expr.match.guard.next retries every matching alternative. The pinned
    // rustc instead treats opposite zero encodings as disjoint when routing a failed
    // guard. Keep both expected results explicit; this is not runtime parity evidence.
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work/float-reference-retries")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let version = Command::new("rustc").arg("--version").output()?;
    assert!(String::from_utf8_lossy(&version.stdout).starts_with("rustc 1.98.0 (88d9e12ae "));
    for ty in ["f32", "f64"] {
        for (name, pattern) in [
            ("negative-first", "-0.0|0.0"),
            ("positive-first", "0.0|-0.0"),
            ("constant-negative-first", "NEG|POS"),
            ("constant-positive-first", "POS|NEG"),
        ] {
            let program = format!(
                "{CHECK}const NEG:{ty}=-0.0;const POS:{ty}=0.0;fn f(x:{ty})->u8{{let mut n=0;match x{{{pattern} if {{n+=1;false}}=>(),_=>()}}n}}\n"
            );
            let file = folder.join(format!("reference-{ty}-{name}.rs"));
            std::fs::write(
                &file,
                format!("{program}fn main(){{check(f(-0.0)==2&&f(0.0)==2&&f(0.0/0.0)==0);}}"),
            )?;
            views(&file, 0)?;
            let oracle = folder.join(format!("rustc-{ty}-{name}.rs"));
            std::fs::write(
                &oracle,
                format!("{program}fn main(){{check(f(-0.0)==1&&f(0.0)==1&&f(0.0/0.0)==0);}}"),
            )?;
            independent(&oracle, true)?;
        }
    }
    Ok(())
}
