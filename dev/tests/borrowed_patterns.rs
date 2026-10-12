use std::{path::Path, process::Command};

const ACCEPTED: &[(&str, &str)] = &[
    ("shared-array", "fn f([x]:&[u8;1])->&u8{x}"),
    ("mutable-array", "fn f([x]:&mut[u8;1])->&mut u8{x}"),
    ("shared-wins", "fn f(xs:&&mut[u8;1]){let[x]=xs;}"),
    ("inner-shared", "fn f(xs:&mut&[u8;1]){let[x]=xs;}"),
    ("nested-mutable", "fn f(xs:&mut&mut[u8;1]){let[x]=xs;}"),
    ("explicit-shared", "fn f(xs:&[u8;1]){let &[x]=xs;}"),
    ("explicit-mutable", "fn f(xs:&mut[u8;1]){let &mut[x]=xs;}"),
    ("explicit-then-inherited", "fn f(xs:&&[u8;1]){let &[x]=xs;}"),
    ("explicit-nested", "fn f(xs:&[&u8;1]){let &[&x]=xs;}"),
    ("explicit-ref", "fn f(xs:&[u8;1]){let &[ref x]=xs;}"),
    (
        "explicit-ref-mut",
        "fn f(xs:&mut[u8;1]){let &mut[ref mut x]=xs;}",
    ),
    ("borrow-owned", "fn f(){let ref x=1u8;}"),
    ("borrow-owned-mut", "fn f(){let ref mut x=1u8;}"),
    (
        "borrow-local-mut",
        "fn f(mut xs:[u8;1]){let [ref mut x]=xs;}",
    ),
    ("no-initializer", "fn f(){let ref x:u8;}"),
    ("infer-reference", "fn f(){let &x;x=1u8;}"),
    ("tuple", "fn f(xs:&(u8,u16)){let(x,y)=xs;}"),
    ("record", "struct S{a:u8,b:u16}fn f(xs:&S){let S{a,b}=xs;}"),
    ("tuple-struct", "struct S(u8);fn f(xs:&mut S){let S(x)=xs;}"),
    ("unit-constructor", "struct S;fn f(xs:&S){let S=xs;}"),
    ("array-rest", "fn f(xs:&[u8;3]){let[a,tail@..]=xs;}"),
    ("array-empty-rest", "fn f(xs:&mut[u8;0]){let[tail@..]=xs;}"),
    ("slice-rest", "fn f(xs:&[u8]){let[tail@..]=xs;}"),
    ("slice-rest-mut", "fn f(xs:&mut[u8]){let[tail@..]=xs;}"),
    ("slice-wild-rest", "fn f(xs:&[u8]){let[..]=xs;}"),
    (
        "slice-explicit-ref",
        "fn f(xs:&[u8]){let &[ref tail@..]=xs;}",
    ),
    (
        "at-inherited",
        "struct S(u8);fn f(xs:&[S;1]){let[x@S(y)]=xs;}",
    ),
    ("at-reference", "fn f(xs:&[u8;1]){let whole@[x]=xs;}"),
    ("at-explicit", "fn f(xs:(u8,u8)){let ref whole@(a,b)=xs;}"),
    ("or-inherited", "fn f(xs:&[u8;1]){let([x]|[x])=xs;}"),
    (
        "or-capture-differs",
        "fn f(xs:(&u8,&[u8;1])){let((x,_)|(_,[x]))=xs;}",
    ),
    (
        "or-capture-mut-differs",
        "fn f(xs:(&mut u8,&mut[u8;1])){let((x,_)|(_,[x]))=xs;}",
    ),
    (
        "macro",
        "macro_rules! bind{($p:pat,$e:expr)=>{let $p=$e;}}fn f(xs:&[u8;1]){bind!([x],xs);let _=x;}",
    ),
    (
        "cfg-field",
        "struct S{a:u8}fn f(xs:&S){let S{#[cfg(no)] missing,a}=xs;}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    ("written-mut", "fn f(xs:&mut[u8;1]){let[mut x]=xs;}"),
    ("written-ref", "fn f(xs:&[u8;1]){let[ref x]=xs;}"),
    ("written-ref-mut", "fn f(xs:&mut[u8;1]){let[ref mut x]=xs;}"),
    ("inherited-reference", "fn f(xs:&[&u8;1]){let[&x]=xs;}"),
    (
        "inherited-reference-mut",
        "fn f(xs:&mut[&mut u8;1]){let[&mut x]=xs;}",
    ),
    ("inherited-at-mut", "fn f(xs:&[u8;1]){let[mut x@_]=xs;}"),
    (
        "inherited-at-child-mut",
        "struct S(u8);fn f(xs:&[S;1]){let[x@S(mut y)]=xs;}",
    ),
    ("shared-pattern-mut-type", "fn f(xs:&mut u8){let &x=xs;}"),
    ("mut-pattern-shared-type", "fn f(xs:&u8){let &mut x=xs;}"),
    ("nonreference-type", "fn f(xs:u8){let &x=xs;}"),
    (
        "or-written-modes",
        "fn f(xs:&[u8;1]){let([x]|&[ref x])=xs;}",
    ),
    (
        "or-written-modes-mut",
        "fn f(xs:&mut[u8;1]){let([x]|&mut[ref mut x])=xs;}",
    ),
    ("or-different-types", "fn f(xs:&[u8;1]){let([x]|&[x])=xs;}"),
    ("slice-empty-refutable", "fn f(xs:&[u8]){let[]=xs;}"),
    ("slice-prefix-refutable", "fn f(xs:&[u8]){let[x,..]=xs;}"),
    ("slice-owned-unsized", "fn f(xs:&[u8]){let &[tail@..]=xs;}"),
    ("array-length", "fn f(xs:&[u8;2]){let[x]=xs;}"),
    (
        "record-privacy",
        "mod m{pub struct S{a:u8}}fn f(xs:&m::S){let m::S{a}=xs;}",
    ),
];

#[test]
fn pattern_inspection_does_not_claim_region_or_loan_validity()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work/pattern-proof-boundary")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (name, source) in [
        ("escape", "fn f(value:u8)->&'static u8{let ref x=value;x}"),
        (
            "mutable-through-shared",
            "fn f(xs:&[u8;1]){let &[ref mut x]=xs;}",
        ),
    ] {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, source)?;
        let compiler = env!("CARGO_BIN_EXE_rustgm");
        let inspected = Command::new(compiler)
            .arg("--emit-patterns")
            .arg(&path)
            .output()?;
        assert!(
            inspected.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&inspected.stderr)
        );
        let view = std::str::from_utf8(&inspected.stdout)?;
        assert!(view.contains("loan=pending"));
        assert!(view.contains("origin=pattern-borrow pending"));
        std::fs::write(path.with_extension("patterns"), &inspected.stdout)?;
        let executed = Command::new(compiler)
            .arg("--emit-lowir")
            .arg(&path)
            .output()?;
        assert_eq!(executed.status.code(), Some(1));
        assert!(executed.stdout.is_empty());
        assert!(String::from_utf8_lossy(&executed.stderr).contains("region and loan proofs"));
        let checked = Command::new("rustc")
            .args(["--edition=2024", "--crate-type=lib", "--emit=metadata"])
            .arg(&path)
            .arg("--crate-name")
            .arg("proof_boundary")
            .arg("-o")
            .arg(folder.join("metadata"))
            .output()?;
        std::fs::write(path.with_extension("rustc-stderr"), &checked.stderr)?;
        assert_eq!(checked.status.code(), Some(1), "{name}");
        assert!(
            String::from_utf8_lossy(&checked.stderr).contains(if name == "escape" {
                "E0515"
            } else {
                "E0596"
            })
        );
    }
    Ok(())
}

#[test]
fn reference_pattern_views_and_errors_match_pinned_rustc() -> Result<(), Box<dyn std::error::Error>>
{
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work/borrowed-vectors")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let version = Command::new("rustc").arg("--version").output()?;
    assert!(String::from_utf8_lossy(&version.stdout).starts_with("rustc 1.98.0 "));
    std::fs::write(folder.join("rustc-version.txt"), &version.stdout)?;
    for (accepted, cases) in [(true, ACCEPTED), (false, REJECTED)] {
        for (name, source) in cases {
            let path = folder.join(format!("{name}.rs"));
            std::fs::write(&path, source)?;
            let compiler = env!("CARGO_BIN_EXE_rustgm");
            let output = Command::new(compiler)
                .arg("--emit-patterns")
                .arg(&path)
                .output()?;
            let wanted = Some(i32::from(!accepted));
            assert_eq!(
                output.status.code(),
                wanted,
                "{name}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            std::fs::write(path.with_extension("patterns"), &output.stdout)?;
            std::fs::write(path.with_extension("rustgm-stderr"), &output.stderr)?;
            let count = Command::new(compiler)
                .arg("--count-patterns")
                .arg(&path)
                .output()?;
            assert_eq!(count.status.code(), wanted, "{name}");
            if accepted {
                assert!(output.stderr.is_empty());
                assert!(count.stderr.is_empty());
                let view = std::str::from_utf8(&output.stdout)?;
                assert!(view.starts_with("selected-patterns 1\n"));
                let footer = view
                    .lines()
                    .last()
                    .and_then(|line| line.strip_prefix("eof "))
                    .ok_or("missing footer")?;
                assert_eq!(count.stdout, format!("{footer}\n").as_bytes());
                let again = Command::new(compiler)
                    .arg("--emit-patterns")
                    .arg(&path)
                    .output()?;
                assert!(again.status.success());
                assert_eq!(again.stdout, output.stdout);
                for mode in ["--emit-body-ir", "--count-ownership", "--emit-lowir"] {
                    let gate = Command::new(compiler).arg(mode).arg(&path).output()?;
                    assert_eq!(gate.status.code(), Some(1), "{name}: {mode}");
                    assert!(gate.stdout.is_empty());
                    assert!(
                        String::from_utf8_lossy(&gate.stderr).contains("region and loan proofs"),
                        "{name}: {}",
                        String::from_utf8_lossy(&gate.stderr)
                    );
                }
            } else {
                assert!(output.stdout.is_empty());
                assert!(count.stdout.is_empty());
                assert!(!output.stderr.is_empty());
            }
            let rustc = Command::new("rustc")
                .args([
                    "--edition=2024",
                    "--crate-type=lib",
                    "--crate-name",
                    "pattern_check",
                    "--emit=metadata",
                ])
                .arg(&path)
                .arg("-o")
                .arg(folder.join("metadata"))
                .output()?;
            std::fs::write(path.with_extension("rustc-stderr"), &rustc.stderr)?;
            assert_eq!(
                rustc.status.code(),
                wanted,
                "{name}: {}",
                String::from_utf8_lossy(&rustc.stderr)
            );
        }
    }
    Ok(())
}
