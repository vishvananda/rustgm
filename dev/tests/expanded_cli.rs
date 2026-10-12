use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

fn run(mode: &str, source: &str) -> std::io::Result<Output> {
    run_args(mode, source, &[])
}
fn run_args(mode: &str, source: &str, args: &[&str]) -> std::io::Result<Output> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
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
