#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{body, borrow::ownership};

const PENDING: &str = "reference expression execution requires region and loan proofs";

fn inspect(source: &str) -> Result<(), String> {
    let (report, mut facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    ownership::check(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        &mut facts.copies,
        &program,
    )
    .map(|_| ())
    .map_err(|e| e.message.to_owned())
}

fn compare(name: &str, cases: &[(&str, Option<&str>)]) -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/reference-access-{}-{name}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory)?;
    for (index, (source, error)) in cases.iter().enumerate() {
        let path = directory.join(format!("case-{index}.rs"));
        std::fs::write(&path, source)?;
        let rustc = std::process::Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=probe",
                "--crate-type=lib",
                "--emit=metadata",
                "--cap-lints=allow",
                "-o",
            ])
            .arg(directory.join("probe.rmeta"))
            .arg(&path)
            .output()?;
        std::fs::write(
            directory.join(format!("case-{index}.stderr")),
            &rustc.stderr,
        )?;
        assert_eq!(
            rustc.status.code(),
            Some(i32::from(error.is_some())),
            "rustc {source}: {}",
            String::from_utf8_lossy(&rustc.stderr)
        );
        let result = inspect(source);
        // Success here is an access proof only: valid reference operations still
        // reach the mandatory loan/region gate instead of receiving Checked.
        assert_eq!(
            result.as_ref().err().map(String::as_str),
            Some(error.unwrap_or(PENDING)),
            "RustGM {source}"
        );
    }
    Ok(())
}

#[test]
fn mutable_access_matches_rustc() -> Result<(), Box<dyn std::error::Error>> {
    compare(
        "mutable",
        &[
            ("fn f(){let mut x=0u8;let p=&mut x;*p=1;}", None),
            (
                "fn f(){let x=0u8;let p=&mut x;}",
                Some("cannot mutably borrow an immutable local"),
            ),
            (
                "fn f(){let x=(0u8,1u8);let p=&mut x.0;}",
                Some("cannot mutably borrow an immutable local"),
            ),
            ("fn f(){let mut x=(0u8,1u8);let p=&mut x.0;}", None),
            ("fn f(){let p=&mut 0u8;*p=1;}", None),
            ("fn f(p:&mut u8){*p=1;}", None),
            (
                "fn f(p:&u8){*p=1;}",
                Some("cannot assign through a shared reference"),
            ),
            (
                "fn f(p:&u8){let q=&mut *p;}",
                Some("cannot mutably borrow shared or static storage"),
            ),
            (
                "fn f(p:&&mut u8){**p=1;}",
                Some("cannot assign through a shared reference"),
            ),
            (
                "fn f(p:&&mut u8){let q=&mut **p;}",
                Some("cannot mutably borrow shared or static storage"),
            ),
            ("fn f(p:&mut &mut u8){**p=1;}", None),
            (
                "fn f(p:&mut &u8){**p=1;}",
                Some("cannot assign through a shared reference"),
            ),
            ("fn f(p:(&mut u8,u8)){*p.0=1;}", None),
            ("fn f(p:[&mut u8;1]){*p[0]=1;}", None),
            ("fn f(p:[&mut u8;1]){let q=&mut *p[0];}", None),
            (
                "fn f(p:&[&mut u8;1]){*p[0]=1;}",
                Some("cannot assign through a shared reference"),
            ),
            ("fn f(ref mut p:u8){*p=1;}", None),
            ("fn f((ref mut p,_):(u8,u8)){*p=1;}", None),
            ("static X:u8=1;fn f(){let p=&X;}", None),
            (
                "static X:u8=1;fn f(){let p=&mut X;}",
                Some("cannot mutably borrow shared or static storage"),
            ),
        ],
    )
}

#[test]
fn reference_initialization_and_moves_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    compare(
        "initialization",
        &[
            (
                "fn f(){let x:u8;let p=&x;}",
                Some("borrowed place may be uninitialized or moved"),
            ),
            (
                "fn f(){let mut x:u8;let p=&mut x;}",
                Some("borrowed place may be uninitialized or moved"),
            ),
            (
                "fn f(){let p:&mut u8;*p=1;}",
                Some("dereferenced pointer may be uninitialized or moved"),
            ),
            (
                "fn f(){let p:&u8;let x=*p;}",
                Some("dereferenced pointer may be uninitialized or moved"),
            ),
            (
                "fn f(c:bool,p:&mut u8){let q;if c{q=p;}*q=1;}",
                Some("dereferenced pointer may be uninitialized or moved"),
            ),
            (
                "fn f(p:&mut u8){let q=p;*q=1;*p=2;}",
                Some("dereferenced pointer may be uninitialized or moved"),
            ),
            ("fn f(p:&mut u8){let q:&mut u8=p;*q=1;*p=2;}", None),
            ("fn f(p:&u8)->u8{let q=p;*p+*q}", None),
            (
                "struct N;fn f(p:&N){let n=*p;}",
                Some("cannot move a non-Copy value out of a reference"),
            ),
            (
                "struct N;fn f(p:&mut N){let n=*p;}",
                Some("cannot move a non-Copy value out of a reference"),
            ),
            ("struct N;fn f(p:&mut N){*p=N;}", None),
            ("struct N;fn f(p:&(N,u8))->u8{p.1}", None),
            (
                "struct N;fn f(p:&(N,u8)){let x=p.0;}",
                Some("cannot move a non-Copy value out of a reference"),
            ),
            (
                "struct N;fn f(){let x=N;let y=x;let p=&x;}",
                Some("borrowed place may be uninitialized or moved"),
            ),
            ("struct N;fn f(){let x=(N,N);let y=x.0;let p=&x.1;}", None),
            (
                "struct N;fn f(){let x=(N,N);let y=x.0;let p=&x;}",
                Some("borrowed place may be uninitialized or moved"),
            ),
            ("struct N;fn f(p:&[N;0]){let []=*p;let _=p;}", None),
            (
                "struct N;fn f(p:&[N;1]){let [x]=*p;}",
                Some("cannot move a non-Copy value out of a reference"),
            ),
        ],
    )
}

#[test]
fn guard_reference_permissions_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    compare(
        "guards",
        &[
            ("fn f(p:&mut u8){match p{x if *x>0=>*x=1,_=>()}}", None),
            (
                "fn f(p:&mut u8){match p{x if {*x=1;true}=>(),_=>()}}",
                Some("cannot assign through a guard binding"),
            ),
            (
                "fn f(p:&mut u8){match p{x if {let q=&mut *x;true}=>(),_=>()}}",
                Some("cannot mutably borrow through a guard binding"),
            ),
            (
                "fn f(p:&mut(u8,u8)){match p{(x,y) if *x>0=>*y=1,_=>()}}",
                None,
            ),
            (
                "fn f(p:&mut(u8,u8)){match p{(x,y) if {*y=1;true}=>(),_=>()}}",
                Some("cannot assign through a guard binding"),
            ),
            (
                "fn f(p:&mut u8){match p{x if {let q=&*x;*q>0}=>(),_=>()}}",
                None,
            ),
        ],
    )
}
