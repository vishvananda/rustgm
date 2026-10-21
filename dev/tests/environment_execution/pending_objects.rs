//! A declared object principal supplies goals before its caller arguments are solved.
use super::{checked_sources, native};
use std::{fmt::Write, path::PathBuf};
const POSITIVES: &[&str] = &[
    "early-type",
    "mutable-input",
    "mixed-output",
    "mixed-constant",
    "unused-public-binder",
    "quantified-reference",
    "early-lifetime-result",
];
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../pa24/tests/spec/pending-object-inputs/{name}.rs"
    ))
}
#[test]
fn pending_principal_goals_preserve_independent_source_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24-pending-object-inputs/source/{}",
        std::process::id(),
    ));
    std::fs::create_dir_all(&evidence)?;
    for &name in POSITIVES {
        checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    for name in ["wrong-written-input", "wrong-array-length"] {
        checked_sources::check(
            &fixture(name),
            &evidence,
            name,
            (1, 1, "trait obligation requires an unambiguous"),
        )?;
    }
    checked_sources::check_modes(
        &fixture("escaping-result"),
        &evidence,
        "escaping-result",
        (1, [0, 1, 1], "access conflicts with a live reference loan"),
    )?;
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn pending_principal_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &name in POSITIVES {
        let source = std::fs::read_to_string(fixture(name))?;
        native::check_cases(
            "parallel/pa24-pending-object-inputs/native",
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
fn repeated_and_distinct_pending_arguments_bound_inference_work()
-> Result<(), Box<dyn std::error::Error>> {
    for distinct in [false, true] {
        let mut observations = Vec::new();
        for width in [8, 32, 128] {
            let mut source = "trait Apply<T>:Fn(T)->T{}impl<T,F:Fn(T)->T>Apply<T> for F{}fn run<T:Copy>(f:&dyn Apply<T>,x:T)->T{f(x)}fn main(){".to_owned();
            for index in 0..width {
                if distinct {
                    write!(
                        source,
                        "let n{index}=[7u8;{}];let f{index}=|x|{{let _=x;n{index}}};let _=run(&f{index},[0u8;{}]);",
                        index + 1,
                        index + 1
                    )?;
                } else {
                    write!(
                        source,
                        "let n{index}=4;let f{index}=|x|x+n{index};let _=run(&f{index},3u16);"
                    )?;
                }
            }
            source.push('}');
            let (_, facts) = super::semantic::analyze(&source)?;
            let work = facts.body_work.inference_steps
                + facts.body_work.canonical_steps
                + facts.body_work.projection_steps;
            observations.push(work);
            eprintln!("pending inputs distinct={distinct} width={width} work={work}");
        }
        assert!(
            observations[2] - observations[1] <= 5 * (observations[1] - observations[0]),
            "distinct={distinct}: {observations:?}"
        );
    }
    Ok(())
}
