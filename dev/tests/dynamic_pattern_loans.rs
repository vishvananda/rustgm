use std::{path::Path, process::Command};

const CASES: &[(&str, bool)] = &[
    ("fn f(s:&mut [u8]){if let [a,..,b]=s{*a+=*b;}}", true),
    (
        "fn f(s:&mut [u8]){if let [a,rest @ ..]=s{let b=&mut rest[0];*a+=*b;}}",
        true,
    ),
    (
        "fn f(s:&mut [u8]){if let [a,rest @ ..,z]=s{if let [b,inner @ ..,y]=rest{*a+=*b;inner[0]+=1;*z+=*y;}}}",
        true,
    ),
    (
        "struct S{a:u8,b:u8}fn f(s:&mut [S]){if let [..,S{a,b}]=s{*a+=*b;}}",
        true,
    ),
    ("fn f(s:&mut [u8]){if let [a,_,..,b]=s{*a+=*b;}}", true),
    (
        "fn f(s:&mut [u8]){if let [a,..]=s{if let [..,b]=s{*a+=*b;}}}",
        false,
    ),
    (
        "fn f(s:&mut [u8]){if let [a,..,b]=s{let c=&mut s[1];*a+=*c;*b+=1;}}",
        false,
    ),
    (
        "fn f(s:&mut [u8]){if let [a,rest @ ..]=s{let b=&mut s[0];*a+=*b;let _=rest;}}",
        false,
    ),
    (
        "fn f(s:&mut [u8]){if let [..,b]=s{let c=&mut s[0];*b+=*c;}}",
        false,
    ),
    (
        "fn f(s:&mut [u8]){match s{[a,rest @ ..] if {rest[0]=1;true}=>*a=2,_=>()}}",
        false,
    ),
    (
        "fn f(s:&mut [u8]){match s{[a,..,b] if {*a=1;true}=>*b=2,_=>()}}",
        false,
    ),
    ("fn f(s:&[u8]){if let [a,rest @ ..]=s{rest[0]=*a;}}", false),
    ("struct N;fn f(s:&[N]){if let &[a,..]=s{let _=a;}}", false),
    (
        "fn f()->&'static [u8]{let a=[7];let s:&[u8]=&a;let [rest @ ..]=s;rest}",
        false,
    ),
];

#[test]
fn dynamic_captures_preserve_disjointness_and_reject_overlapping_access()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/dynamic-pattern-loans-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for (i, &(source, accepted)) in CASES.iter().enumerate() {
        let path = folder.join(format!("case-{i}.rs"));
        std::fs::write(&path, format!("{source}fn main(){{}}"))?;
        let independent = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=probe",
                "--cap-lints=allow",
                "--emit=metadata",
            ])
            .arg(&path)
            .arg("-o")
            .arg(path.with_extension("rmeta"))
            .output()?;
        std::fs::write(path.with_extension("rustc.stderr"), &independent.stderr)?;
        let status = Some(i32::from(!accepted));
        assert_eq!(
            independent.status.code(),
            status,
            "rustc {source}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        let mut errors = Vec::new();
        for mode in ["--emit-lowir", "--count-lowir"] {
            let candidate = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&path)
                .output()?;
            std::fs::write(
                path.with_extension(format!("{mode}.stderr")),
                &candidate.stderr,
            )?;
            assert_eq!(
                candidate.status.code(),
                status,
                "{mode} {source}: {}",
                String::from_utf8_lossy(&candidate.stderr)
            );
            assert_eq!(candidate.stdout.is_empty(), !accepted);
            assert_eq!(candidate.stderr.is_empty(), accepted);
            errors.push(candidate.stderr);
        }
        assert_eq!(errors[0], errors[1]);
    }
    Ok(())
}
