use std::{path::Path, process::Command};

#[test]
fn slice_execution_preserves_access_lifetime_and_alias_rejections()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/wide-constraints-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for (name, body) in [
        ("shared-write", "fn f(s:&[u8]){s[0]=7;}"),
        ("local-return", "fn f()->&'static [u8]{let a=[7];&a}"),
        ("mutable-return", "fn f()->&'static mut [u8]{&mut [7]}"),
        (
            "indexed-alias",
            "fn f(s:&mut [u8]){let a=&mut s[0];let b=&mut s[1];*a+=*b;}",
        ),
        (
            "live-shared",
            "fn f(s:&mut [u8])->u8{let a=&s[0];s[0]=1;*a}",
        ),
        ("owned-move", "struct N;fn f(s:&[N]){let a=s[0];}"),
        ("literal-mutable", "fn f(){let a:&mut [u8]=b\"x\";}"),
        ("string-index", "fn f(s:&str){let a=s[0];}"),
    ] {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, format!("{body}fn main(){{}}"))?;
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
        assert_eq!(independent.status.code(), Some(1), "{name}");
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
            assert_eq!(candidate.status.code(), Some(1), "{name} {mode}");
            assert!(candidate.stdout.is_empty() && !candidate.stderr.is_empty());
            errors.push(candidate.stderr);
        }
        assert_eq!(errors[0], errors[1], "{name}");
    }
    Ok(())
}
