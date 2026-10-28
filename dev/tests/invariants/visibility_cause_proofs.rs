//! Source authentication, charged failed reads and cold layout costs stay explicit.
use super::*;
use crate::resolve::{package::ErrorData, unit::FileId};
#[test]
fn visibility_source_failures_keep_primary_errors_and_policy_costs_are_measured() -> TestResult {
    let source = fixture(25, "visibility-warn")?;
    let mut found = report(&source)?;
    let key = found
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.visibilities.as_deref())
        .and_then(|causes| causes.records.first())
        .ok_or("actual cause")?
        .key;
    let first = found.unused_visibility_warnings()?;
    assert_eq!(first.len(), 1);
    let prior = found.lint_producer().ok_or("source producer")?.work().steps;
    found.package.units[0].source = SourceFile::new(source.as_bytes().to_vec())?;
    assert!(found.unused_visibility_warnings().is_err());
    assert!(found.lint_producer().ok_or("source producer")?.work().steps > prior);
    let primary = found.retain_visibility_warnings(Error::at(FileId(0), 31, "primary failure"));
    assert_eq!(primary.message, "primary failure");
    assert_eq!(primary.span.start, 31);
    assert!(primary.warnings().is_empty());
    assert_eq!(
        primary.to_string(),
        "file 0: 31: primary failure\n  note: retained warning source could not be authenticated; further warnings unavailable"
    );
    let budget = report(&source)?;
    assert!(
        budget
            .lint_producer()
            .ok_or("source producer")?
            .diagnostic_read(key, usize::MAX)
            .is_err()
    );
    let primary = budget.retain_visibility_warnings(Error::at(FileId(0), 31, "primary failure"));
    assert_eq!(
        primary.to_string(),
        "file 0: 31: primary failure\n  note: retained warning work or storage budget exceeded; further warnings unavailable"
    );
    let ordinary = report("const ORDINARY:u8=7;fn main(){}")?;
    assert!(ordinary.attribute_effects.is_none());
    let ordinary = pipeline::analyze_source(&ordinary)?;
    assert_eq!(ordinary.facts.retained_policy_storage(), (0, 0, 0));
    let affected = report("#![allow(unused_visibilities)]const ORDINARY:u8=7;fn main(){}")?;
    let affected = pipeline::analyze_source(&affected)?;
    let (nodes, capacity, outer) = affected.facts.retained_policy_storage();
    assert_eq!(nodes, affected.facts.safety_work().retained_nodes);
    assert!(nodes > 0 && capacity >= nodes);
    assert_eq!(size_of::<crate::resolve::lint_policy::Level>(), 1);
    let (rows, row_capacity, row_size) = affected.facts.retained_policy_rows();
    println!(
        "lint policy nodes={nodes} capacity={capacity} dense_bytes={} cold_rows={rows} cold_bytes={} prior_dense_bytes={} outer_bytes={outer}",
        capacity * size_of::<u32>(),
        row_capacity * row_size,
        capacity * 12
    );
    println!(
        "unused visibility cold Effects={} Causes={} Record={} ErrorData={} RetentionNote={} Report={} Engine={} Node={} NodeFacts={} Scope={} ResolutionDefinition={} MacroBody={} MacroDefinition={}",
        size_of::<Effects>(),
        size_of::<Causes>(),
        size_of::<Record>(),
        size_of::<ErrorData>(),
        size_of::<RetentionNote>(),
        size_of::<Report>(),
        size_of::<Engine<'_, NoFiles>>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::sema::model::NodeFacts>(),
        size_of::<crate::resolve::model::Scope>(),
        size_of::<crate::resolve::model::Definition>(),
        size_of::<crate::resolve::expansion::Body>(),
        size_of::<crate::resolve::expansion::Definition>()
    );
    assert_eq!(size_of::<RetentionNote>(), size_of::<bool>());
    assert_eq!(size_of::<Error>(), size_of::<usize>());
    assert_eq!(size_of::<crate::resolve::expansion::Body>(), 96);
    assert_eq!(size_of::<crate::resolve::expansion::Definition>(), 136);
    Ok(())
}
