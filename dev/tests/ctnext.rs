//! Native instances retain original const proof without becoming CTFE executions.
use crate::resolve as source;
use rustgm::pipeline;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    program_in("source-const-cleanup", name)
}
fn program_in(cluster: &str, name: &str) -> Result<rustgm::lowir::program::Program> {
    let text = source::fixture(&format!("pa31/tests/spec/{cluster}/{name}.rs"))?;
    let report = source::source_provider(&text, "source_const_cleanup")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let roots = facts
        .functions
        .iter()
        .filter(|function| {
            let definition = &report.resolution.definitions[function.definition.0 as usize];
            matches!(report.package.names.get(definition.name), "probe" | "run")
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
fn actual_unconditional_const_original_has_a_native_runtime_instance() -> Result {
    program("runct")?;
    Ok(())
}
#[test]
fn native_demand_cannot_hide_an_invalid_const_original() -> Result {
    let error = program("badown").err().ok_or("invalid original accepted")?;
    assert!(error.to_string().contains("const Destruct"), "{error}");
    Ok(())
}
#[test]
fn constant_initializer_cleanup_remains_explicit_required_work() -> Result {
    for name in ["ctgood", "ctbad"] {
        let error = program(name)
            .err()
            .ok_or("interpreter boundary changed; qualify execution")?;
        assert!(
            error
                .to_string()
                .contains("const cleanup selection requires its actual source function"),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn runtime_instance_permission_does_not_pay_a_genuine_const_call_debt() -> Result {
    let error = program_in("source-const-drop", "rtneg")
        .err()
        .ok_or("nonconst Drop paid const call")?;
    assert!(error.to_string().contains("can't drop"), "{error}");
    Ok(())
}
#[test]
#[ignore = "requires locked CPPGM object backend and C compiler"]
fn genuine_unconditional_const_drop_native_body_consumes_runtime_data() -> Result {
    use std::{path::Path, process::Command};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/source-transport/va-cleanup/native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let input = folder.join("source.lowir");
    let object = folder.join("source.o");
    let executable = folder.join("observer");
    rustgm::lowir::writer::write(&program("runct")?, &mut std::fs::File::create(&input)?)?;
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
                .arg(root.join("pa31/tests/spec/source-const-cleanup/runct.c"))
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
        let output = command.output()?;
        std::fs::write(folder.join(format!("{name}.stdout")), &output.stdout)?;
        std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if name == "native" {
            assert_eq!(output.stdout, b"59\n");
        }
    }
    Ok(())
}
