//! Persistent independent C ABI controls; pinned Rust's return ICE stays explicit.
use super::{ROOT, TestResult};
use std::{
    path::Path,
    process::{Command, Output},
};
fn run(folder: &Path, name: &str, command: &mut Command) -> TestResult<Output> {
    let result = command.output()?;
    std::fs::write(folder.join(format!("{name}.stdout")), &result.stdout)?;
    std::fs::write(folder.join(format!("{name}.stderr")), &result.stderr)?;
    assert!(
        result.status.success(),
        "{name}: {}: {}",
        result.status,
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(result)
}
#[test]
#[ignore = "requires the pinned CPPGM object profile, pinned Rust and a host C compiler"]
fn actual_pointer_arguments_and_normal_return_execute_with_original_c_observers() -> TestResult {
    super::actual_export_import_wrapper_copy_pressure_and_return_bodies_emit_checked_lowir()?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    for (name, expected) in [
        ("native-export", b"7\n".as_slice()),
        ("native-return", b"7\n"),
        ("native-wrapper", b"7\n"),
        ("native-caller-copy", b"16\n"),
        ("native-register-pressure", b"16\n"),
    ] {
        let folder = root.join(format!(
            "obj/parallel/forced-indirect-abi-v1/candidate/{name}"
        ));
        let observer = root.join(format!("{ROOT}/{name}.c"));
        let object = folder.join("candidate.o");
        run(
            &folder,
            "candidate-object",
            Command::new("python3")
                .current_dir(root)
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(folder.join("provider.lowir"))
                .arg("-o")
                .arg(&object),
        )?;
        let candidate = folder.join("candidate.native");
        run(
            &folder,
            "candidate-link",
            Command::new("cc")
                .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
                .arg(&observer)
                .arg(&object)
                .arg("-o")
                .arg(&candidate),
        )?;
        assert_eq!(
            run(&folder, "candidate-native", &mut Command::new(&candidate))?.stdout,
            expected,
            "{name}"
        );
        // The pinned compiler accepts return metadata but ICEs during codegen.
        // Preserve that unqualified reference positive; the C ABI observer still runs.
        if name != "native-return" {
            rust_reference(root, &folder, name, &observer, expected)?;
        }
    }
    Ok(())
}
fn rust_reference(
    root: &Path,
    folder: &Path,
    name: &str,
    observer: &Path,
    expected: &[u8],
) -> TestResult {
    let archive = folder.join("reference.a");
    let output = run(
        folder,
        "rust-staticlib",
        Command::new("rustc")
            .env("RUSTC_BOOTSTRAP", "1")
            .args([
                "+1.98.0",
                "--crate-name=indirect_native",
                "--crate-type=staticlib",
                "--edition=2024",
                "--target=x86_64-unknown-linux-gnu",
                "-Cpanic=abort",
                "-Coverflow-checks=on",
                "-Cdebug-assertions=off",
                "--print=native-static-libs",
            ])
            .arg(root.join(format!("{ROOT}/{name}.rs")))
            .arg("-o")
            .arg(&archive),
    )?;
    let diagnostics = std::str::from_utf8(&output.stderr)?;
    let libraries = diagnostics
        .lines()
        .find_map(|line| {
            line.split_once("native-static-libs:")
                .map(|(_, libraries)| libraries)
        })
        .ok_or("missing Rust runtime libraries")?;
    let reference = folder.join("reference.native");
    run(
        folder,
        "rust-link",
        Command::new("cc")
            .arg(observer)
            .arg(&archive)
            .args(libraries.split_whitespace())
            .arg("-o")
            .arg(&reference),
    )?;
    assert_eq!(
        run(folder, "rust-native", &mut Command::new(&reference))?.stdout,
        expected,
        "{name}"
    );
    Ok(())
}
