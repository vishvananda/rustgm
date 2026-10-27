//! Structural count preparation pays before its one index allocation.
use super::*;
use crate::expand::{config::Config, macros::session};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;
#[test]
fn indexed_counts_prepay_rejection_and_reuse_only_the_same_binding_structure() -> Result {
    let source = "macro_rules!m{($( [ $($x:ident),* ] ),*)=>{${count($x)}}}m!([a,b],[c]);";
    let report =
        session::analyze(source, &Config::default()).map_err(|error| format!("{error:?}"))?;
    let call = &report.invocations[0];
    let definition = &report.definitions[call.definition].compiled.definition;
    let bindings = Bindings::from_match(&call.matched).map_err(|error| format!("{error:?}"))?;
    let query = Count {
        definition,
        rule: &definition.rules[call.matched.rule],
        variable: VariableId(0),
        path: &[],
        depth: 0,
    };
    let mut rejected = Cardinalities::default();
    let mut work = 999_990;
    let error = rejected
        .count(&bindings, &query, &mut work)
        .err()
        .ok_or("count index exceeded its payment")?;
    assert!(error.message.contains("budget"));
    assert!(rejected.index.is_none());
    let mut counts = Cardinalities::default();
    let mut work = 0;
    assert_eq!(
        counts
            .count(&bindings, &query, &mut work)
            .map_err(|e| format!("{e:?}"))?,
        3
    );
    let before = work;
    let first_entries = counts.first.len();
    let index = counts.index.as_deref().ok_or("actual subtree index")?;
    let ends = index.ends.capacity();
    let positions: usize = index.positions.values().map(Vec::capacity).sum();
    for _ in 0..100 {
        assert_eq!(
            counts
                .count(&bindings, &query, &mut work)
                .map_err(|e| format!("{e:?}"))?,
            3
        );
    }
    let index = counts.index.as_deref().ok_or("warm subtree index")?;
    assert_eq!(counts.first.len(), first_entries);
    assert_eq!(index.ends.capacity(), ends);
    assert_eq!(
        index.positions.values().map(Vec::capacity).sum::<usize>(),
        positions
    );
    assert!(
        work - before < 1400,
        "warm queries rescanned captured subtrees"
    );
    Ok(())
}
