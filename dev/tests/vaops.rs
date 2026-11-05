//! Source vararg roles preserve ordinary headers, ownership and implementation domains.
use crate::resolve as test_source;
use rustgm::{
    lowir::program::Program,
    pipeline,
    resolve::{expansion::Report, model::DefId},
};
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn analyze(name: &str) -> Result<(Report, pipeline::Analysis)> {
    let text = test_source::fixture(&format!("pa31/tests/spec/source-va-list/{name}.rs"))?;
    let report = test_source::source_provider(&text, "source_va_list")?;
    let facts = pipeline::analyze_source(&report)?;
    Ok((report, facts))
}
fn program(name: &str) -> Result<Program> {
    compiled(name).map(|(program, _)| program)
}
fn compiled(name: &str) -> Result<(Program, usize)> {
    let (report, analysis) = analyze(name)?;
    let roots = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .filter(|(_, d)| {
            matches!(
                report.package.names.get(d.name),
                "probe" | "trait_value" | "va_value" | "va_indirect"
            )
        })
        .map(|(index, _)| u32::try_from(index).map(DefId))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    let output = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?;
    Ok((output, facts.body_work.canonical_steps))
}
#[test]
fn vararg_roles_retain_actual_headers_traits_fields_and_ownership() -> Result {
    let rows = test_source::fixture("pa31/tests/spec/source-va-list/cases.tsv")?;
    let mut failures = Vec::new();
    let mut count = 0;
    for line in rows.lines() {
        let mut fields = line.splitn(3, '\t');
        let name = fields.next().ok_or("case")?;
        let accepted = fields.next().ok_or("status")? == "0";
        let cause = fields.next().ok_or("cause")?;
        let result = program(name);
        let message = result.as_ref().err().map(ToString::to_string);
        let matched = if accepted {
            result.is_ok()
        } else {
            message.as_ref().is_some_and(|message| {
                !message.contains("semantic contract is not implemented")
                    && cause.split('|').any(|word| message.contains(word))
            })
        };
        if !matched {
            failures.push(format!(
                "{name}: expected {accepted}, cause {cause}; {}",
                message.unwrap_or_else(|| "accepted".to_owned())
            ));
        }
        count += 1;
    }
    assert_eq!(count, 33);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn vararg_trait_demands_have_bounded_canonical_work() -> Result {
    let (_, initial) = compiled("bound0")?;
    let mut growth = Vec::new();
    for width in [8, 32, 128] {
        let (_, sample) = compiled(&format!("bound{width}"))?;
        growth.push(
            sample
                .checked_sub(initial)
                .ok_or("canonical work decreased")?,
        );
    }
    eprintln!("vararg trait work: base={initial}, growth={growth:?}");
    assert!(growth[0] > 0 && growth[1] >= growth[0]);
    assert!(growth[2] <= growth[1].saturating_mul(5), "{growth:?}");
    Ok(())
}
#[test]
#[ignore = "requires current locked CPPGM object backend and host C compiler"]
fn ordinary_vararg_role_consumers_execute_real_bodies() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    for (name, expected) in [("vnative", "23 17\n"), ("indir", "17 11\n")] {
        let folder = root.join(format!(
            "obj/parallel/source-transport/valist/native-{name}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&folder)?;
        let ir = folder.join("source.lowir");
        rustgm::lowir::writer::write(&program(name)?, &mut std::fs::File::create(&ir)?)?;
        let object = folder.join("source.o");
        checked(
            "backend",
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(&ir)
                .arg("-o")
                .arg(&object),
            &folder,
        )?;
        let executable = folder.join("observer");
        checked(
            "link",
            Command::new("cc")
                .arg(
                    root.join("pa31/tests/spec/source-va-list")
                        .join(format!("{name}.c")),
                )
                .arg(object)
                .arg("-o")
                .arg(&executable),
            &folder,
        )?;
        let actual = checked("native", &mut Command::new(executable), &folder)?;
        std::fs::write(folder.join("native.stdout"), &actual)?;
        assert_eq!(actual, expected.as_bytes());
    }
    Ok(())
}
fn checked(label: &str, command: &mut Command, folder: &Path) -> Result<Vec<u8>> {
    let output = command.output()?;
    std::fs::write(folder.join(format!("{label}.stderr")), &output.stderr)?;
    if !output.status.success() {
        return Err(format!("{label}: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(output.stdout)
}
