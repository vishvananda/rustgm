//! Version-nine source roles and real generic originals survive a fresh CLI store.
use super::{TestResult, root, run};
use std::{path::Path, process::Command};

fn provider(folder: &Path) -> TestResult<std::path::PathBuf> {
    let metadata = folder.join("provider.rgcrate");
    run(
        folder,
        "provider",
        Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-lowir",
                "--edition=2024",
                "--cap-lints=allow",
                "--crate-type=lib",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "from_fallback",
                "--source-core",
                "minicore@from_fallback",
                "--source-profile",
                "module-conversions-v9",
            ])
            .arg("--metadata-output")
            .arg(&metadata)
            .arg("-o")
            .arg(folder.join("provider.lowir"))
            .arg(root()?.join("pa31/tests/spec/source-from/nested-into-dot.rs")),
    )?;
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(bytes.get(13..15), Some([9, 0].as_slice()));
    let decoded = rustgm::support::crates::decode(&bytes)?;
    assert_eq!(rustgm::support::crates::encode(&decoded)?, bytes);
    let mut older = bytes;
    older[13] = 8;
    assert!(rustgm::support::crates::decode(&older).is_err());
    Ok(metadata)
}

fn qualify(native: bool) -> TestResult {
    let root = root()?;
    let lane = if native { "native" } else { "frontend" };
    let folder = root
        .join("obj/parallel/source-from-clients")
        .join(format!("{lane}-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    let metadata = provider(&folder)?;
    let mut failures = Vec::new();
    for (name, expected) in [("default", b"32\n"), ("explicit", b"64\n")] {
        let result = (|| -> TestResult {
            let lowir = folder.join(format!("{name}.lowir"));
            run(
                &folder,
                &format!("{name}-client"),
                Command::new(env!("CARGO_BIN_EXE_rustgm"))
                    .args([
                        "--emit-lowir",
                        "--edition=2024",
                        "--cap-lints=allow",
                        "--crate-type=lib",
                        "--crate-name",
                        "conversion_client",
                        "--crate-disambiguator",
                        name,
                        "--source-client",
                        "minicore@from_fallback",
                        "--extern",
                    ])
                    .arg(format!("minicore={}", metadata.display()))
                    .arg("-o")
                    .arg(&lowir)
                    .arg(root.join(format!("pa31/tests/spec/source-from-clients/{name}.rs"))),
            )?;
            assert!(!std::fs::read(&lowir)?.is_empty());
            if native {
                let object = folder.join(format!("{name}.o"));
                let executable = folder.join(format!("{name}.native"));
                run(
                    &folder,
                    &format!("{name}-backend"),
                    Command::new("python3")
                        .arg(root.join("scripts/backend.py"))
                        .arg("--emit-object")
                        .arg(&lowir)
                        .arg("-o")
                        .arg(&object),
                )?;
                run(
                    &folder,
                    &format!("{name}-link"),
                    Command::new("cc")
                        .arg(root.join("pa31/tests/spec/source-from-clients/observer.c"))
                        .arg(&object)
                        .arg("-lm")
                        .arg("-o")
                        .arg(&executable),
                )?;
                let output = run(
                    &folder,
                    &format!("{name}-execute"),
                    &mut Command::new(&executable),
                )?;
                if output.stdout != expected {
                    return Err(format!(
                        "{name}: expected {expected:?}, actual {:?}",
                        output.stdout
                    )
                    .into());
                }
                std::fs::remove_file(object)?;
                std::fs::remove_file(executable)?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            failures.push(format!("{name}: {error}"));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n").into())
    }
}

#[test]
fn imported_from_and_into_keep_real_generic_bodies_and_explicit_float_constraints() -> TestResult {
    qualify(false)
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and host C linker"]
fn imported_conversion_bodies_match_independent_native_results() -> TestResult {
    qualify(true)
}
