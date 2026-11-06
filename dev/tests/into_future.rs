//! The actual `IntoFuture` member remains an ordinary owned source conversion.
use crate::resolve as source;
use rustgm::pipeline;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let text = source::fixture(&format!("pa31/tests/spec/into-future-source/{name}.rs"))?;
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = source::source_provider_config(&text, "actual_into_future", &config)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let roots = facts
        .functions
        .iter()
        .filter(|f| {
            matches!(
                report
                    .package
                    .names
                    .get(report.resolution.definitions[f.definition.0 as usize].name),
                "ordinary" | "custom" | "blanket" | "borrowed" | "fwref" | "fwpin"
            )
        })
        .map(|f| f.definition)
        .collect::<Vec<_>>();
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?)
}
#[test]
fn complete_modules_and_original_blanket_reach_real_ordinary_calls() -> Result {
    let mut errors = Vec::new();
    for name in [
        "whole", "blanket", "explicit", "borrow", "alt", "norole", "rename", "native",
    ] {
        match program(name) {
            Ok(_) => eprintln!("{name}: complete source/body/lowering passed"),
            Err(error) => errors.push(format!("{name}: {error}")),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n").into())
    }
}
#[test]
fn diagnostic_companions_keep_real_ordinary_impl_contracts() -> Result {
    for name in ["split", "canon", "nopin", "own", "safe"] {
        program(name)?;
    }
    Ok(())
}
#[test]
fn source_outputs_moves_borrows_and_real_const_unsafe_body_rules_remain() -> Result {
    let mut failures = Vec::new();
    for (name, causes) in [
        (
            "escape",
            &["required lifetime does not outlive its target"][..],
        ),
        ("move", &["moved"][..]),
        ("output", &["associated", "requirement", "equality"][..]),
        (
            "missing",
            &["associated type requires an unambiguous validated implementation"][..],
        ),
        (
            "notfut",
            &["associated type definition does not satisfy its declared bounds"][..],
        ),
        (
            "fake",
            &["associated type requires an unambiguous validated implementation"][..],
        ),
        ("const", &["const"][..]),
        ("unsafe", &["incompatible types"][..]),
        ("body", &["type", "mismatch"][..]),
        ("duplicate", &["duplicate"][..]),
        ("free", &["required declaration target"][..]),
        ("inher", &["required declaration target"][..]),
        ("provided", &["required declaration target"][..]),
    ] {
        match program(name) {
            Ok(_) => failures.push(format!("{name}: invalid source accepted")),
            Err(error) => {
                let text = error.to_string();
                eprintln!("{name}: {text}");
                let expected_site = match name {
                    "unsafe" => Some(13_350),
                    "body" => Some(13_373),
                    _ => None,
                };
                let wrong_site = expected_site.is_some_and(|expected| {
                    error
                        .downcast_ref::<rustgm::resolve::package::Error>()
                        .is_none_or(|actual| {
                            actual.file.source().is_none_or(|file| file.0 != 0)
                                || actual.span.start != expected
                        })
                });
                if wrong_site
                    || !causes.iter().any(|cause| text.contains(cause))
                    || text.contains("not implemented")
                    || text.contains("budget")
                {
                    failures.push(format!("{name}: {text}"));
                }
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n").into())
    }
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn original_and_custom_operand_dependent_conversions_execute_natively() -> Result {
    native_program("native", "observer.c", b"23 23 47 59\n")
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn both_original_future_forwarders_execute_natively() -> Result {
    native_program("fwd", "fwd.c", b"71 83\n")
}
fn native_program(name: &str, observer: &str, expected: &[u8]) -> Result {
    use std::{path::Path, process::Command};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/source-transport/into-future/{name}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let input = folder.join("source.lowir");
    let object = folder.join("source.o");
    let executable = folder.join("observer");
    rustgm::lowir::writer::write(&program(name)?, &mut std::fs::File::create(&input)?)?;
    for (stage, command) in [
        (
            "backend",
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(&input)
                .arg("-o")
                .arg(&object),
        ),
        (
            "link",
            Command::new("cc")
                .arg(
                    root.join("pa31/tests/spec/into-future-source")
                        .join(observer),
                )
                .arg(&object)
                .arg("-o")
                .arg(&executable),
        ),
        ("native", &mut Command::new(&executable)),
    ] {
        std::fs::write(
            folder.join(format!("{stage}.command")),
            format!("{command:?}\n"),
        )?;
        let result = command.output()?;
        std::fs::write(folder.join(format!("{stage}.stdout")), &result.stdout)?;
        std::fs::write(folder.join(format!("{stage}.stderr")), &result.stderr)?;
        if !result.status.success() {
            return Err(format!("{stage}: {}", String::from_utf8_lossy(&result.stderr)).into());
        }
        if stage == "native" {
            assert_eq!(result.stdout, expected);
        }
    }
    Ok(())
}

#[test]
fn uncapped_original_documentation_is_a_separate_rust_positive_required_gap() -> Result {
    let text = source::fixture("pa31/tests/spec/into-future-source/whole.rs")?;
    let report = source::source_provider(&text, "uncapped_into_future")?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("uncapped doc boundary changed; qualify diagnostics")?;
    assert!(
        error
            .to_string()
            .contains("uncapped documentation option diagnostics are not implemented"),
        "{error}"
    );
    Ok(())
}
