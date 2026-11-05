//! Actual compound language contracts keep mutable places, effects and call proof.
use crate::resolve as test_source;
use rustgm::{
    lowir::program::Program,
    pipeline,
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
    },
};
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn analyze(name: &str) -> Result<(Report, pipeline::Analysis)> {
    let text = test_source::fixture(&format!("pa31/tests/spec/source-assign-ten/{name}.rs"))?;
    let report = test_source::source_provider(&text, "assignment_ten")?;
    let facts = pipeline::analyze_source(&report)?;
    Ok((report, facts))
}
fn lower(report: &Report, analysis: pipeline::Analysis) -> Result<Program> {
    let mut roots = Vec::new();
    for (index, d) in report.resolution.definitions.iter().enumerate() {
        let name = report.package.names.get(d.name);
        if d.kind == DefKind::Function
            && (matches!(
                name,
                "probe"
                    | "use_op"
                    | "calc"
                    | "primitive"
                    | "cross"
                    | "builtin"
                    | "explicit"
                    | "constant"
                    | "packed"
                    | "order_value"
                    | "order_state"
            ) || ["custom_", "default_", "primitive_", "boolean_", "bound_"]
                .iter()
                .any(|p| name.starts_with(p)))
        {
            roots.push(DefId(u32::try_from(index)?));
        }
    }
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    Ok(rustgm::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?)
}
fn program(name: &str) -> Result<Program> {
    let (report, facts) = analyze(name)?;
    lower(&report, facts)
}
fn cause_matches(kind: &str, error: &str) -> bool {
    let message = error.to_lowercase();
    let words: &[&str] = match kind {
        "role" => &[
            "requires its actual trait",
            "requires its actual language declaration",
        ],
        "rhs" | "implementation" => &["trait", "implementation", "incompatible"],
        "output" | "receiver" => &["signature", "method", "incompatible", "unit"],
        "unsafe" => &["safe", "unsafe"],
        "mutable" => &["mutab", "immutable"],
        "alias" => &["borrow", "loan", "conflict"],
        "place" => &["assignment requires"],
        "const" => &[
            "const",
            "trait requirement",
            "trait bound",
            "trait obligation",
            "no implementation for",
            "cannot add-assign",
        ],
        "alignment" => &["align", "packed"],
        "scalar" => &["scalar", "incompatible"],
        _ => &[],
    };
    !message.contains("semantic contract is not implemented")
        && !message.contains("expected a token")
        && words.iter().any(|w| message.contains(w))
}
#[test]
fn all_ten_assignments_keep_declared_semantic_causes_and_normal_borrow_proof() -> Result {
    let rows = test_source::fixture("pa31/tests/spec/source-assign-ten/cases.tsv")?;
    let mut failures = Vec::new();
    let mut count = 0;
    for line in rows.lines() {
        let mut fields = line.splitn(4, '\t');
        let file = fields.next().ok_or("case")?;
        let expected = fields.next().ok_or("status")?;
        let cause = fields.next().ok_or("cause")?;
        let detail = fields.next().ok_or("detail")?;
        let name = file.strip_suffix(".rs").ok_or("suffix")?;
        let result = program(name);
        match (expected, result) {
            ("0", Ok(_)) => {}
            ("1", Err(error)) if cause_matches(cause, &error.to_string()) => {}
            (_, result) => failures.push(format!(
                "{file}: {cause} ({detail}); {}",
                result
                    .err()
                    .map_or_else(|| "accepted".to_owned(), |e| e.to_string())
            )),
        }
        count += 1;
    }
    assert_eq!(count, 138);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn actual_mutable_operator_demand_has_bounded_canonical_work() -> Result {
    let (_, base) = analyze("runtime")?;
    let initial = base.facts.body_work.canonical_steps;
    let mut growth = Vec::new();
    for n in [8, 32, 128] {
        let (_, sample) = analyze(&format!("bound-{n}"))?;
        growth.push(
            sample
                .facts
                .body_work
                .canonical_steps
                .checked_sub(initial)
                .ok_or("work decreased")?,
        );
    }
    eprintln!("source assignment work: base={initial}, growth={growth:?}");
    assert!(growth[0] > 0 && growth[1] >= growth[0]);
    assert!(growth[2] <= growth[1].saturating_mul(5), "{growth:?}");
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
#[test]
#[ignore = "requires current locked CPPGM object backend and host C compiler"]
fn real_custom_default_const_packed_and_order_effects_execute_natively() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let cases = [
        (
            "runtime",
            "observer.c",
            format!("{}\n", ["23"; 20].join(" ")),
        ),
        ("primitive-body", "primitive-observer.c", "11 3\n".into()),
        ("const-primitive-valid", "constant-observer.c", "6\n".into()),
        ("const-explicit-valid", "constant-observer.c", "4\n".into()),
        ("const-custom", "constant-observer.c", "11\n".into()),
        ("packed-primitive", "packed-observer.c", "11\n".into()),
        ("order-custom", "order-observer.c", "1 2\n".into()),
        ("provided-effect", "constant-observer.c", "23\n".into()),
        ("provided-copy", "constant-observer.c", "23\n".into()),
    ];
    for (name, observer, expected) in cases {
        let folder = root.join(format!(
            "obj/parallel/source-transport/assignments/native-{name}-{}",
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
                    root.join("pa31/tests/spec/source-assign-ten")
                        .join(observer),
                )
                .arg(object)
                .arg("-o")
                .arg(&executable),
            &folder,
        )?;
        let actual = checked("native", &mut Command::new(executable), &folder)?;
        std::fs::write(folder.join("native.stdout"), &actual)?;
        assert_eq!(actual, expected.as_bytes(), "{name}");
    }
    Ok(())
}
