//! Actual ?Sized original, sized instantiation and destruction run through CPPGM.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn checked(folder: &Path, name: &str, command: &mut Command) -> Result<Vec<u8>> {
    let output = command.output()?;
    std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "{name}: {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}
#[test]
#[ignore = "requires configured CPPGM backend and pinned independent Rust"]
fn generic_unsized_parameter_original_executes_sized_instance_and_drop() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root.join(format!(
        "obj/parallel/unsized-parameter-runtime/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let source = root.join("pa30/tests/spec/unsized-parameters/native.rs");
    let observer = folder.join("observer.o");
    checked(
        &folder,
        "observer",
        Command::new("cc")
            .arg("-c")
            .arg(root.join("pa30/tests/spec/unsized-parameters/observer.c"))
            .arg("-o")
            .arg(&observer),
    )?;
    let ir = folder.join("native.lowir");
    checked(
        &folder,
        "frontend",
        Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-lowir", "--cap-lints=allow"])
            .arg(&source)
            .arg("-o")
            .arg(&ir),
    )?;
    let native = folder.join("native.exe");
    checked(
        &folder,
        "backend",
        Command::new("python3")
            .arg(root.join("scripts/backend.py"))
            .arg(ir)
            .arg("--native-object")
            .arg(&observer)
            .arg("-o")
            .arg(&native),
    )?;
    assert_eq!(
        checked(&folder, "native-execute", &mut Command::new(native))?,
        b"7\n9\n"
    );
    let independent = folder.join("independent.exe");
    checked(
        &folder,
        "independent",
        Command::new("rustc")
            .env("RUSTC_BOOTSTRAP", "1")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--cap-lints=allow",
                "-Cpanic=abort",
            ])
            .arg(format!("-Clink-arg={}", observer.display()))
            .arg(source)
            .arg("-o")
            .arg(&independent),
    )?;
    assert_eq!(
        checked(
            &folder,
            "independent-execute",
            &mut Command::new(independent)
        )?,
        b"7\n9\n"
    );
    Ok(())
}
