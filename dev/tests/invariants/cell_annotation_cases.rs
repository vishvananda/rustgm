//! Genuine pinned declaration fixtures keep ordinary language permissions separate.
use super::*;
use crate::{pipeline, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
#[path = "confusable_sources.rs"]
mod confusables;
#[path = "touched_markers.rs"]
mod frontier;
#[path = "suspension_sources.rs"]
mod suspension;
fn input(cluster: &str, name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/{cluster}/{name}.rs"
    ))?)
}
fn checked(cluster: &str, name: &str) -> TestResult<(Report, pipeline::Analysis)> {
    let report = test_source::resolve(&input(cluster, name)?, true)?;
    let analysis = pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
    Ok((report, analysis))
}
fn lower((report, analysis): (Report, pipeline::Analysis)) -> TestResult {
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    Ok(())
}
fn assert_v(report: &Report, facts: &Declarations) -> TestResult {
    let initializer = facts
        .initializers
        .iter()
        .find(|row| {
            report
                .package
                .names
                .get(report.resolution.definitions[row.definition.0 as usize].name)
                == "V"
        })
        .ok_or("actual V initializer")?;
    let reader = crate::sema::values::Reader::new(facts);
    assert!(matches!(
        reader.view(
            reader
                .definition(initializer.definition)
                .ok_or("completed V")?
        )?,
        crate::sema::values::View::Scalar(crate::sema::model::Scalar::Integer(7))
    ));
    Ok(())
}
