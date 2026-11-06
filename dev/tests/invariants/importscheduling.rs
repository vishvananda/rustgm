//! Imported declarations are terminal scheduling rows, never checked source bodies.
use super::{Status, imported, initial_states};

#[test]
fn opaque_aware_import_schedule_requires_the_real_external_ledger()
-> Result<(), Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture("pa31/tests/spec/source-field-captures/native.rs")?;
    let report = crate::test_source::source_provider(&source, "import_schedule")?;
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    let (index, function) = facts
        .functions
        .iter()
        .copied()
        .enumerate()
        .find(|(_, function)| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "record"
        })
        .ok_or("actual import declaration")?;
    assert!(imported(&facts, function));
    assert!(!facts.typed_function_bodies.contains(&function.definition));
    assert!(matches!(initial_states(&facts)[index], Status::Checked));
    facts.foreign = None;
    assert!(!imported(&facts, function));
    assert!(matches!(initial_states(&facts)[index], Status::Unseen));
    Ok(())
}
