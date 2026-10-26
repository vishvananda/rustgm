use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

fn run(mode: &str, source: &str) -> std::io::Result<Output> {
    run_args(mode, source, &[])
}
fn run_args(mode: &str, source: &str, args: &[&str]) -> std::io::Result<Output> {
    run_env(mode, source, args, &[])
}
fn run_env(
    mode: &str,
    source: &str,
    args: &[&str],
    environment: &[(&str, std::ffi::OsString)],
) -> std::io::Result<Output> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .env_clear()
        .envs(environment.iter().map(|(name, value)| (*name, value)))
        .arg(mode)
        .args(args)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("missing stdin"))?
        .write_all(source.as_bytes())?;
    child.wait_with_output()
}

#[test]
fn expanded_cli_preserves_typed_capture_counts_and_deterministic_views() -> std::io::Result<()> {
    let source = "macro_rules! twice {($e:expr)=>{$e+$e}} fn answer()->i32{twice!(21)}";
    let first = run("--emit-expanded-crate", source)?;
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(first.stderr.is_empty());
    let text = String::from_utf8_lossy(&first.stdout);
    assert!(text.starts_with("expanded-crate 1\n"));
    assert!(text.contains(" rule=0 parser-calls=1\n"));
    assert!(text.contains(" macros 1 invocations 1 captures 1 "));
    assert!(!text.contains("pending-expansion="));
    let again = run("--emit-expanded-crate", source)?;
    assert!(again.status.success());
    assert_eq!(first.stdout, again.stdout);
    assert_eq!(first.stderr, again.stderr);
    let count = run("--count-expanded-crate", source)?;
    assert!(count.status.success());
    assert!(count.stderr.is_empty());
    let end = text
        .lines()
        .last()
        .ok_or_else(|| std::io::Error::other("empty view"))?;
    assert_eq!(
        String::from_utf8_lossy(&count.stdout).trim_end(),
        end.strip_prefix("end-expanded-crate ")
            .ok_or_else(|| std::io::Error::other("missing view footer"))?
    );
    Ok(())
}

#[test]
fn expanded_cli_finishes_names_and_emits_no_partial_view_on_source_failure() -> std::io::Result<()>
{
    for source in [
        "macro_rules! m{()=>{missing}} fn f(){m!();}",
        "macro_rules! m{()=>{1 2}} fn f(){m!();}",
        "fn f(){unknown!();}",
        "fn f(){let _=concat!(missing!());}",
        "fn f(){let _=concat!((42));}",
        "compile_error!(concat!(\"stop \", stringify!(now)));",
    ] {
        for mode in ["--emit-expanded-crate", "--count-expanded-crate"] {
            let output = run(mode, source)?;
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert!(output.stdout.is_empty());
            assert!(output.stderr.starts_with(b"ERROR: file "));
        }
    }
    Ok(())
}

#[test]
fn expanded_cli_renders_owned_builtin_values_without_parser_callbacks() -> std::io::Result<()> {
    let result = run(
        "--emit-expanded-crate",
        "fn f(){let _=cfg!(unix);let _=line!();let _=column!();}",
    )?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = String::from_utf8_lossy(&result.stdout);
    assert_eq!(text.matches(" rule=builtin parser-calls=0\n").count(), 3);
    for name in ["cfg", "line", "column"] {
        assert!(text.contains(&format!(" builtin={name}\n")));
    }
    assert!(text.contains(" macros 3 invocations 3 captures 0 "));
    Ok(())
}

#[test]
fn expanded_cli_reports_physical_offsets_and_macro_notes_atomically() -> std::io::Result<()> {
    let result = run(
        "--emit-expanded-crate",
        "macro_rules! inner{()=>{missing}} macro_rules! outer{()=>{inner!()}} fn f(){outer!();}",
    )?;
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        "ERROR: file 0: 24: unresolved value name\n  in macro inner at file 0: 58; defined at file 0: 0\n  in macro outer at file 0: 76; defined at file 0: 34\n"
    );
    Ok(())
}

#[test]
fn expanded_cli_preserves_stdin_name_and_explicit_crate_name() -> std::io::Result<()> {
    let source = "const F:&str=file!();mod child{const M:&str=module_path!();}";
    for (args, expected) in [
        (vec![], "rust_out::child"),
        (vec!["--crate-name", "chosen"], "chosen::child"),
    ] {
        let result = run_args("--emit-expanded-crate", source, &args)?;
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let text = String::from_utf8_lossy(&result.stdout);
        let values: Vec<_> = text
            .lines()
            .filter(|line| line.starts_with("value "))
            .collect();
        assert!(values.iter().any(|line| line.ends_with("\"<anon>\"")));
        assert!(
            values
                .iter()
                .any(|line| line.ends_with(&format!("\"{expected}\"")))
        );
    }
    let conflict = run_args(
        "--emit-expanded-crate",
        "#![crate_name=\"inner\"]const M:&str=module_path!();",
        &["--crate-name", "outer"],
    )?;
    assert_eq!(conflict.status.code(), Some(1));
    assert!(conflict.stdout.is_empty());
    Ok(())
}

#[test]
fn expanded_cli_reads_only_the_childs_explicit_environment() -> std::io::Result<()> {
    let source = "fn f(){let _=env!(concat!(\"RUSTGM_\",\"TEST_ENV\"));}";
    let output = run_env(
        "--emit-expanded-crate",
        source,
        &[],
        &[("RUSTGM_TEST_ENV", "value\r\n".into())],
    )?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"value\\r\\n\""));
    let missing = run("--emit-expanded-crate", source)?;
    assert_eq!(missing.status.code(), Some(1));
    assert!(missing.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("is not defined"));
    Ok(())
}

#[cfg(unix)]
#[test]
fn expanded_cli_diagnoses_non_unicode_environment_values() -> std::io::Result<()> {
    use std::os::unix::ffi::OsStringExt;
    let value = std::ffi::OsString::from_vec(vec![0xff]);
    let output = run_env(
        "--emit-expanded-crate",
        "fn f(){let _=env!(\"RUSTGM_TEST_ENV\");}",
        &[],
        &[("RUSTGM_TEST_ENV", value)],
    )?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("is not Unicode"));
    Ok(())
}

#[test]
fn source_edition_options_retain_configuration_and_keep_metadata_profile_closed()
-> Result<(), Box<dyn std::error::Error>> {
    let keyword = edition_fixture("keyword")?;
    for (args, accepted) in [
        (vec!["--edition=2021"], true),
        (vec![], false),
        (vec!["--edition=2024"], false),
        (vec!["--edition=2024", "--edition=2021"], true),
        (vec!["--edition=2021", "--edition=2024"], false),
    ] {
        let output = run_args("--emit-types", &keyword, &args)?;
        assert_eq!(output.status.code(), Some(i32::from(!accepted)));
        assert_eq!(output.stdout.is_empty(), !accepted);
        assert_eq!(output.stderr.is_empty(), accepted);
    }
    let configured = edition_fixture("configured")?;
    for args in [
        vec!["--cfg", "selected", "--edition=2021"],
        vec!["--edition=2021", "--cfg", "selected"],
    ] {
        let output = run_args("--emit-lowir", &configured, &args)?;
        assert!(output.status.success());
        assert!(!output.stdout.is_empty() && output.stderr.is_empty());
    }
    let invalid = run_args("--emit-types", &keyword, &["--edition=2020"])?;
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    assert_eq!(invalid.stderr, b"USAGE: unknown option: --edition=2020\n");
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/cli-edition-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let metadata = folder.join("forbidden.rgm");
    let path = metadata.to_str().ok_or("metadata path is not UTF-8")?;
    let output = run_args(
        "--emit-lowir",
        &edition_fixture("plain-provider")?,
        &[
            "--edition=2021",
            "--source-core",
            "minicore@driver_edition",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "driver_edition",
            "--crate-type=lib",
            "--metadata-output",
            path,
        ],
    )?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty() && !metadata.exists());
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(
        diagnostic.contains("source export requires defining Rust2024 edition"),
        "{diagnostic}"
    );
    std::fs::remove_dir(&folder)?;
    Ok(())
}

fn edition_fixture(name: &str) -> std::io::Result<String> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa34/tests/spec/driver-edition/{name}.rs")),
    )
}
