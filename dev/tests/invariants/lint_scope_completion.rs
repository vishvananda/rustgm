//! Lexical expectations stay pending until their actual nested bodies complete.
use crate::{
    pipeline,
    resolve::{expansion::Report, model::DefId},
    support::diagnostics::WarningCode,
};
use std::fmt::Write;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn definition(report: &Report, name: &str) -> Result<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == name)
        .ok_or("missing actual function declaration")?;
    Ok(DefId(u32::try_from(index)?))
}

#[test]
fn unfinished_nested_function_cannot_finalize_its_parents_expectation() -> Result {
    for (name, expected) in [
        ("selected-nested-fulfilled", vec![]),
        (
            "selected-nested-unfulfilled",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
    ] {
        let source =
            crate::test_source::fixture(&format!("pa40/tests/spec/lint-expectations/{name}.rs"))?;
        let report = crate::test_source::resolve(&source, true)?;
        let parent = definition(&report, "checked")?;
        let child = definition(&report, "nested")?;
        let (facts, _) = pipeline::prepare_local_runtime(&report, &[parent])?;
        assert!(facts.typed_function_bodies.contains(&parent));
        assert!(!facts.typed_function_bodies.contains(&child));
        assert!(
            facts.warnings().next().is_none(),
            "{name}: premature completion"
        );
        let (facts, _) = pipeline::prepare_local_runtime(&report, &[parent, child])?;
        assert!(facts.typed_function_bodies.contains(&child));
        assert_eq!(
            facts
                .warnings()
                .map(|warning| warning.code)
                .collect::<Vec<_>>(),
            expected,
            "{name}: actual descendant completion"
        );
        let analysis = pipeline::analyze_source(&report)?;
        assert_eq!(
            analysis
                .facts
                .warnings()
                .map(|warning| warning.code)
                .collect::<Vec<_>>(),
            expected,
            "{name}: full source"
        );
    }
    Ok(())
}

#[test]
fn descendant_completion_is_one_paid_census_across_expectations_and_unrelated_bodies() -> Result {
    for (parents, noise) in [(1, 0), (8, 0), (32, 0), (128, 0), (1, 128), (1, 512)] {
        let mut source = String::new();
        for index in 0..parents {
            writeln!(
                source,
                "#[expect(deprecated)]fn parent{index}(){{fn child{index}(){{}}}}"
            )?;
        }
        for index in 0..noise {
            writeln!(source, "fn noise{index}(){{}}")?;
        }
        let report = crate::test_source::resolve(&source, true)?;
        let roots = (0..parents)
            .map(|index| definition(&report, &format!("parent{index}")))
            .collect::<Result<Vec<_>>>()?;
        let (mut facts, _) = pipeline::prepare_local_runtime(&report, &roots)?;
        assert_eq!(facts.typed_function_bodies.len(), parents);
        let retained = facts.retained_policy_rows();
        let producer = report.lint_producer().ok_or("actual source producer")?;
        let mut observations = Vec::new();
        for _ in 0..2 {
            let before = producer.work().steps;
            facts.finalize_lints(&report, true)?;
            let paid = producer.work().steps - before;
            let declarations = report.resolution.definitions.len();
            // The u16 member set bounds each body query; source-owner work is
            // separate from the one shared declaration census.
            let member_bound = 2 * usize::try_from(u16::BITS)? + 2;
            let bound = member_bound * declarations + 32 * parents + 128;
            assert!(
                paid <= bound,
                "parents={parents} noise={noise} declarations={declarations} paid={paid} bound={bound}"
            );
            assert!(facts.warnings().next().is_none());
            assert_eq!(facts.typed_function_bodies.len(), parents);
            assert_eq!(facts.retained_policy_rows(), retained);
            observations.push(paid);
        }
        assert_eq!(
            observations[0], observations[1],
            "no result or authority cache"
        );
        println!(
            "lint_nested parents={parents} unrelated_bodies={noise} declarations={} paid_steps={}",
            report.resolution.definitions.len(),
            observations[0]
        );
    }
    Ok(())
}

#[test]
fn failed_completion_payment_retains_attempted_work_without_finalization() -> Result {
    let source = crate::test_source::fixture(
        "pa40/tests/spec/lint-expectations/selected-nested-unfulfilled.rs",
    )?;
    let report = crate::test_source::resolve(&source, true)?;
    let parent = definition(&report, "checked")?;
    let (mut facts, _) = pipeline::prepare_local_runtime(&report, &[parent])?;
    let producer = report.lint_producer().ok_or("actual source producer")?;
    let key = report.resolution.definitions[parent.0 as usize]
        .node
        .ok_or("actual parent source")?;
    producer
        .diagnostic_payload(key, 10_000_000 - producer.work().steps, 0)
        .map_err(crate::resolve::package::Error::from)?;
    let before = producer.work().steps;
    let error = facts
        .finalize_lints(&report, true)
        .err()
        .ok_or("exhausted completion budget accepted")?;
    assert!(error.to_string().contains("budget exceeded"), "{error}");
    assert!(producer.work().steps > before);
    assert!(facts.warnings().next().is_none());
    assert_eq!(facts.typed_function_bodies.len(), 1);
    Ok(())
}
