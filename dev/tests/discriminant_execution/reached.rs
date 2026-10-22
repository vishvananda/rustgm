//! Reached casts resume exact canonical discriminant dependencies in the query graph.
use super::{checked_sources, native, source};
use std::{
    fmt::Write,
    path::{Path, PathBuf},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ACCEPTED: &[&str] = &[
    "nested-callee",
    "callee-pattern-forward",
    "shared-forward-tag",
    "associated-forward-tag",
    "former-forward-boundary",
    "post-declaration-cast",
    "nested-tag-cfg",
    "nested-tag-pattern",
    "signed-tag",
    "implicit-tag",
    "skipped-cast",
    "skipped-fault",
    "skipped-local-cast",
    "skipped-named-cast",
    "skipped-parameter-cast",
    "skipped-factory-cast",
    "after-return-cast",
    "other-variant-cast",
    "skipped-other-variant-cast",
    "reached-binding",
    "block-cast-cycle",
];
const REJECTED: &[&str] = &[
    "callee-cycle",
    "cross-callee-cycle",
    "skipped-cycle",
    "grouped-cast-cycle",
    "skipped-repr-cycle",
    "direct-root-skipped-cycle",
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/enum-dependency-readiness")
        .join(format!("{name}.rs"))
}
#[test]
fn reached_dependencies_match_independent_objects() -> TestResult {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-enum-dependencies/source")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for &name in ACCEPTED.iter().chain(REJECTED) {
        let status = i32::from(!ACCEPTED.contains(&name));
        checked_sources::check(&fixture(name), &evidence, name, (status, status, "cycle"))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn reached_dependencies_have_native_values() -> TestResult {
    let sources = ACCEPTED
        .iter()
        .filter(|&&name| !["former-forward-boundary", "block-cast-cycle"].contains(&name))
        .map(|&name| Ok((name, std::fs::read_to_string(fixture(name))?)))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let cases = sources
        .iter()
        .map(|(name, text)| {
            Ok((
                *name,
                text.strip_prefix(native::CHECK)
                    .ok_or("missing check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa25-enum-dependencies/native", &cases)
}
#[test]
fn canonical_tags_and_repeated_roots_are_reused() -> TestResult {
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut text = String::from("enum E{");
        for n in 0..width {
            write!(text, "V{n}=f()+{n},")?;
        }
        text.push_str("}const fn f()->isize{F::B as isize}enum F{B=7}");
        for n in 0..width {
            write!(text, "const C{n}:isize=E::V{n} as isize;")?;
        }
        text.push_str("fn main(){}");
        let report = source::resolve(&text, true)?;
        let mut prepared = rustgm::pipeline::analyze_source(&report)?;
        let before = (
            prepared.facts.constant_work.expressions,
            prepared.facts.constant_work.discriminant_visits,
            prepared.facts.body_work.cfg_functions.get(),
        );
        assert_eq!(before.0, width);
        assert_eq!(before.2, 1);
        assert_eq!(
            prepared.facts.aggregates.discriminant_work(),
            (width + 1, width + 1)
        );
        observations.push((width, before.1));
        assert!(
            rustgm::sema::values::Reader::new(&prepared.facts)
                .discriminant(rustgm::sema::members::VariantId(u32::MAX))
                .is_err()
        );
        let definitions = prepared
            .facts
            .initializers
            .iter()
            .map(|i| i.definition)
            .collect::<Vec<_>>();
        {
            let mut evaluator = rustgm::sema::values::Evaluator::new(&report, &mut prepared.facts);
            for _ in 0..3 {
                for &definition in &definitions {
                    evaluator.definition(definition)?;
                }
            }
        }
        assert_eq!(
            before,
            (
                prepared.facts.constant_work.expressions,
                prepared.facts.constant_work.discriminant_visits,
                prepared.facts.body_work.cfg_functions.get(),
            )
        );
    }
    println!("callee discriminant entry visits: {observations:?}");
    assert_eq!(observations[1].1 - observations[0].1, 32 - 8);
    assert_eq!(observations[2].1 - observations[1].1, 128 - 32);
    Ok(())
}
