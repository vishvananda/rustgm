use std::{path::Path, process::Command};

const CASES: &[(&str, &str, bool)] = &[
    (
        "unreachable-borrow",
        "#[repr(packed)]struct P(u64);fn f(p:P){return;let _=&p.0;}",
        false,
    ),
    (
        "unreachable-pattern",
        "#[repr(packed)]struct P(u64);fn f(p:P){return;let P(ref a)=p;}",
        false,
    ),
    (
        "unreachable-match",
        "#[repr(packed)]struct P(u64);fn f(p:P){return;match p{P(ref a)=>()}}",
        false,
    ),
    (
        "unreachable-guard",
        "#[repr(packed)]struct P(u64);fn f(p:P){return;match p{P(a) if a>0=>(),_=>()}}",
        false,
    ),
    (
        "unreachable-conditional",
        "#[repr(packed)]struct P(u64);fn f(p:P){return;if let P(ref a)=p{}}",
        false,
    ),
    (
        "unreachable-dereference",
        "#[repr(packed)]struct P<'a>(&'a u64);fn f(p:P<'_>){return;let _=&*p.0;}",
        true,
    ),
    (
        "unreachable-copy",
        "#[repr(packed)]struct P(u64);fn f(p:P){return;let P(a)=p;}",
        true,
    ),
    (
        "packed-shared",
        "#[repr(packed)]struct P{a:u8,b:u64}fn f(p:P){let _=&p.b;}",
        false,
    ),
    (
        "packed-mutable",
        "#[repr(packed)]struct P{a:u8,b:u64}fn f(mut p:P){let _=&mut p.b;}",
        false,
    ),
    (
        "packed-first-field",
        "#[repr(packed)]struct P(u64);fn f(p:P){let _=&p.0;}",
        false,
    ),
    (
        "packed-zst",
        "#[repr(packed)]struct P{a:u8,b:[u64;0]}fn f(p:P){let _=&p.b;}",
        false,
    ),
    (
        "packed-nested",
        "struct I{a:u64}#[repr(packed)]struct P(I);fn f(p:P){let _=&p.0.a;}",
        false,
    ),
    (
        "packed-array",
        "#[repr(packed)]struct P([u64;2]);fn f(p:P){let _=&p.0[1];}",
        false,
    ),
    (
        "packed-array-pattern",
        "#[repr(packed)]struct P([u64;2]);fn f(p:P){let [ref a,..]=p.0;}",
        false,
    ),
    (
        "packed-binding",
        "#[repr(packed)]struct P(u64);fn f(p:P){let P(ref a)=p;}",
        false,
    ),
    (
        "packed-guard",
        "#[repr(packed)]struct P{a:u8,b:u64}fn f(p:P)->u64{match p{P{b,..} if b>0=>b,_=>0}}",
        false,
    ),
    (
        "packed-pointer-storage",
        "#[repr(packed)]struct P<'a>(&'a u64);fn f(p:P<'_>){let _=&p.0;}",
        false,
    ),
    (
        "packed-two",
        "#[repr(packed(2))]struct P(u32);fn f(p:P){let _=&p.0;}",
        false,
    ),
    (
        "nested-packed",
        "#[repr(packed)]struct I(u64);struct P(I);fn f(p:P){let _=&p.0.0;}",
        false,
    ),
    (
        "packed-byte",
        "#[repr(packed)]struct P{a:u8,b:u64}fn f(p:P){let _=&p.a;}",
        true,
    ),
    (
        "packed-owned",
        "#[repr(packed)]struct P{a:u8,b:u64}fn f(p:P)->u64{p.b}",
        true,
    ),
    (
        "packed-copied",
        "#[repr(packed)]struct P(u64);fn f(p:P){let x=p.0;let _=&x;}",
        true,
    ),
    (
        "packed-aggregate",
        "#[repr(packed)]struct P(u64);fn f(p:P){let _=&p;}",
        true,
    ),
    (
        "packed-two-valid",
        "#[repr(packed(2))]struct P{a:u8,b:u16}fn f(p:P){let _=&p.b;}",
        true,
    ),
    (
        "packed-eight",
        "#[repr(packed(8))]struct P{a:u8,b:u64}fn f(p:P){let _=&p.b;}",
        true,
    ),
    (
        "packed-dereference",
        "#[repr(packed)]struct P<'a>(&'a u64);fn f(p:P<'_>){let _=&*p.0;}",
        true,
    ),
    (
        "packed-dereference-field",
        "struct I{a:u64}#[repr(packed)]struct P<'a>(&'a I);fn f(p:P<'_>){let _=&p.0.a;}",
        true,
    ),
    (
        "packed-dereference-guard",
        "#[repr(packed)]struct P<'a>(&'a (u64,));fn f(p:P<'_>)->u64{match *p.0{(a,) if a>0=>a,_=>0}}",
        true,
    ),
    (
        "aligned-zero-sized",
        "#[repr(align(64))]struct Z;fn f(x:Z){let _=&x;}",
        true,
    ),
    (
        "aligned-wrapper",
        "#[repr(align(64))]struct P(u64);fn f(p:P){let _=&p.0;}",
        true,
    ),
    (
        "ordinary-array",
        "fn f(p:[u64;2]){let _=&p[1];let [ref a,..]=p;}",
        true,
    ),
    (
        "ordinary-field",
        "struct P{a:u8,b:u64}fn f(p:P){let _=&p.b;}",
        true,
    ),
];

#[test]
fn alignment_checks_unused_borrows_and_resets_at_dereferences()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/reference-alignment-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    for &(name, source, accepted) in CASES {
        let path = directory.join(format!("{name}.rs"));
        std::fs::write(&path, format!("{source}fn main(){{}}"))?;
        let independent = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--cap-lints=allow",
                "--crate-name=probe",
                "--emit=metadata",
                "-o",
            ])
            .arg(directory.join("probe.rmeta"))
            .arg(&path)
            .output()?;
        std::fs::write(path.with_extension("rustc.stderr"), &independent.stderr)?;
        assert_eq!(
            independent.status.code(),
            Some(i32::from(!accepted)),
            "rustc {name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        for mode in ["--emit-ownership", "--count-ownership", "--emit-lowir"] {
            let candidate = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&path)
                .output()?;
            assert_eq!(
                candidate.status.code(),
                Some(i32::from(!accepted)),
                "{name} {mode}: {}",
                String::from_utf8_lossy(&candidate.stderr)
            );
            if accepted {
                assert!(candidate.stderr.is_empty());
            } else {
                assert!(candidate.stdout.is_empty());
                assert!(
                    String::from_utf8_lossy(&candidate.stderr).contains("unaligned packed field"),
                    "{name} {mode}: {}",
                    String::from_utf8_lossy(&candidate.stderr)
                );
            }
        }
    }
    Ok(())
}
