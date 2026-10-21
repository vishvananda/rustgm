//! Inherited object inputs constrain inference through ordinary required goals.
use super::{checked_sources, native};
use std::{fmt::Write, path::PathBuf};

const POSITIVES: &[&str] = &[
    "inherited",
    "annotated",
    "mutable",
    "once",
    "mixed",
    "deep",
    "diamond",
    "unused-binder",
    "quantified-input",
    "quantified-output",
    "unresolved-target",
];
const NEGATIVES: &[(&str, i32, &str)] = &[
    (
        "wrong-annotation",
        1,
        "trait obligation requires an unambiguous",
    ),
    (
        "wrong-capture",
        1,
        "trait obligation requires an unambiguous",
    ),
    (
        "wrong-output",
        1,
        "trait obligation requires an unambiguous",
    ),
    (
        "principal-missing",
        1,
        "trait obligation requires an unambiguous",
    ),
    (
        "quantified-static-input",
        0,
        "required lifetime does not outlive its target",
    ),
    (
        "output-escape",
        0,
        "required lifetime does not outlive its target",
    ),
];
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/object-input-context/{name}.rs"))
}
fn evidence(kind: &str) -> Result<PathBuf, std::io::Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24-object-inputs/{kind}/{}",
        std::process::id(),
    ));
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
#[test]
fn inherited_requirements_preserve_independent_object_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = evidence("source")?;
    for &name in POSITIVES {
        checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    for &(name, types, reason) in NEGATIVES {
        checked_sources::check_modes(&fixture(name), &evidence, name, (1, [types, 1, 1], reason))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn inherited_numeric_callables_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &name in POSITIVES {
        let source = std::fs::read_to_string(fixture(name))?;
        native::check_cases(
            "parallel/pa24-object-inputs/native",
            &[(
                name,
                source
                    .strip_prefix(native::CHECK)
                    .ok_or("missing check prefix")?,
                false,
            )],
        )?;
    }
    Ok(())
}
#[test]
fn repeated_and_distinct_object_targets_bound_inference_work()
-> Result<(), Box<dyn std::error::Error>> {
    for distinct in [false, true] {
        let mut observations = Vec::new();
        for width in [8, 32, 128] {
            let mut source =
                "trait Apply<T>:Fn(T)->T{}impl<T,F:Fn(T)->T>Apply<T> for F{}fn main(){".to_owned();
            for index in 0..width {
                let count = if distinct { index + 1 } else { 1 };
                write!(
                    source,
                    "let n{index}=[7u8;{count}];let f{index}=|x|{{let _:[u8;{count}]=x;n{index}}};let _:&dyn Apply<[u8;{count}]>=&f{index};"
                )?;
            }
            source.push('}');
            let (_, facts) = super::semantic::analyze(&source)?;
            let work = facts.body_work.inference_steps
                + facts.body_work.canonical_steps
                + facts.body_work.projection_steps;
            observations.push(work);
            eprintln!(
                "object inputs distinct={distinct} width={width} inference_canonical_projection_steps={work}"
            );
        }
        assert!(
            observations[2] - observations[1] <= 5 * (observations[1] - observations[0]),
            "distinct={distinct}: {observations:?}"
        );
    }
    Ok(())
}
