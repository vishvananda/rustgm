use rustgm::{
    sema::{
        model::Scalar,
        patterns::{
            Interval,
            scalar::{Coverage, Domain},
        },
    },
    types::Primitive,
};
use std::{path::Path, process::Command};

const CHECK: &str = "fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}\n";
const ACCEPTED: &[(&str, &str)] = &[
    (
        "irrefutable-or-arm",
        "fn f(x:u8)->u8{match x{0|_=>7,_=>99}}fn main(){check(f(0)==7&&f(3)==7);}",
    ),
    (
        "never-scrutinee",
        "fn diverge()->!{match loop{} {}}fn main(){check(true);}",
    ),
    (
        "whole-unit-temporary",
        "struct N;fn main(){let whole@N=N;let _=whole;check(true);}",
    ),
    (
        "boolean",
        "fn f(x:bool)->u8{match x{true=>3,false=>7}}fn main(){check(f(true)==3&&f(false)==7);}",
    ),
    (
        "ordered-ranges",
        "fn f(x:u8)->u8{match x{0..=9=>1,5..=20=>2,_=>3}}fn main(){check(f(7)==1&&f(15)==2&&f(30)==3);}",
    ),
    (
        "exclusive-range",
        "fn f(x:u8)->u8{match x{0..5=>1,5.. =>2}}fn main(){check(f(4)==1&&f(5)==2&&f(255)==2);}",
    ),
    (
        "open-ranges",
        "fn f(x:i8)->u8{match x{..=-1=>1,0.. =>2}}fn main(){check(f(-128)==1&&f(127)==2);}",
    ),
    (
        "i128-bounds",
        "fn f(x:i128)->u8{match x{-170141183460469231731687303715884105728..=-1=>1,0.. =>2}}fn main(){check(f(-170141183460469231731687303715884105728)==1&&f(170141183460469231731687303715884105727)==2);}",
    ),
    (
        "u128-bounds",
        "fn f(x:u128)->u8{match x{340282366920938463463374607431768211455=>1,_=>2}}fn main(){check(f(340282366920938463463374607431768211455)==1&&f(0)==2);}",
    ),
    (
        "character-gap",
        "fn f(x:char)->u8{match x{'\u{0}'..='\u{d7ff}'=>1,'\u{e000}'..='\u{10ffff}'=>2}}fn main(){check(f('\u{d7ff}')==1&&f('\u{e000}')==2&&f('\u{10ffff}')==2);}",
    ),
    (
        "character-exclusive",
        "fn f(x:char)->u8{match x{'\u{d7ff}'..'\u{e000}'=>1,_=>2}}fn main(){check(f('\u{d7ff}')==1&&f('\u{e000}')==2);}",
    ),
    (
        "scalar-binding",
        "fn f(x:u8)->u8{match x{n@0..=9=>n+1,n=>n}}fn main(){check(f(5)==6&&f(10)==10);}",
    ),
    (
        "mutable-binding",
        "fn f(x:u8)->u8{match x{mut n=>{n+=1;n}}}fn main(){check(f(5)==6);}",
    ),
    (
        "or-binding",
        "fn f(x:u8)->u8{match x{n@1|n@3=>n+1,_=>0}}fn main(){check(f(1)==2&&f(3)==4&&f(2)==0);}",
    ),
    (
        "nested-at",
        "fn f(x:u8)->u8{match x{a@b@(0|1)=>a+b,_=>0}}fn main(){check(f(1)==2);}",
    ),
    (
        "nested-match",
        "fn f(x:u8,y:bool)->u8{match x{0=>match y{true=>7,false=>9},_=>3}}fn main(){check(f(0,true)==7&&f(0,false)==9&&f(1,true)==3);}",
    ),
    (
        "scrutinee-once",
        "fn main(){let mut n=0;let x=match {n+=1;n}{1=>7,_=>9};check(n==1&&x==7);}",
    ),
    (
        "indexed-scrutinee-once",
        "fn main(){let mut n=0;let xs=[7u8];let x=match xs[{n+=1;0}]{7=>3,_=>5};check(n==1&&x==3);}",
    ),
    (
        "discard-uninitialized",
        "fn main(){let x:u8;match x{_=>{}}check(true);}",
    ),
    (
        "unreachable-literal",
        "fn main(){let x:u8;match x{_|0=>{}}check(true);}",
    ),
    (
        "full-range-uninitialized",
        "fn main(){let x:u8;match x{0..=255=>{}}check(true);}",
    ),
    (
        "return-arm",
        "fn f(x:bool)->u8{let n=match x{true=>return 7,false=>9};n}fn main(){check(f(true)==7&&f(false)==9);}",
    ),
    (
        "loop-transfer",
        "fn f()->u8{let mut n=2;loop{match n{0=>break 7,_=>{n-=1;continue;}}}}fn main(){check(f()==7);}",
    ),
    (
        "diverging-arm",
        "fn f(x:u8)->u8{match x{0=>loop{},_=>7}}fn main(){check(f(1)==7);}",
    ),
    (
        "function-join",
        "fn a(x:u8)->u8{x+1}fn b(x:u8)->u8{x+2}fn f(x:bool)->u8{(match x{true=>a,false=>b})(3)}fn main(){check(f(true)==4&&f(false)==5);}",
    ),
    (
        "configured-arm",
        "fn f(x:bool)->u8{match x{#[cfg(no)] _=>unknown,true=>1,false=>2}}fn main(){check(f(false)==2);}",
    ),
    (
        "macro",
        "macro_rules! choose{($x:expr,$p:pat)=>{match $x{$p=>7,_=>9}}}fn main(){check(choose!(1u8,0|1)==7&&choose!(2u8,0|1)==9);}",
    ),
];
const REJECTED: &[&str] = &[
    "fn main(){let x:u8;match x{_=>(),0=>()}}",
    "fn main(){let x:u8;match x{_=>(),n=>{let _=n;}}}",
    "fn main(){let x:u8;match 0u8{_=>(),_=>{let _=x+1;}}}",
    "struct N;fn main(){let a@b@N=N;let _=a;let _=b;}",
    "fn main(){match true{true=>()}}",
    "fn main(){let x=3u8;match x{0..=2=>(),4.. =>()}}",
    "fn main(){match true{_ if true=>()}}",
    "fn main(){match true{0=>(),_=>()}}",
    "fn main(){match 1u8{256=>(),_=>()}}",
    "fn main(){match 1u8{-1=>(),_=>()}}",
    "fn main(){match 1i8{-129=>(),_=>()}}",
    "fn main(){match 1u8{2..1=>(),_=>()}}",
    "fn main(){match 1u8{0..0=>(),_=>()}}",
    "fn main(){match true{false..=true=>()}}",
    "fn main(){match true{_ if 1=>(),_=>()}}",
    "fn main(){let _=match true{true=>1u8,false=>false};}",
    "fn main(){match 0u8{x@0|1=>{let _=x;},_=>()}}",
    "fn main(){let x:u8;match x{0|_=>()}}",
    "fn main(){let x:u8;match x{y=>{let _=y;}}}",
    "fn main(){let x=1u8;let 0=x;}",
];

#[test]
fn scalar_domains_preserve_endpoints_and_the_unicode_gap() -> Result<(), &'static str> {
    for primitive in Primitive::ALL {
        let Some(domain) = Domain::new(primitive) else {
            continue;
        };
        for rank in [0, domain.ordered().upper / 2, domain.ordered().upper] {
            assert_eq!(
                domain.rank(domain.value(rank)?, false)?,
                rank,
                "{primitive:?}"
            );
        }
    }
    let signed = Domain::new(Primitive::I128).ok_or("missing signed domain")?;
    assert_eq!(signed.witness(0)?, i128::MIN.to_string());
    assert_eq!(signed.witness(signed.full().upper)?, i128::MAX.to_string());
    let chars = Domain::new(Primitive::Char).ok_or("missing character domain")?;
    assert_eq!(
        chars.rank(Scalar::Character('\u{d7ff}'), false)? + 1,
        chars.rank(Scalar::Character('\u{e000}'), false)?
    );
    assert_eq!(chars.value(0xd800)?, Scalar::Character('\u{e000}'));
    Ok(())
}

#[test]
fn interval_index_matches_every_subset_of_a_small_domain() -> Result<(), &'static str> {
    let domain = Domain::new(Primitive::U8).ok_or("missing byte domain")?;
    for mask in 0_u32..256 {
        let mut coverage = Coverage::default();
        for bit in 0..8 {
            if mask & (1 << bit) != 0 {
                coverage.insert(Interval {
                    lower: bit,
                    upper: bit,
                })?;
            }
        }
        for lower in 0..8 {
            for upper in lower..8 {
                let useful = (lower..=upper).any(|bit| mask & (1 << bit) == 0);
                assert_eq!(coverage.useful(Interval { lower, upper })?, useful);
            }
        }
        let first = (0..8).find(|bit| mask & (1 << bit) == 0).unwrap_or(8);
        assert_eq!(coverage.missing(domain).map(|i| i.lower), Some(first));
        coverage.insert(domain.full())?;
        assert_eq!(coverage.missing(domain), None);
    }
    let mut coverage = Coverage::default();
    for _ in 0..Coverage::LIMIT {
        coverage.charge()?;
    }
    assert_eq!(
        coverage.charge(),
        Err("pattern coverage work budget exceeded")
    );
    Ok(())
}

#[test]
fn scalar_match_programs_agree_with_rustc_and_publish_stable_views()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work/scalar-vectors")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (name, source) in ACCEPTED {
        let file = folder.join(format!("{name}.rs"));
        std::fs::write(&file, format!("{CHECK}{source}"))?;
        let compiler = env!("CARGO_BIN_EXE_rustgm");
        let ir = Command::new(compiler)
            .arg("--emit-lowir")
            .arg(&file)
            .output()?;
        assert!(
            ir.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&ir.stderr)
        );
        let warnings: &[u32] = if *name == "irrefutable-or-arm" {
            &[102]
        } else {
            &[]
        };
        crate::warning_contracts::unreachable(&ir.stderr, warnings)?;
        std::fs::write(file.with_extension("lowir"), &ir.stdout)?;
        for stage in ["patterns", "lowir"] {
            let first = Command::new(compiler)
                .arg(format!("--emit-{stage}"))
                .arg(&file)
                .output()?;
            let again = Command::new(compiler)
                .arg(format!("--emit-{stage}"))
                .arg(&file)
                .output()?;
            assert!(first.status.success() && again.status.success());
            crate::warning_contracts::unreachable(&first.stderr, warnings)?;
            crate::warning_contracts::unreachable(&again.stderr, warnings)?;
            assert_eq!(first.stdout, again.stdout);
            let count = Command::new(compiler)
                .arg(format!("--count-{stage}"))
                .arg(&file)
                .output()?;
            assert!(count.status.success());
            crate::warning_contracts::unreachable(&count.stderr, warnings)?;
            if stage == "patterns" {
                let view = std::str::from_utf8(&first.stdout)?;
                let footer = view
                    .lines()
                    .last()
                    .and_then(|s| s.strip_prefix("eof "))
                    .ok_or("missing pattern footer")?;
                assert_eq!(count.stdout, format!("{footer}\n").as_bytes());
            }
        }
        let executable = folder.join(name);
        let rustc = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "match_values",
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
        assert!(
            ran.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&ran.stderr)
        );
        assert!(ran.stdout.is_empty() && ran.stderr.is_empty());
    }
    Ok(())
}

#[test]
fn scalar_match_rejections_agree_with_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work/scalar-errors")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
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
                "match_reject",
                "--emit=metadata",
            ])
            .arg(&file)
            .arg("-o")
            .arg(folder.join("metadata"))
            .output()?;
        std::fs::write(file.with_extension("rustc-stderr"), &rustc.stderr)?;
        assert_eq!(
            rustc.status.code(),
            Some(1),
            "{source}: {}",
            String::from_utf8_lossy(&rustc.stderr)
        );
    }
    Ok(())
}
