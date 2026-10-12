#[path = "support/body_machine.rs"]
mod body_machine;
use body_machine::{Data, Machine};
use rustgm::{
    body::{
        self,
        ir::{Owner, Program},
    },
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::{bodies, declarations, model::Declarations},
    support::source::SourceFile,
};
use std::{path::Path, process::Command};
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn analyze(source: &str) -> Result<(Report, Declarations, Program), String> {
    let report = resolve::expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )
    .and_then(resolve::analyze_expanded)
    .map_err(|e| e.to_string())?;
    let facts = declarations::analyze(&report)
        .and_then(|f| bodies::analyze(&report, f))
        .map_err(|e| e.to_string())?;
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    Ok((report, facts, program))
}
const CASES: &[(&str, &str, u128)] = &[
    (
        "wildcard-uninitialized-place",
        "fn entry()->u64{let x:u64;let _=(x);let y:();let _=y;0}",
        0,
    ),
    ("literal", "fn entry()->u64{42}", 42),
    (
        "binary-snapshot",
        "fn entry()->u64{let mut x=1;x+{x=2;x}}",
        3,
    ),
    (
        "argument-order",
        "fn sum(a:u64,b:u64)->u64{a*10+b} fn entry()->u64{let mut x=1;sum(x,{x=9;x})}",
        19,
    ),
    (
        "callee-before-arguments",
        "fn a(x:u64)->u64{x+1} fn b(x:u64)->u64{x+20} fn entry()->u64{let mut f:fn(u64)->u64=a;f({f=b;1})}",
        2,
    ),
    (
        "compound-rhs-first",
        "fn entry()->u64{let mut x=1;x+={x=10;3};x}",
        13,
    ),
    (
        "assignment-rhs-first",
        "fn entry()->u64{let mut x=1;x={x=5;9};x}",
        9,
    ),
    (
        "and-skips",
        "fn entry()->u64{let mut x=3;false&&{x=9;true};x}",
        3,
    ),
    (
        "or-skips",
        "fn entry()->u64{let mut x=3;true||{x=9;false};x}",
        3,
    ),
    (
        "nested-side-effects",
        "fn entry()->u64{let mut x=0;(true&&{x+=1;false})||{x+=2;true};x}",
        3,
    ),
    (
        "nested-short-circuit",
        "fn entry()->u64{let a=false;let b=true;((a&&(b||a))||(b&&(a||b))) as u64}",
        1,
    ),
    (
        "nested-if",
        "fn entry()->u64{if true{if false{1}else{2}}else{3}}",
        2,
    ),
    (
        "else-if",
        "fn entry()->u64{if false{1}else if true{2}else{3}}",
        2,
    ),
    ("while", "fn entry()->u64{let mut x=0;while x<5{x+=1;}x}", 5),
    (
        "loop-continue",
        "fn entry()->u64{let mut x=0;loop{x+=1;if x<4{continue;}break x;}}",
        4,
    ),
    (
        "while-continue",
        "fn entry()->u64{let mut n=0;let mut sum=0;while n<5{n+=1;if n==3{continue;}sum+=n;}sum}",
        12,
    ),
    (
        "outer-break",
        "fn entry()->u64{'a:loop{loop{break 'a 9;}}}",
        9,
    ),
    (
        "outer-continue",
        "fn entry()->u64{let mut x=0;'a:loop{x+=1;loop{if x<3{continue 'a;}break 'a x;}}}",
        3,
    ),
    ("labeled-block", "fn entry()->u64{'a:{break 'a 7;}}", 7),
    (
        "block-fallthrough",
        "fn entry()->u64{'a:{if false{break 'a 7;}8}}",
        8,
    ),
    ("early-return", "fn entry()->u64{if true{return 12;}99}", 12),
    (
        "condition-effects",
        "fn entry()->u64{let mut x=0;while {x+=1;x<4}{}x}",
        4,
    ),
    (
        "condition-short-circuit",
        "fn entry()->u64{let mut x=0;while (x<4)&&(x!=2){x+=1;}x}",
        2,
    ),
    (
        "labeled-condition-break",
        "fn entry()->u64{let mut x=0;'a:while {x+=1;if x==3{break 'a;}true}{}x}",
        3,
    ),
    (
        "branch-initialization",
        "fn entry()->u64{let x;if true{x=5;}else{x=6;}x}",
        5,
    ),
    ("unit-call", "fn unit(){} fn entry()->u64{unit();42}", 42),
    (
        "recursive-call",
        "fn fact(n:u64)->u64{if n==0{1}else{n*fact(n-1)}} fn entry()->u64{fact(5)}",
        120,
    ),
    (
        "function-pointer-join",
        "fn a()->u64{7} fn b()->u64{8} fn entry()->u64{let f:fn()->u64=loop{if true{break a;}break b;};f()}",
        7,
    ),
    ("grouped-binding", "fn entry()->u64{let (x)=4;let _=9;x}", 4),
    (
        "shadowing",
        "fn entry()->u64{let x=1;{let x=2;let _=x;}x}",
        1,
    ),
    (
        "signed-shift",
        "fn entry()->u64{let x:i8=-8;((x>>1)==-4) as u64}",
        1,
    ),
    (
        "const-initializers",
        "const A:u64=B+1;const B:u64=8;fn entry()->u64{A}",
        9,
    ),
    (
        "static-initializer",
        "static A:u64=17;fn entry()->u64{A}",
        17,
    ),
];
#[test]
fn region_conversions_remain_explicit_obligations() -> Result<(), String> {
    let (_, facts, program) = analyze("fn f<'a,'b>(x:&'a u8,y:&'b u8)->&'a u8{y}")?;
    assert_eq!(program.region_equalities.len(), 1);
    assert_eq!(facts.region_equalities.len(), 1);
    let body = &program.bodies[0];
    let conversion = body
        .values
        .iter()
        .find(|value| matches!(value.kind, body::ir::Rvalue::Constrain(_)))
        .ok_or("missing constrained conversion")?;
    assert_eq!(conversion.ty, body.result);
    Ok(())
}
#[test]
fn deep_bodies_are_iterative_and_views_are_deterministic() -> Result<(), String> {
    let source = format!("fn entry()->u64{{0{}}}", "+1".repeat(10_000));
    let (report, facts, program) = analyze(&source)?;
    let Owner::Function { definition, .. } = program.bodies[0].owner else {
        return Err("expected function".into());
    };
    let actual = Machine {
        program: &program,
        types: &facts.types,
        fuel: 100_000,
    }
    .run(definition, &[], 0)?;
    assert_eq!(actual, Data::Bits(10_000));
    let mut first = Vec::new();
    let mut second = Vec::new();
    let mut count = Vec::new();
    body::body_view::write(&program, &facts.types, &mut first).map_err(|e| e.to_string())?;
    let rebuilt = body::build(&report, &facts).map_err(|e| e.to_string())?;
    body::body_view::write(&rebuilt, &facts.types, &mut second).map_err(|e| e.to_string())?;
    body::body_view::count(&program, &mut count).map_err(|e| e.to_string())?;
    assert_eq!(first, second);
    assert!(first.ends_with(&[b"eof ", count.as_slice()].concat()));
    Ok(())
}
#[test]
fn cfg_execution_agrees_with_declared_results_and_rustc() -> Result<(), String> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa10-work/runtime")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    for (index, (name, source, expected)) in CASES.iter().enumerate() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa10/tests/spec/execution/{name}.rs"));
        assert_eq!(
            std::fs::read_to_string(fixture)
                .map_err(|e| e.to_string())?
                .trim_end(),
            *source
        );
        let (report, facts, program) =
            analyze(source).map_err(|e| format!("case {index}: {source}: {e}"))?;
        let entry = program
            .bodies
            .iter()
            .find_map(|body| match body.owner {
                Owner::Function { definition, .. }
                    if report
                        .package
                        .names
                        .get(report.resolution.definitions[definition.0 as usize].name)
                        == "entry" =>
                {
                    Some(definition)
                }
                _ => None,
            })
            .ok_or("no entry function")?;
        let actual = Machine {
            program: &program,
            types: &facts.types,
            fuel: 100_000,
        }
        .run(entry, &[], 0)
        .map_err(|e| format!("case {index}: {source}: {e}"))?;
        assert_eq!(actual, Data::Bits(*expected), "case {index}: {source}");
        let path = directory.join(format!("case-{index}.rs"));
        let binary = directory.join(format!("case-{index}"));
        std::fs::write(
            &path,
            format!("{source}\nfn main(){{print!(\"{{}}\",entry());}}\n"),
        )
        .map_err(|e| e.to_string())?;
        let compiled = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "body_check",
                "-C",
                "panic=abort",
                "-C",
                "overflow-checks=on",
            ])
            .arg(&path)
            .arg("-o")
            .arg(&binary)
            .output()
            .map_err(|e| e.to_string())?;
        if !compiled.status.success() {
            return Err(format!(
                "rustc case {index}: {}",
                String::from_utf8_lossy(&compiled.stderr)
            ));
        }
        let ran = Command::new(binary).output().map_err(|e| e.to_string())?;
        assert!(ran.status.success(), "rustc runtime case {index}");
        assert!(ran.stderr.is_empty(), "rustc stderr case {index}");
        assert_eq!(
            ran.stdout,
            expected.to_string().as_bytes(),
            "rustc result case {index}"
        );
    }
    Ok(())
}
