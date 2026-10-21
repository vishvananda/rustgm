//! Nested formation modes and wildcard duties are independently qualified.
use super::{native, rustc, semantic};
use std::fmt::Write;
const CASES: &[(&str, bool)] = &[
    ("move-shared-noncopy", true),
    ("moving-mutable-child", true),
    ("unique-formation", true),
    ("unique-conflict", false),
    ("shared", true),
    ("mutable", true),
    ("move-once", true),
    ("move-copy", true),
    ("precise-field", true),
    ("great-grandchild", true),
    ("local-shadow", true),
    ("local-mutable", true),
    ("empty-child", true),
    ("mutable-formation", true),
    ("immutable-formation-call", false),
    ("borrow-conflict", false),
    ("once-twice", false),
    ("unique-reference", true),
    ("wildcard-uninitialized", false),
    ("wildcard-moved-after", true),
    ("wildcard-moved-inside", false),
    ("wildcard-disjoint", true),
    ("wildcard-live-loan", true),
    ("wildcard-reference-invalid", false),
    ("wildcard-reference-ended", true),
];
fn fixture(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa24/tests/spec/nested-captures/{name}.rs")),
    )
}
#[test]
fn independent_nested_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        rustc::qualify(
            &format!("parallel/pa24/nested-execution/qualification/{name}"),
            &fixture(name)?,
            accepted,
        )?;
    }
    Ok(())
}

#[test]
fn declared_nested_source_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/nested-execution/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, fixture(name)?)?;
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&path)
            .output()?;
        std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
        if output.status.code() != Some(i32::from(!accepted)) {
            failures.push(format!(
                "{name}: {:?}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn scaling_source(kind: &str, width: usize) -> Result<String, std::fmt::Error> {
    if kind == "deep" {
        let mut expression = "x".to_owned();
        for index in 0..width {
            expression = format!("{{let f{index}=||{expression};f{index}()}}");
        }
        return Ok(format!(
            "fn main(){{let x=7u8;let f=||{expression};let _:u8=f();}}"
        ));
    }
    let fields = if kind == "distinct" { width } else { 1 };
    let tuple = "7u8,".repeat(fields);
    let mut body = String::new();
    for index in 0..width {
        let field = if kind == "distinct" { index } else { 0 };
        write!(body, "let v{index}=p.{field};")?;
    }
    Ok(format!(
        "fn main(){{let p=({tuple});let f=||{{let g=||{{{body}0u8}};g()}};let _:u8=f();}}"
    ))
}
#[test]
fn nested_imports_share_deep_and_repeated_prefix_work() -> Result<(), Box<dyn std::error::Error>> {
    for kind in ["deep", "distinct", "repeated"] {
        let mut graph_work = Vec::new();
        let mut canonical = Vec::new();
        let widths = if kind == "deep" {
            [2, 8, 32]
        } else {
            [4, 16, 64]
        };
        for width in widths {
            let source = scaling_source(kind, width)?;
            rustc::qualify(
                &format!("parallel/pa24/nested-execution/scaling/{kind}-{width}"),
                &source,
                true,
            )?;
            let (_, facts) = semantic::analyze(&source)?;
            graph_work.push(
                facts
                    .closures
                    .all()
                    .iter()
                    .filter_map(|body| facts.closures.environment(body.identity))
                    .map(|environment| environment.plan.work)
                    .sum::<usize>(),
            );
            canonical.push(facts.body_work.canonical_steps);
        }
        eprintln!("{kind}: graph={graph_work:?}, canonical={canonical:?}");
        assert_eq!(
            graph_work[2] - graph_work[1],
            4 * (graph_work[1] - graph_work[0]),
            "{kind}"
        );
        assert!(
            canonical[2] - canonical[1] <= 5 * (canonical[1] - canonical[0]),
            "{kind}"
        );
    }
    Ok(())
}
#[test]
fn native_nested_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        if accepted
            && let Err(error) = native::check_cases(
                "parallel/pa24/nested-execution/native",
                &[(name, &fixture(name)?, false)],
            )
        {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
