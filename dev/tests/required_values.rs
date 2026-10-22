//! Complete source constants exercise readiness, checked calls and native results.
use crate::native_references as native;
use crate::resolve as source;
use std::{path::Path, process::Command};

const POSITIVES: &[(&str, &str, bool)] = &[
    (
        "scalar-call",
        "const fn f(n:u32)->u32{n+3}const C:u32=f(4);fn main(){check(C==7);}",
        false,
    ),
    (
        "combined-root",
        "const fn f(n:u32)->u32{n+3}const C:u32=1+f(4)*2;fn main(){check(C==15);}",
        false,
    ),
    (
        "recursive",
        "const fn f(n:u32)->u32{if n==0{1}else{n*f(n-1)}}const C:u32=f(6);fn main(){check(C==720);}",
        false,
    ),
    (
        "mutation-loop",
        "const fn f()->u32{let mut i=0;let mut n=0;while i<10{n+=i;i+=1;}n}const C:u32=f();fn main(){check(C==45);}",
        false,
    ),
    (
        "early-array",
        "const fn size(n:usize)->usize{n+2}fn main(){let a:[u8;size(2)]=[7;4];check(a[3]==7);}",
        false,
    ),
    (
        "const-argument",
        "const fn size(n:usize)->usize{n+2}struct A<const N:usize>([u8;N]);fn main(){let a:A<{size(2)}>=A([7;4]);check(a.0[3]==7);}",
        false,
    ),
    (
        "reference-mutation",
        "const fn bump(n:&mut u32){*n+=2;}const fn run()->u32{let mut n=6;bump(&mut n);n}const C:u32=run();fn main(){check(C==8);}",
        false,
    ),
    (
        "generic-const",
        "const fn f<const N:usize>(n:usize)->usize{N+n}const C:usize=f::<5>(3);fn main(){check(C==8);}",
        false,
    ),
    (
        "generic-type",
        "const fn f<T:Copy>(n:T)->T{n}const C:u32=f::<u32>(7);fn main(){check(C==7);}",
        false,
    ),
    (
        "forward-value-header",
        "const fn f()->usize{N+1}type A=[u8;f()];const N:usize=2;fn main(){let a:A=[7;3];check(a[2]==7);}",
        false,
    ),
    (
        "reordered-value-header",
        "const N:usize=2;const fn f()->usize{N+1}type A=[u8;f()];fn main(){let a:A=[7;3];check(a[2]==7);}",
        false,
    ),
    (
        "forward-function-header",
        "const fn f()->usize{g()}type A=[u8;f()];const fn g()->usize{3}fn main(){let a:A=[7;3];check(a[2]==7);}",
        false,
    ),
    (
        "reordered-function-header",
        "const fn g()->usize{3}const fn f()->usize{g()}type A=[u8;f()];fn main(){let a:A=[7;3];check(a[2]==7);}",
        false,
    ),
    (
        "forward-constructor-header",
        "const fn f()->usize{let _=S(7);3}type A=[u8;f()];struct S(u32);fn main(){let a:A=[7;3];check(a[2]==7);}",
        false,
    ),
    (
        "reordered-constructor-header",
        "struct S(u32);const fn f()->usize{let _=S(7);3}type A=[u8;f()];fn main(){let a:A=[7;3];check(a[2]==7);}",
        false,
    ),
    (
        "forward-static-header",
        "const fn f()->usize{let _=&S;3}type A=[u8;f()];static S:u32=7;fn main(){let a:A=[7;3];check(a[2]==7);}",
        false,
    ),
    (
        "reordered-static-header",
        "static S:u32=7;const fn f()->usize{let _=&S;3}type A=[u8;f()];fn main(){let a:A=[7;3];check(a[2]==7);}",
        false,
    ),
    (
        "promoted-result",
        "const fn f()->&'static u32{&7}const C:&u32=f();fn main(){check(*C==7);}",
        false,
    ),
];

#[test]
#[ignore = "requires the configured CPPGM native backend"]
fn checked_source_constants_match_independent_native_values() {
    let mut errors = Vec::new();
    for case in POSITIVES {
        if let Err(error) = native::check_cases("pa25-constant-sources", &[*case]) {
            errors.push(format!("{}: {error}", case.0));
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

const REJECTIONS: &[(&str, &str)] = &[
    (
        "unused-nonconst-body",
        "fn g()->u32{7}const fn f()->u32{g()}fn main(){}",
    ),
    (
        "skipped-nonconst-call",
        "fn g()->usize{7}type A=[u8;{if true{7}else{g()}}];fn main(){}",
    ),
    (
        "function-pointer-call",
        "const fn f(n:u32)->u32{n}const C:u32=(f as fn(u32)->u32)(3);fn main(){let _=C;}",
    ),
    (
        "overflow",
        "const fn f(n:u8)->u8{n+1}const C:u8=f(255);fn main(){let _=C;}",
    ),
    (
        "division-by-zero",
        "const fn f(n:u8)->u8{1/n}const C:u8=f(0);fn main(){let _=C;}",
    ),
    (
        "escaping-local",
        "const fn f()->&'static u32{let x=7;&x}fn main(){}",
    ),
    (
        "value-cycle",
        "const fn f()->usize{N}const N:usize=f();type A=[u8;N];fn main(){}",
    ),
];

#[test]
fn required_failures_match_independently_declared_object_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root.join(format!("obj/pa25-constant-outcomes/{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for &(name, text) in REJECTIONS {
        let input = folder.join(format!("{name}.rs"));
        std::fs::write(&input, text)?;
        let independent = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=constant_outcome",
                "--emit=obj",
                "-o",
            ])
            .arg(folder.join(format!("{name}.o")))
            .arg(&input)
            .output()?;
        std::fs::write(
            folder.join(format!("{name}.rustc.stderr")),
            &independent.stderr,
        )?;
        assert_eq!(
            independent.status.code(),
            Some(1),
            "{name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        let candidate = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&input)
            .output()?;
        std::fs::write(
            folder.join(format!("{name}.candidate.stderr")),
            &candidate.stderr,
        )?;
        assert_eq!(
            candidate.status.code(),
            Some(1),
            "{name}: {}",
            String::from_utf8_lossy(&candidate.stderr)
        );
    }
    Ok(())
}

#[test]
fn early_late_and_repeated_demands_construct_each_function_cfg_once()
-> Result<(), Box<dyn std::error::Error>> {
    for text in [
        "const fn f(n:usize)->usize{n+1}const A:usize=f(1);const B:usize=f(2);fn main(){let _=(A,B);}",
        "const fn f(n:usize)->usize{n+1}type A=[u8;f(1)];const B:usize=f(2);fn main(){let _:A=[0;2];let _=B;}",
        "const fn f<const N:usize>()->usize{N}const A:usize=f::<3>();const B:usize=f::<4>();fn main(){let _=(A,B);}",
    ] {
        let mut report = source::resolve(text, true)?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
        assert_eq!(facts.body_work.cfg_functions.get(), 2, "{text}");
        assert!(facts.constant_work.expressions >= 2, "{text}");
        assert!(facts.constant_work.frames_started >= 4, "{text}");
        assert!(facts.constant_work.steps > 0, "{text}");
        assert!(facts.constant_work.code_bytes > 0, "{text}");
        assert!(facts.constant_work.peak_frames >= 2, "{text}");
    }
    Ok(())
}

#[test]
fn repeated_roots_prepare_one_append_only_value_index() -> Result<(), Box<dyn std::error::Error>> {
    for width in [8, 32, 128] {
        let mut text = String::from("const fn f(n:usize)->usize{n+1}");
        for index in 0..width {
            use std::fmt::Write;
            write!(text, "const C{index}:usize=f({index});")?;
        }
        text.push_str("fn main(){let _=(");
        for index in 0..width {
            use std::fmt::Write;
            write!(text, "C{index},")?;
        }
        text.push_str(");}");
        let mut report = source::resolve(&text, true)?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
        assert_eq!(facts.constant_work.expressions, width);
        let index_size =
            facts.definitions.len() + facts.initializers.len() + report.package.units.len();
        assert!(
            facts.constant_work.value_preparation <= index_size * 2,
            "{width} roots: {} preparation visits for {index_size} index entries",
            facts.constant_work.value_preparation
        );
        assert_eq!(facts.body_work.cfg_functions.get(), 2);
    }
    Ok(())
}
