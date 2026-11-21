//! The shadow census partitions the queries it saw; nothing is counted unless started.
use super::*;

#[test]
fn counted_queries_partition_into_open_and_keyed_rows() -> Result<(), Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-comparisons/equality/actual-equality.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "query_census_rows")?;
    assert!(rows().is_empty());
    start();
    let counted = crate::pipeline::analyze_source(&report).map(|_| ());
    let rows = rows();
    STATE.with(|state| state.borrow_mut().take());
    counted?;
    assert_eq!(rows.len(), Phase::ALL.len() * Level::ALL.len());
    let mut queries = 0;
    for (_, _, row) in rows {
        assert_eq!(row.queries, row.open + row.independent + row.dependent);
        assert!(row.regional <= row.independent + row.dependent);
        assert!(row.independent_first <= row.independent_keys);
        assert!(row.independent_keys <= row.independent);
        assert!(row.dependent_first <= row.dependent_keys);
        assert!(row.dependent_keys <= row.dependent);
        // Equal premises merge callers; renaming merges equal shapes further.
        assert!(row.structural_first <= row.structural_keys);
        assert!(row.structural_keys <= row.dependent_keys);
        assert!(row.renamed_first <= row.renamed_keys);
        assert!(row.renamed_keys <= row.structural_keys);
        queries += row.queries;
    }
    assert!(queries > 0);
    Ok(())
}
