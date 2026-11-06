//! Initialized guards execute actual plan events in their owning const frame.
use crate::resolve as source;
use rustgm::{
    pipeline,
    sema::model::Scalar,
    sema::values::{Reader, View},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn analyze(name: &str) -> Result<(rustgm::resolve::expansion::Report, pipeline::Analysis)> {
    let text = source::fixture(&format!("pa31/tests/spec/source-ctfe-flags/{name}.rs"))?;
    let report = source::source_provider(&text, "ctfe_conditional_flags")?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn observed(name: &str) -> Result<(u128, usize, usize)> {
    let (report, analysis) = analyze(name)?;
    let initializer = analysis
        .facts
        .initializers
        .iter()
        .find(|i| {
            report
                .package
                .names
                .get(report.resolution.definitions[i.definition.0 as usize].name)
                == "FLAGGED"
        })
        .ok_or("actual FLAGGED initializer")?;
    let reader = Reader::new(&analysis.facts);
    let value = reader
        .definition(initializer.definition)
        .ok_or("completed FLAGGED value")?;
    let View::Scalar(Scalar::Integer(value)) = reader.view(value)? else {
        return Err("FLAGGED result type".into());
    };
    Ok((
        value,
        analysis.facts.constant_work.steps,
        analysis.facts.constant_work.peak_slots,
    ))
}
#[test]
fn initialized_move_branch_loop_and_nested_frames_match_independent_rust() -> Result {
    let mut errors = Vec::new();
    for (name, expected) in [
        ("skip", 0),
        ("init", 41),
        ("both", 41),
        ("repl", 59),
        ("lifo", 41),
        ("move", 41),
        ("loop", 41),
        ("child", 41),
        ("chskip", 41),
        ("early", 59),
        ("parin", 59),
        ("parmov", 59),
    ] {
        match observed(name) {
            Ok((value, steps, slots)) => {
                eprintln!(
                    "{name}: observed={value} expected={expected} steps={steps} slots={slots}"
                );
                if value != expected {
                    errors.push(format!("{name}: {value} != {expected}"));
                }
            }
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
fn flags_cross_words_and_pay_bounded_work() -> Result {
    let mut costs = Vec::new();
    for width in [8, 32, 65, 128] {
        let (value, steps, slots) = observed(&format!("wide{width}"))?;
        assert_eq!(value, 41);
        costs.push((steps, slots));
        eprintln!("flag width={width}: steps={steps} slots={slots}");
    }
    assert!(costs[3].0 <= costs[1].0.saturating_mul(5));
    assert!(costs[3].1 <= costs[1].1.saturating_mul(5));
    Ok(())
}
#[test]
fn real_uninitialized_move_loan_const_unsafe_and_expired_causes_remain() -> Result {
    let mut errors = Vec::new();
    for (name, cause) in [
        ("uninit", "uninitialized"),
        ("double", "moved"),
        ("loan", "loan"),
        ("noncon", "const"),
        ("unsafe", "unsafe"),
        ("dang", "expired storage"),
    ] {
        let result = (|| -> Result {
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
        })();
        match result {
            Ok(()) => errors.push(format!("{name}: invalid source accepted")),
            Err(error) => {
                let text = error.to_string();
                eprintln!("{name}: {text}");
                if !text.contains(cause) || text.contains("not implemented") {
                    errors.push(format!("{name}: {text}"));
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n").into())
    }
}

fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
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
        .filter(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                .starts_with("observed")
        })
        .map(|f| f.definition)
        .collect::<Vec<_>>();
    if roots.is_empty() {
        return Err("actual flag observer exports".into());
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
fn actual_conditional_effects_reach_eight_native_observers() -> Result {
    use std::{path::Path, process::Command};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    for (name, expected) in [
        ("onoff", "0 41\n"),
        ("move", "41\n"),
        ("loop", "41\n"),
        ("child", "41\n"),
        ("early", "59\n"),
        ("wide65", "41\n"),
        ("parin", "59\n"),
        ("parmov", "59\n"),
    ] {
        let folder = root.join(format!(
            "obj/parallel/source-transport/ctfe-flags/native-{name}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&folder)?;
        let input = folder.join("source.lowir");
        let object = folder.join("source.o");
        let executable = folder.join("observer");
        rustgm::lowir::writer::write(&program(name)?, &mut std::fs::File::create(&input)?)?;
        let observer = if name == "onoff" {
            "onoff.c"
        } else {
            "observe.c"
        };
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
                        root.join("pa31/tests/spec/source-ctfe-flags")
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
