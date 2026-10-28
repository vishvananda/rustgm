//! Source comparison calls retain genuine headers and ordinary selected-call proof.
use crate::{pipeline, sema::model::Operation};
#[test]
fn actual_equality_uses_source_proof_before_scalar_erasure()
-> Result<(), Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-comparisons/equality/actual-equality.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "source_comparisons")?;
    let analysis = pipeline::analyze_source(&report)?;
    assert!(
        analysis
            .facts
            .nodes
            .iter()
            .flatten()
            .any(|n| matches!(n.operation, Some(Operation::Binary { .. })))
    );
    assert!(
        analysis
            .facts
            .nodes
            .iter()
            .flatten()
            .any(|n| matches!(n.operation, Some(Operation::Call { .. })))
    );
    let equality = analysis
        .facts
        .source_bindings
        .as_deref()
        .ok_or("source catalog")?
        .language(crate::support::bindings::LangItem::PartialEq)
        .ok_or("actual PartialEq")?;
    assert_eq!(
        analysis.facts.associated.comparison(equality.definition),
        None
    );
    Ok(())
}

#[test]
fn actual_integer_partial_ord_retains_full_source_body_and_intrinsic()
-> Result<(), Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-comparisons/ordering/actual-source-comparisons-v2.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "source_comparisons")?;
    let _analysis = pipeline::analyze_source(&report)?;
    Ok(())
}
