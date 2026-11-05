//! Const destruction proves actual nominal fields and real const Drop owners.
use crate::resolve as source;
use rustgm::{pipeline, resolve::expansion::Report};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn analyze(name: &str) -> Result<(Report, pipeline::Analysis)> {
    let text = source::fixture(&format!("pa31/tests/spec/source-const-drop/{name}.rs"))?;
    let report = source::source_provider(&text, "source_const_drop")?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
#[test]
fn genuine_const_destruct_proves_fields_and_selected_const_drop() -> Result {
    for name in [
        "plain2", "ownc2", "field2", "manual", "union", "tuplec", "arrayc",
    ] {
        analyze(name).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
#[test]
fn nonconst_drop_in_self_or_semantic_generic_field_is_rejected() {
    for name in ["ownn2", "flneg2", "tuplen", "arrayn", "array0"] {
        let error = analyze(name).err().map(|e| e.to_string());
        assert!(
            error
                .as_deref()
                .is_some_and(|message| message.contains("can't drop")),
            "{name}: {error:?}"
        );
    }
}

fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    compiled(name).map(|(program, _)| program)
}
fn compiled(name: &str) -> Result<(rustgm::lowir::program::Program, usize)> {
    let (report, analysis) = analyze(name)?;
    let roots = analysis
        .facts
        .functions
        .iter()
        .filter(|function| {
            let definition = &report.resolution.definitions[function.definition.0 as usize];
            matches!(
                report.package.names.get(definition.name),
                "probe" | "run" | "runtime"
            )
        })
        .map(|f| f.definition)
        .collect::<Vec<_>>();
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?;
    Ok((program, facts.body_work.canonical_steps))
}
#[test]
fn initialized_const_drop_and_inactive_moved_borrowed_originals_compile() -> Result {
    for name in [
        "initc", "inact", "moved", "borrow", "hold", "count2", "rtinst", "rtcnt",
    ] {
        program(name).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn actual_nonconst_owned_cleanup_is_a_semantic_rejection() -> Result {
    for name in ["initn", "part"] {
        let error = program(name)
            .err()
            .ok_or("active nonconst Drop accepted")?
            .to_string();
        assert!(
            error.contains("const Destruct") && !error.contains("not implemented"),
            "{error}"
        );
    }
    Ok(())
}
#[test]
fn conditional_const_drop_impl_preserves_actual_declared_premise() -> Result {
    for name in ["cond3", "clone2", "foo2", "fake"] {
        analyze(name).map_err(|e| format!("{name}: {e}"))?;
    }
    for name in ["strng2", "fstrg2", "unco2", "othr"] {
        let error = analyze(name)
            .err()
            .ok_or_else(|| format!("{name}: stronger Drop requirement accepted"))?;
        assert!(
            error.to_string().contains("type requirements"),
            "{name}: {error}"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn actual_const_drop_runtime_body_writes_the_runtime_operand() -> Result {
    use std::{path::Path, process::Command};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    for (name, observer, expected) in [("count2", "count.c", "41\n"), ("rtcnt", "rtcnt.c", "47\n")]
    {
        let folder = root.join(format!(
            "obj/parallel/source-transport/va-const/native-{name}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&folder)?;
        let input = folder.join("source.lowir");
        let object = folder.join("source.o");
        let executable = folder.join("observer");
        rustgm::lowir::writer::write(&program(name)?, &mut std::fs::File::create(&input)?)?;
        for (name, command) in [
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
                        root.join("pa31/tests/spec/source-const-drop")
                            .join(observer),
                    )
                    .arg(&object)
                    .arg("-o")
                    .arg(&executable),
            ),
            ("native", &mut Command::new(&executable)),
        ] {
            std::fs::write(
                folder.join(format!("{name}.command")),
                format!("{command:?}\n"),
            )?;
            let result = command.output()?;
            std::fs::write(folder.join(format!("{name}.stdout")), &result.stdout)?;
            std::fs::write(folder.join(format!("{name}.stderr")), &result.stderr)?;
            assert!(
                result.status.success(),
                "{name}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            if name == "native" {
                assert_eq!(result.stdout, expected.as_bytes());
            }
        }
    }
    Ok(())
}

#[test]
fn actual_definition_cleanup_work_grows_with_demanded_bodies() -> Result {
    let (_, baseline) = compiled("work0")?;
    let mut growth = Vec::new();
    for width in [8, 32, 128] {
        let (_, work) = compiled(&format!("work{width}"))?;
        growth.push(
            work.checked_sub(baseline)
                .ok_or("canonical work decreased")?,
        );
    }
    eprintln!("const cleanup canonical work: base={baseline}, growth={growth:?}");
    assert!(growth[0] > 0 && growth[1] >= growth[0]);
    assert!(growth[2] <= growth[1].saturating_mul(5), "{growth:?}");
    Ok(())
}

#[test]
fn selected_enum_and_runtime_instance_requirements_remain_named() -> Result {
    for (name, gap) in [
        ("egood", "selected const enum cleanup leaves"),
        ("empty", "selected const enum cleanup leaves"),
        ("rtonce", "constant evaluation cannot run destructors"),
    ] {
        let error = program(name)
            .err()
            .ok_or_else(|| format!("{name}: boundary changed; qualify it"))?;
        assert!(error.to_string().contains(gap), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn ordinary_copy_does_not_forge_a_const_destruct_call_premise() -> Result {
    let error = analyze("cpbnd")
        .err()
        .ok_or("Copy silently supplied const Destruct")?;
    assert!(error.to_string().contains("can't drop"), "{error}");
    Ok(())
}
#[test]
fn original_unit_result_const_payment_is_still_explicit_required_work() -> Result {
    for name in ["plain", "ownc", "fieldc", "cond2"] {
        let error = analyze(name)
            .err()
            .ok_or("unit payment boundary changed; qualify it")?;
        assert!(
            error.to_string().contains("primitive result"),
            "{name}: {error}"
        );
    }
    Ok(())
}
