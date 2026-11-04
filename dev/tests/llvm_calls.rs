//! Real Rust entrypoints compare signed and unsigned LLVM operations with pinned Rust.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn checked(folder: &Path, label: &str, command: &mut Command) -> Result<Vec<u8>> {
    let output = command.output()?;
    std::fs::write(folder.join(format!("{label}.stderr")), &output.stderr)?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "LLVM {label}: {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}
#[test]
#[ignore = "requires configured CPPGM object backend and pinned independent Rust"]
fn unary_llvm_calls_execute_all_widths_without_external_llvm_symbols() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root.join(format!(
        "obj/parallel/llvm-linkage-runtime/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let fixtures = root.join("pa28/tests/spec/llvm-linkage");
    let observer = folder.join("observer.o");
    checked(
        &folder,
        "observer",
        Command::new("cc")
            .arg("-c")
            .arg(fixtures.join("observer.c"))
            .arg("-o")
            .arg(&observer),
    )?;
    for name in ["native", "native-signed", "native-mixed", "native-control"] {
        let source = fixtures.join(format!("{name}.rs"));
        let ir = folder.join(format!("{name}.lowir"));
        checked(
            &folder,
            &format!("{name}-frontend"),
            Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .args([
                    "--emit-lowir",
                    "--cap-lints=allow",
                    "--crate-name",
                    "llvm_linkage",
                ])
                .arg("-o")
                .arg(&ir)
                .arg(&source),
        )?;
        let native = folder.join(format!("{name}.native"));
        checked(
            &folder,
            &format!("{name}-backend"),
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg(&ir)
                .arg("--native-object")
                .arg(&observer)
                .arg("-o")
                .arg(&native),
        )?;
        let independent = folder.join(format!("{name}.rustc"));
        checked(
            &folder,
            &format!("{name}-independent"),
            Command::new("rustc")
                .env("RUSTC_BOOTSTRAP", "1")
                .args([
                    "+1.98.0",
                    "--edition=2024",
                    "--cap-lints=allow",
                    "--crate-name=llvm_linkage",
                    "-Cpanic=abort",
                    "-Cdebug-assertions=no",
                ])
                .arg(format!("-Clink-arg={}", observer.display()))
                .arg(&source)
                .arg("-o")
                .arg(&independent),
        )?;
        assert_eq!(
            checked(
                &folder,
                &format!("{name}-native-execute"),
                &mut Command::new(native)
            )?,
            b"1\n"
        );
        assert_eq!(
            checked(
                &folder,
                &format!("{name}-independent-execute"),
                &mut Command::new(independent)
            )?,
            b"1\n"
        );
    }
    Ok(())
}
