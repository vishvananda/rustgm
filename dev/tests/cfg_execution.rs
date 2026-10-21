//! Execution tests consume ordinary typed CFGs and independent const-value checks.
#[path = "helpers/qualify.rs"]
mod rustc;
use rustgm::{
    body,
    const_eval::{Limits, Machine},
    expand::config::Config,
    resolve::{self, load},
    sema::{bodies, declarations, model::Scalar, values::Evaluator},
    support::source::SourceFile,
};
use std::path::Path;
struct Memory;
impl load::Sources for Memory {
    fn read(&mut self, _: &Path) -> Result<Option<load::InputFile>, String> {
        Ok(None)
    }
}
fn execute(source: &str, limits: Limits) -> Result<(u128, rustgm::const_eval::Work), String> {
    let input = load::InputFile {
        identity: "execution.rs".into(),
        path: "execution.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    };
    let report = resolve::expansion::analyze(input, &mut Memory, &Config::default())
        .and_then(resolve::analyze_expanded)
        .map_err(|e| e.to_string())?;
    let facts = declarations::analyze(&report).map_err(|e| e.to_string())?;
    let mut facts = bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let definition = facts
        .functions
        .last()
        .ok_or("missing test function")?
        .definition;
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let mut machine = Machine::new(Evaluator::new(&report, &mut facts), &program.bodies, limits)
        .map_err(str::to_string)?;
    let value = machine
        .definition(definition, &[])
        .map_err(|e| e.message.to_string())?
        .as_scalar()
        .map_err(|e| e.to_string())?;
    if let Scalar::Integer(value) = value {
        Ok((value, machine.work()))
    } else {
        Err("test result is not an integer".into())
    }
}
const CASES: &[(&str, u128, &str)] = &[
    ("scalar", 9, "const fn run()->u32{4+5}"),
    ("locals", 15, "const fn run()->u32{let mut x=3;x+=12;x}"),
    ("branch", 8, "const fn run()->u32{if true{8}else{1/0}}"),
    (
        "loop",
        45,
        "const fn run()->u32{let mut i=0;let mut sum=0;while i<10{sum+=i;i+=1;}sum}",
    ),
    ("return", 4, "const fn run()->u32{let x=4;return x;}"),
    (
        "call",
        7,
        "const fn f(n:u32)->u32{n+3}const fn run()->u32{f(4)}",
    ),
    (
        "recursive",
        720,
        "const fn f(n:u32)->u32{if n==0{1}else{n*f(n-1)}}const fn run()->u32{f(6)}",
    ),
    (
        "tuple",
        9,
        "const fn run()->u32{let mut p=(3,5);p.0=4;p.0+p.1}",
    ),
    (
        "array",
        8,
        "const fn run()->u32{let mut p=[0;4];let mut i=0;while i<4{p[i]=i as u32+5;i+=1;}p[3]}",
    ),
    (
        "shared-borrow",
        6,
        "const fn run()->u32{let x=6;let p=&x;*p}",
    ),
    (
        "mutable-borrow",
        8,
        "const fn run()->u32{let mut x=6;let p=&mut x;*p+=2;x}",
    ),
    (
        "reference-call",
        8,
        "const fn f(x:&mut u32){*x+=2;}const fn run()->u32{let mut x=6;f(&mut x);x}",
    ),
    (
        "field-reference",
        10,
        "const fn run()->u32{let mut x=(1,4);let p=&mut x.1;*p=9;x.0+x.1}",
    ),
    (
        "match",
        7,
        "const fn run()->u32{let x=2;match x{1=>9,2=>7,_=>0}}",
    ),
    (
        "nested-calls",
        28,
        "const fn add(a:u32,b:u32)->u32{a+b}const fn run()->u32{add(add(3,5),add(8,12))}",
    ),
    (
        "early-return",
        9,
        "const fn f(n:u32)->u32{if n>3{return n+5;}0}const fn run()->u32{f(4)}",
    ),
    (
        "break-value",
        12,
        "const fn run()->u32{let mut n=0;loop{n+=3;if n>=12{break n;}}}",
    ),
    (
        "continue",
        12,
        "const fn run()->u32{let mut i=0;let mut n=0;while i<6{i+=1;if i%2==1{continue;}n+=i;}n}",
    ),
    (
        "short-circuit",
        7,
        "const fn run()->u32{if false&&(1/0==1){0}else if true||(1/0==1){7}else{0}}",
    ),
    (
        "shared-alias",
        8,
        "const fn run()->u32{let mut n=6;let p=&mut n;*p+=1;*p+=1;n}",
    ),
    (
        "nested-product",
        14,
        "const fn run()->u32{let mut p=((2,3),[4,5]);p.0.1=5;p.1[0]=7;p.0.0+p.0.1+p.1[0]}",
    ),
    (
        "enum",
        9,
        "enum E{A(u32),B}const fn run()->u32{let e=E::A(9);match e{E::A(n)=>n,E::B=>0}}",
    ),
    (
        "temporary-borrow",
        12,
        "const fn run()->u32{let p=&(7,5);p.0+p.1}",
    ),
];
#[test]
fn typed_cfg_values_match_independent_const_assertions() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, expected, source) in CASES {
        rustc::qualify(
            &format!("const-execution-{name}"),
            &format!(
                "#![allow(unconditional_panic)]{source} const _: [(); {expected}] = [(); run() as usize];"
            ),
            true,
        )?;
        let (actual, _) = execute(source, Limits::default()).map_err(|e| format!("{name}: {e}"))?;
        assert_eq!(actual, expected, "{name}");
    }
    Ok(())
}
#[test]
fn execution_limits_are_explicit_and_host_stack_independent() {
    let result = execute(
        "const fn run()->u32{loop{}}",
        Limits {
            steps: 100,
            ..Limits::default()
        },
    );
    assert_eq!(
        result.err().as_deref(),
        Some("constant execution step limit exceeded")
    );
    let result = execute(
        "const fn run()->u32{run()}",
        Limits {
            frames: 8,
            ..Limits::default()
        },
    );
    assert_eq!(
        result.err().as_deref(),
        Some("constant execution frame limit exceeded")
    );
}
