//! Real selected const Drop bodies execute before their actual storage endpoints.
use crate::resolve as source;
use rustgm::pipeline;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn analyze(name: &str) -> Result<(rustgm::resolve::expansion::Report, pipeline::Analysis)> {
    let text = source::fixture(&format!("pa31/tests/spec/source-ctfe-cleanup/{name}.rs"))?;
    let report = source::source_provider(&text, "ctfe_cleanup_execution")?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn complete(name: &str) -> Result {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(name)?;
    rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    Ok(())
}

#[test]
fn actual_effect_order_move_and_dead_branch_cleanup_values_match_rust() -> Result {
    let mut errors = Vec::new();
    for (name, expected) in [
        ("ct41", 41),
        ("ct59", 59),
        ("ctonce", 41),
        ("ctdead", 0),
        ("ctlifo", 41),
        ("ctmove", 7),
    ] {
        match observed(name) {
            Ok(actual) if actual == expected => eprintln!("{name}: observed {actual}"),
            Ok(actual) => errors.push(format!("{name}: observed {actual}, expected {expected}")),
            Err(error) => errors.push(format!("{name}: {error}")),
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n").into());
    }
    Ok(())
}
fn observed(name: &str) -> Result<u128> {
    let (report, analysis) = analyze(name)?;
    let initializer = analysis
        .facts
        .initializers
        .iter()
        .find(|initializer| {
            report
                .package
                .names
                .get(report.resolution.definitions[initializer.definition.0 as usize].name)
                == "OBSERVED"
        })
        .ok_or("actual OBSERVED initializer")?;
    let reader = rustgm::sema::values::Reader::new(&analysis.facts);
    let value = reader
        .definition(initializer.definition)
        .ok_or("completed OBSERVED value")?;
    let rustgm::sema::values::View::Scalar(rustgm::sema::model::Scalar::Integer(actual)) =
        reader.view(value)?
    else {
        return Err("OBSERVED is not its actual scalar result".into());
    };
    assert!(analysis.facts.constant_work.frames_started >= 2);
    eprintln!(
        "{name}: frames={} steps={} code_bytes={}",
        analysis.facts.constant_work.frames_started,
        analysis.facts.constant_work.steps,
        analysis.facts.constant_work.code_bytes
    );
    Ok(actual)
}
#[test]
fn actual_expired_storage_is_rejected_by_reached_cleanup() -> Result {
    let error = analyze("ctdang")
        .err()
        .ok_or("cleanup wrote expired storage")?;
    assert!(error.to_string().contains("expired storage"), "{error}");
    Ok(())
}
#[test]
fn conditional_flags_follow_the_actual_initialization_events() -> Result {
    let (report, analysis) = analyze("ctcond")?;
    for (name, expected) in [("OFF", 0), ("ON", 41)] {
        let initializer = analysis
            .facts
            .initializers
            .iter()
            .find(|i| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[i.definition.0 as usize].name)
                    == name
            })
            .ok_or("actual conditional initializer")?;
        let reader = rustgm::sema::values::Reader::new(&analysis.facts);
        let value = reader
            .definition(initializer.definition)
            .ok_or("completed conditional result")?;
        assert!(
            matches!(reader.view(value)?,rustgm::sema::values::View::Scalar(
            rustgm::sema::model::Scalar::Integer(actual)) if actual==expected)
        );
    }
    Ok(())
}

#[test]
fn real_nonconst_unsafe_and_opposing_loan_causes_remain_independent() -> Result {
    let mut failures = Vec::new();
    for (name, cause) in [
        ("ctbad", "const"),
        ("ctcall", "const"),
        ("ctunsf", "unsafe"),
        ("dborr", "loan"),
    ] {
        match complete(name) {
            Ok(()) => failures.push(format!("{name}: invalid source accepted")),
            Err(error) => {
                let text = error.to_string();
                eprintln!("{name}: {text}");
                if !text.contains(cause) || text.contains("not implemented") {
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

fn native_program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(name)?;
    let roots = facts
        .functions
        .iter()
        .filter(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "observed"
        })
        .map(|f| f.definition)
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err("actual native observer root".into());
    }
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
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn actual_evaluated_cleanup_effects_reach_five_native_observers() -> Result {
    use std::{path::Path, process::Command};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    for (name, expected) in [
        ("n41", "41\n"),
        ("n59", "59\n"),
        ("nonce", "41\n"),
        ("nlifo", "41\n"),
        ("nmove", "7\n"),
    ] {
        let folder = root.join(format!(
            "obj/parallel/source-transport/ctfe-exec/native-{name}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&folder)?;
        let input = folder.join("source.lowir");
        let object = folder.join("source.o");
        let executable = folder.join("observer");
        rustgm::lowir::writer::write(&native_program(name)?, &mut std::fs::File::create(&input)?)?;
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
                    .arg(root.join("pa31/tests/spec/source-ctfe-cleanup/observe.c"))
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
                return Err(format!(
                    "{name}/{stage}: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
                .into());
            }
            if stage == "native" {
                assert_eq!(result.stdout, expected.as_bytes(), "{name}");
            }
        }
    }
    Ok(())
}
