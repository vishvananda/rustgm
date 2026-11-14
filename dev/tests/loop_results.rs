//! Repeated callback results get fresh identities without forgetting input loans.
use crate::{provider_traits, resolve};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn packet(owner: &str, count: usize, source: bool) -> Result {
    let prefix = format!("{owner}/tests/spec/loop-results");
    let table = resolve::fixture(&format!("{prefix}/cases.tsv"))?;
    let mut observed = 0;
    for line in table.lines() {
        let columns: Vec<_> = line.split('\t').collect();
        let [path, status, reason] = columns.as_slice() else {
            return Err("invalid loop result outcome row".into());
        };
        let input = resolve::fixture(path)?;
        let result = if source {
            provider_traits::lower(&input)
        } else {
            provider_traits::lower_report(&resolve::resolve(&input, true)?)
        };
        if *status == "0" {
            result.map_err(|error| format!("{path}: {error}"))?;
        } else {
            let error = result.err().ok_or_else(|| format!("{path}: accepted"))?;
            assert!(error.to_string().contains(reason), "{path}: {error}");
        }
        observed += 1;
    }
    assert_eq!(observed, count);
    Ok(())
}

#[test]
fn repeated_safe_results_preserve_storage_and_input_opposition() -> Result {
    packet("pa16", 8, false)
}

#[test]
fn repeated_raw_results_pass_original_and_source_proofs() -> Result {
    packet("pa27", 8, false)?;
    packet("pa31", 9, true)
}

#[test]
#[ignore = "requires pinned Rust, CPPGM and the host C linker"]
fn repeated_results_execute_two_distinct_referents() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root
        .join("obj/parallel/loop-results-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for owner in ["pa27", "pa31"] {
        for (stem, observer, expected) in [
            ("native", "observer.c", "9\n"),
            ("native-observed", "observed.c", "9 3 9\n"),
        ] {
            native(root, &folder, owner, stem, observer, expected)?;
        }
    }
    Ok(())
}

fn native(
    root: &std::path::Path,
    folder: &std::path::Path,
    owner: &str,
    stem: &str,
    observer: &str,
    expected: &str,
) -> Result {
    use crate::provider_traits::native::run;
    use std::process::Command;
    let packet = root.join(owner).join("tests/spec/loop-results");
    let input = packet.join(format!("{stem}.rs"));
    let name = format!("{owner}-{stem}");
    let object = folder.join(format!("{name}.reference.o"));
    run(
        folder,
        &format!("{name}-rustc"),
        Command::new("rustc")
            .env("RUSTC_BOOTSTRAP", "1")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-type=lib",
                "--emit=obj",
                "-Cpanic=abort",
                "-Cdebug-assertions=no",
                "-Coverflow-checks=yes",
                "--cap-lints=allow",
                "--crate-name=loop_results",
            ])
            .arg(&input)
            .arg("-o")
            .arg(&object),
    )?;
    let reference = folder.join(format!("{name}.reference"));
    run(
        folder,
        &format!("{name}-reference-link"),
        Command::new("cc")
            .arg(packet.join(observer))
            .arg(object)
            .arg("-o")
            .arg(&reference),
    )?;
    let output = Command::new(reference).output()?;
    std::fs::write(
        folder.join(format!("{name}-reference.stdout")),
        &output.stdout,
    )?;
    std::fs::write(
        folder.join(format!("{name}-reference.stderr")),
        &output.stderr,
    )?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, expected.as_bytes());
    assert!(output.stderr.is_empty());
    let input = std::fs::read_to_string(input)?;
    let program = if owner == "pa31" {
        provider_traits::lower(&input)?
    } else {
        provider_traits::lower_report(&resolve::resolve(&input, true)?)?
    };
    crate::ffi_program::check(
        root,
        folder,
        &name,
        &program,
        &std::fs::read_to_string(packet.join(observer))?,
        expected,
    )
}
