use crate::semantic;
use rustgm::{body, borrow};

fn check(source: &str) -> Result<usize, String> {
    let (report, mut facts) = semantic::analyze(source)?;
    let mut program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    body::promote::analyze(&report, &mut facts, &mut program).map_err(|e| e.to_string())?;
    let contracts = body::regions::collect(&facts)?;
    let regions = borrow::regions::check_bodies(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )
    .map_err(|e| e.to_string())?;
    let proof = borrow::check(
        &mut borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            copies: &mut facts.copies,
        },
        &regions,
        &program,
    )
    .map_err(|e| e.to_string())?;
    Ok(proof.loans.iter().map(|p| p.loans).sum())
}
fn cases(group: &str, cases: &[(&str, bool)]) -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/loans-{}-{group}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    let mut failures = Vec::new();
    for (i, &(source, accepted)) in cases.iter().enumerate() {
        let input = directory.join(format!("case-{i}.rs"));
        std::fs::write(&input, source)?;
        let result = std::process::Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-type=lib",
                "--crate-name=probe",
                "--cap-lints=allow",
                "--emit=metadata",
                "-o",
            ])
            .arg(directory.join("probe.rmeta"))
            .arg(input)
            .output()?;
        std::fs::write(directory.join(format!("case-{i}.stderr")), &result.stderr)?;
        assert_eq!(
            result.status.code(),
            Some(i32::from(!accepted)),
            "rustc {source}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let actual = check(source);
        if actual.is_ok() != accepted {
            failures.push(format!("{i}: {source}: {actual:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn last_uses_aliases_and_reborrow_permissions() -> Result<(), Box<dyn std::error::Error>> {
    cases(
        "uses",
        &[
            ("struct S<'a>(&'a u8);fn f<'a>(p:&'a u8)->S<'a>{S(p)}", true),
            (
                "enum E<'a>{A(&'a mut u8),B}fn f<'a>(p:&'a mut u8)->E<'a>{E::A(p)}",
                true,
            ),
            ("fn f(){let mut x=0;let p=&x;let y=*p;x=1;}", true),
            ("fn f(){let mut x=0;let p=&x;x=1;let y=*p;}", false),
            ("fn f(){let mut x=0;let p=&mut x;*p=2;x=1;}", true),
            ("fn f(){let mut x=0;let p=&mut x;x=1;*p=2;}", false),
            ("fn f(){let mut x=0;let p=&mut x;let y=x;*p=2;}", false),
            ("fn f(){let mut x=0;let p=&x;let q=&x;let y=(*p,*q);}", true),
            (
                "fn f(){let mut x=0;let p=&x;let q=&mut x;let y=*p;*q=2;}",
                false,
            ),
            (
                "fn f(){let mut x=0;let p=&mut x;let q=&x;*p=2;let y=*q;}",
                false,
            ),
            ("fn f(p:&mut u8){let q=&mut *p;*q=1;*p=2;}", true),
            ("fn f(p:&mut u8){let q=&mut *p;*p=2;*q=1;}", false),
            ("fn f(p:&mut u8){let q=&*p;let y=*p;let z=*q;}", true),
            ("fn f(p:&mut u8){let q=&*p;*p=2;let z=*q;}", false),
            ("fn f(p:&mut u8){let q=&mut *p;let z=*p;*q=1;}", false),
            ("fn f<'a>(p:&'a u8)->&'a u8{let q=&*p;q}", true),
            ("fn f<'a>(p:&'a mut u8)->&'a mut u8{let q=&mut *p;q}", true),
            (
                "fn f<'a>(p:&'a mut u8){let q:&'a mut u8=&mut *p;*q=1;*p=2;}",
                false,
            ),
            (
                "fn f(){let mut x=0;let mut y=0;let mut p=&mut x;*p=1;p=&mut y;x=2;*p=3;}",
                true,
            ),
            (
                "fn f(){let mut x=0;let mut y=0;let mut p=&mut x;let q=&mut *p;p=&mut y;*q=1;*p=2;}",
                true,
            ),
            (
                "fn f(){let x=0;let mut p=&x;let q=&*p;p=&1;let y=*q;}",
                true,
            ),
            ("struct N;fn f(){let x=N;let p=&x;let y=x;let q=p;}", false),
            ("struct N;fn f(){let x=N;let p=&x;let q=p;let y=x;}", true),
        ],
    )
}

#[test]
fn cli_checks_loans_and_keeps_reference_lowering_explicit() -> Result<(), Box<dyn std::error::Error>>
{
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    for (source, diagnostic) in [
        (
            "fn f(){let mut x=0;let p=&x;let y=*p;x=1;}fn main(){}",
            None,
        ),
        (
            "fn f(){let mut x=0;let p=&x;x=1;let y=*p;}fn main(){}",
            Some("access conflicts with a live reference loan"),
        ),
        (
            "fn f(){let p;{let x=0;p=&x;}let y=*p;}fn main(){}",
            Some("access conflicts with a live reference loan"),
        ),
    ] {
        for mode in ["--emit-ownership", "--count-ownership", "--emit-lowir"] {
            let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            child
                .stdin
                .take()
                .ok_or("missing compiler input")?
                .write_all(source.as_bytes())?;
            let output = child.wait_with_output()?;
            let expected = diagnostic;
            assert_eq!(
                output.status.code(),
                Some(i32::from(expected.is_some())),
                "{source}: {mode}"
            );
            if let Some(message) = expected {
                assert!(output.stdout.is_empty());
                assert!(
                    String::from_utf8_lossy(&output.stderr).contains(message),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            } else {
                assert!(output.stderr.is_empty());
                assert!(!output.stdout.is_empty());
            }
        }
    }
    Ok(())
}

#[test]
fn two_phase_borrows_activate_at_the_consuming_call() -> Result<(), Box<dyn std::error::Error>> {
    cases(
        "activation",
        &[
            (
                "fn take(_: &mut u8,_:u8){}fn f(p:&mut u8){take(p,*p);}",
                true,
            ),
            (
                "fn take(_: &mut u8,_:u8){}fn f(p:&mut u8){take(&mut *p,*p);}",
                false,
            ),
            (
                "fn take(_: &mut u8,_:u8){}fn f(p:&mut u8){take({p},*p);}",
                false,
            ),
            (
                "fn take(_: &mut u8,_:&u8){}fn f(p:&mut u8){take(p,&*p);}",
                false,
            ),
            (
                "fn take(_: &mut u8,_:u8){}fn f(p:&mut u8){take(p,{let q=&*p;*q});}",
                true,
            ),
            (
                "fn take(_: &mut u8,_:&mut u8){}fn f(p:&mut u8){take(p,p);}",
                false,
            ),
            (
                "fn take(_: &mut u8,_:u8){}fn f(p:&mut u8){take(p,{*p=2;0});}",
                false,
            ),
            (
                "fn take(_: &mut u8,_:u8){}fn f(p:&mut u8,c:bool){take(p,if c{*p}else{0});}",
                true,
            ),
            (
                "fn take(_: &mut u8,_:u8){}fn f(p:&mut u8,c:bool){take(p,if c{*p}else{return});}",
                true,
            ),
            (
                "fn take(_: &mut u8,_:u8){}fn f(p:&mut u8,c:bool){take(if c{p}else{return},*p);}",
                false,
            ),
        ],
    )
}

#[test]
fn field_disjointness_and_conservative_dynamic_indices() -> Result<(), Box<dyn std::error::Error>> {
    cases(
        "places",
        &[
            (
                "fn f(){let mut x=(0,0);let p=&mut x.0;let q=&mut x.1;*p=1;*q=2;}",
                true,
            ),
            ("fn f(){let mut x=(0,0);let p=&x.0;x.1=2;let y=*p;}", true),
            (
                "fn f(){let mut x=(0,0);let p=&x.0;x=(1,2);let y=*p;}",
                false,
            ),
            (
                "struct S{a:u8,b:u8}fn f(p:&mut S){let a=&mut p.a;let b=&mut p.b;*a=1;*b=2;}",
                true,
            ),
            (
                "struct S{a:u8,b:u8}fn f(p:&mut S){let a=&mut p.a;let b=&mut p.a;*a=1;*b=2;}",
                false,
            ),
            (
                "fn f(){let mut x=[0,1];let p=&mut x[0];let q=&mut x[1];*p=1;*q=2;}",
                false,
            ),
            (
                "fn f(){let mut x=[0,1];let [ref mut p,ref mut q]=x;*p=1;*q=2;}",
                true,
            ),
            (
                "fn f(){let mut x=[0,1,2];let [ref mut p,ref mut q@..]=x;*p=1;q[0]=2;}",
                true,
            ),
            (
                "fn f(){let mut x=[(0,1),(2,3)];let p=&mut x[0].0;let q=&mut x[1].1;*p=1;*q=2;}",
                true,
            ),
            (
                "fn f(){let mut x=(0,0);let p=&mut x;let q=&mut p.0;let r=&mut p.1;*q=1;*r=2;}",
                true,
            ),
        ],
    )
}

#[test]
fn storage_ends_and_loop_paths_bound_loans() -> Result<(), Box<dyn std::error::Error>> {
    cases(
        "storage",
        &[
            ("fn f(){let p;{let x=0;p=&x;}let y=*p;}", false),
            ("fn f(){let p;{let x=0;p=&x;let y=*p;}}", true),
            (
                "fn id(x:&u8)->&u8{x}fn value()->u8{1}fn f(){let p=id(&value());let y=*p;}",
                false,
            ),
            ("fn value()->u8{1}fn f(){let p=&value();let y=*p;}", true),
            (
                "fn f(c:bool){let mut x=0;let p=&x;if c{let y=*p;}else{x=1;}}",
                true,
            ),
            (
                "fn f(c:bool){let mut x=0;let p=&x;if c{x=1;}let y=*p;}",
                false,
            ),
            (
                "fn f(mut c:bool){let mut x=0;let p=&x;while c{let y=*p;x=1;c=false;}}",
                false,
            ),
            (
                "fn f(mut c:bool){let mut x=0;let p=&x;while c{let y=*p;c=false;}x=1;}",
                true,
            ),
            (
                "fn f(mut c:bool){let mut x=0;while c{let p=&mut x;*p=1;c=false;}x=2;}",
                true,
            ),
            ("fn f(){let mut x=0;let a=[&x;0];x=1;let b=a;}", true),
            ("fn f(){let mut x=0;let a=[&x;1];x=1;let b=a;}", false),
        ],
    )
}

#[test]
fn guards_preserve_protected_referents_and_release_divergent_paths()
-> Result<(), Box<dyn std::error::Error>> {
    cases(
        "guards",
        &[
            (
                "fn f(x:u8){let r;match x{a if {r=&a;true}=>(),_=>return}let y=*r;}",
                true,
            ),
            ("fn f(x:u8){match x{a if {let r=&a;*r==0}=>(),_=>()}}", true),
            (
                "fn f(p:&mut (u8,u8)){match *p{(0,_) if {p.0=1;true}=>(),_=>()}}",
                false,
            ),
            (
                "fn f(p:&mut (u8,u8)){match *p{(0,_) if {p.1=1;true}=>(),_=>()}}",
                true,
            ),
            (
                "fn f(p:&mut (u8,u8)){match *p{(r,_) if {p.0=1;r==0}=>(),_=>()}}",
                false,
            ),
            (
                "fn f(p:&mut (u8,u8)){match *p{(r,_) if {p.1=1;r==0}=>(),_=>()}}",
                true,
            ),
            (
                "fn f(p:&mut (u8,u8)){match *p{(0,_) if {p.0=1;return;}=>(),_=>()}}",
                true,
            ),
            (
                "fn f(p:&mut (u8,u8)){match *p{(r,_) if {p.0=1;let x=r;return;}=>(),_=>()}}",
                false,
            ),
            (
                "fn change(p:&mut u8)->bool{*p=1;true}fn f(p:&mut (u8,u8)){match *p{(0,_) if change(&mut p.0)=>(),_=>()}}",
                false,
            ),
            (
                "fn change(p:&mut u8)->bool{*p=1;true}fn f(p:&mut (u8,u8)){match *p{(0,_) if change(&mut p.1)=>(),_=>()}}",
                true,
            ),
            (
                "fn f(p:&mut (u8,u8)){match *p{(r,_) if match r{v if v==0=>true,_=>false}=>(),_=>()}}",
                true,
            ),
        ],
    )
}
