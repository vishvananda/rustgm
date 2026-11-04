//! Selected static libraries and suppressed missing dependencies execute real Rust entries.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn run(folder: &Path, name: &str, command: &mut Command) -> Result<Vec<u8>> {
    let output = command.output()?;
    std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "{name}: {:?}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}
fn prepare(root: &Path, folder: &Path) -> Result {
    std::fs::create_dir_all(folder)?;
    let fixtures = root.join("pa29/tests/spec/conditional-native-links");
    run(
        folder,
        "selected-object",
        Command::new("cc")
            .arg("-c")
            .arg(fixtures.join("selected.c"))
            .arg("-o")
            .arg(folder.join("selected.o")),
    )?;
    run(
        folder,
        "selected-archive",
        Command::new("ar")
            .arg("rcs")
            .arg(folder.join("libselected_native.a"))
            .arg(folder.join("selected.o")),
    )?;
    run(
        folder,
        "observer-object",
        Command::new("cc")
            .arg("-c")
            .arg(fixtures.join("observer.c"))
            .arg("-o")
            .arg(folder.join("observer.o")),
    )?;
    Ok(())
}
#[test]
#[ignore = "requires the configured CPPGM object backend and pinned independent Rust"]
fn conditional_static_dependencies_execute_without_linking_false_libraries() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repository root")?;
    let folder = root.join(format!(
        "obj/parallel/conditional-native-links-runtime/{}",
        std::process::id()
    ));
    prepare(root, &folder)?;
    for name in ["native", "native-control"] {
        let source = root.join(format!(
            "pa29/tests/spec/conditional-native-links/{name}.rs"
        ));
        let lowir = folder.join(format!("{name}.lowir"));
        let manifest = folder.join(format!("{name}.json"));
        run(
            &folder,
            &format!("{name}-frontend"),
            Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .args([
                    "--emit-lowir",
                    "--cap-lints=allow",
                    "--crate-name",
                    "conditional_native",
                    "--native-links-output",
                ])
                .arg(&manifest)
                .arg("-o")
                .arg(&lowir)
                .arg(&source),
        )?;
        assert_eq!(
            std::fs::read_to_string(&manifest)?,
            "{\"schema\":1,\"target\":\"x86_64-linux\",\"libraries\":[{\"kind\":\"static\",\"name\":\"selected_native\"}]}\n"
        );
        let candidate = folder.join(format!("{name}.native"));
        run(
            &folder,
            &format!("{name}-backend"),
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg(&lowir)
                .arg("--native-links")
                .arg(&manifest)
                .arg("--native-search")
                .arg(&folder)
                .arg("--native-object")
                .arg(folder.join("observer.o"))
                .arg("-o")
                .arg(&candidate),
        )?;
        let independent = folder.join(format!("{name}.rustc"));
        run(
            &folder,
            &format!("{name}-rustc"),
            Command::new("rustc")
                .env("RUSTC_BOOTSTRAP", "1")
                .args([
                    "+1.98.0",
                    "--edition=2024",
                    "--crate-name=conditional_native",
                    "--cap-lints=allow",
                    "-Cpanic=abort",
                    "-Cdebug-assertions=no",
                ])
                .arg(format!("-Lnative={}", folder.display()))
                .arg(format!(
                    "-Clink-arg={}",
                    folder.join("observer.o").display()
                ))
                .arg(&source)
                .arg("-o")
                .arg(&independent),
        )?;
        assert_eq!(
            run(
                &folder,
                &format!("{name}-execute"),
                &mut Command::new(candidate)
            )?,
            b"7 9\n"
        );
        assert_eq!(
            run(
                &folder,
                &format!("{name}-independent-execute"),
                &mut Command::new(independent)
            )?,
            b"7 9\n"
        );
    }
    assert_eq!(
        std::fs::read(folder.join("native.lowir"))?,
        std::fs::read(folder.join("native-control.lowir"))?
    );
    Ok(())
}
